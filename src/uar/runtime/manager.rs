use crate::llm::{LlmSettings, Message, MessageRole, Orchestrator};
use crate::mcp::registry::McpRegistry;
use crate::session::SessionStore;
use crate::uar::domain::{
    artifact::AgentArtifact,
    context::ContextConfig,
    events::NormalizedEvent,
    runs::{Run, RunStatus},
};
use crate::uar::runtime::context::manager::ContextManager;
use crate::uar::runtime::skills::SkillRegistry;
use futures::StreamExt;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{RwLock, broadcast};
use uuid::Uuid;
use tracing::instrument;

#[derive(Clone, Debug)]
pub struct RunManager {
    // Map run_id -> (Run metadata, broadcast sender)
    active_runs: Arc<RwLock<HashMap<String, (Run, broadcast::Sender<NormalizedEvent>)>>>,
    settings: LlmSettings,
    global_mcp: Arc<McpRegistry>,
    sessions: SessionStore,
    skills: Arc<RwLock<SkillRegistry>>,
    vector_matcher: Arc<crate::uar::runtime::matching::VectorMatcher>,
    tag_matcher: Arc<crate::uar::runtime::matching::TagMatcher>,
    context_manager: Arc<ContextManager>,
    // Persistence layer (optional)
    pub persistence: Option<Arc<dyn crate::uar::persistence::PersistenceLayer>>,
}

impl RunManager {
    pub async fn new(
        settings: LlmSettings,
        global_mcp: Arc<McpRegistry>,
        sessions: SessionStore,
        skills: Arc<RwLock<SkillRegistry>>,
        vector_matcher: Arc<crate::uar::runtime::matching::VectorMatcher>,
        persistence: Option<Arc<dyn crate::uar::persistence::PersistenceLayer>>,
    ) -> Self {
        // Initialize vector matcher if not already (caller should ideally do this)
        if let Err(e) = vector_matcher.initialize().await {
            tracing::error!("Failed to initialize VectorMatcher: {:?}", e);
        }

        let tag_matcher = Arc::new(crate::uar::runtime::matching::TagMatcher::new());
        let context_manager = Arc::new(ContextManager::new(ContextConfig::default()));

        Self {
            active_runs: Arc::new(RwLock::new(HashMap::new())),
            settings,
            global_mcp,
            sessions,
            skills,
            vector_matcher,
            tag_matcher,
            context_manager,
            persistence,
        }
    }

    #[instrument(
        skip(self, artifact, input),
        fields(
            agent_id = %artifact.id, 
            session_id = ?session_id, 
            user_id = ?user_id,
            run_id = tracing::field::Empty
        )
    )]
    pub async fn start_run(
        &self,
        artifact: AgentArtifact,
        input: String,
        session_id: Option<String>,
        user_id: Option<String>,
    ) -> String {
        let run_id = Uuid::new_v4().to_string();
        tracing::Span::current().record("run_id", &run_id);
        tracing::info!("Starting new run");
        let (tx, _) = broadcast::channel(100); // Buffer size 100

        // 1. Resolve Session
        let session = if let Some(id) = session_id {
            self.sessions.get_or_create(&id)
        } else {
            self.sessions.create()
        };

        // 2. Add User Message
        session.add_user_message(&input);

        let run = Run {
            run_id: run_id.clone(),
            agent_id: artifact.id.clone(),
            conversation_id: Some(session.id().to_string()),
            user_id,
            status: RunStatus::Running,
            context: serde_json::json!({ "input": input }),
        };

        {
            let mut runs = self.active_runs.write().await;
            runs.insert(run_id.clone(), (run, tx.clone()));
        }

        // 3. Prepare Messages
        // We prioritize the Artifact's system prompt.
        let mut messages = Vec::new();
        let mut system_prompt = artifact.prompt.system.clone();

        // RAG Retrieval
        if let Some(db) = &self.persistence {
            match self.vector_matcher.embed_batch(vec![input.clone()]).await {
                Ok(embeddings) => {
                    if let Some(query_vec) = embeddings.first() {
                        match db.search_knowledge(query_vec, 3, 0.7).await {
                            Ok(matches) => {
                                if !matches.is_empty() {
                                    system_prompt.push_str("\n\n[RELEVANT KNOWLEDGE]\n");
                                    for m in matches {
                                        system_prompt.push_str(&format!("- {}\n", m.chunk.content));
                                    }
                                }
                            }
                            Err(e) => tracing::error!("RAG search failed: {:?}", e),
                        }
                    }
                }
                Err(e) => tracing::error!("RAG embedding failed: {:?}", e),
            }
        }

        // SKILL INJECTION: Composite Matcher (Tag -> Vector -> LLM Selection in future)
        let skills_registry = self.skills.read().await;
        // Ensure skills are indexed for vector matching
        // Ideal optimization: Index on separate background task or when skills loaded.
        // For now, we index if needed on query (handled internally by VectorMatcher, but cache validity depends on it).
        // Since we share the matcher, we should be careful.
        // VectorMatcher::match_skills calls index_skills if cache empty.

        let mut matched_skills = HashMap::new();

        // 1. Tag Matching (High Confidence)
        if let Ok(matches) = crate::uar::domain::matching::SkillMatcher::match_skills(
            self.tag_matcher.as_ref(),
            &input,
            &skills_registry,
        )
        .await
        {
            for m in matches {
                matched_skills.insert(m.skill_id.clone(), m.skill);
            }
        }

        // 2. Vector Matching (Medium Confidence)
        if let Ok(matches) = crate::uar::domain::matching::SkillMatcher::match_skills(
            self.vector_matcher.as_ref(),
            &input,
            &skills_registry,
        )
        .await
        {
            for m in matches {
                // Don't overwrite explicit tag matches if they exist, but here we just dedup by ID
                matched_skills.entry(m.skill_id.clone()).or_insert(m.skill);
            }
        }

        let sorted_skills: Vec<_> = matched_skills.values().collect();
        // Collect registries to merge (starting with global)
        let mut registries_to_merge = Vec::new();

        for skill in sorted_skills {
            // Append skill prompt overlay
            system_prompt.push_str("\n\n[SKILL: ");
            system_prompt.push_str(&skill.title);
            system_prompt.push_str("]\n");
            system_prompt.push_str(&skill.prompt_overlay);

            // Init Skill Tools
            if let Some(config) = &skill.mcp_config {
                match McpRegistry::from_config(config).await {
                    Ok(reg) => registries_to_merge.push(reg),
                    Err(e) => {
                        tracing::error!("Failed to init tools for skill {}: {:?}", skill.title, e)
                    }
                }
            }
        }

        messages.push(Message {
            role: MessageRole::System,
            content: system_prompt,
            tool_call_id: None,
            tool_calls: None,
        });
        messages.extend(session.messages());

        // Context Management
        let (optimized_messages, context_action) =
            self.context_manager.apply(messages, 128000).await;
        let messages = optimized_messages;
        if let Some(act) = context_action {
            let _ = tx.send(NormalizedEvent::ContextAction(act));
        }

        // Spawn async execution task
        // Create per-run Orchestrator.

        // Merge registries
        let mut final_mcp = (*self.global_mcp).clone();
        for reg in registries_to_merge {
            final_mcp = final_mcp.merge(&reg);
        }
        let mcp = Arc::new(final_mcp);

        let settings = self.settings.clone();

        let orchestrator = Arc::new(Orchestrator::new(settings, mcp));

        let execute_run_id = run_id.clone();
        let execute_agent_id = artifact.id.clone();
        let tx_clone = tx.clone();
        let execution_session = session.clone();

        tokio::spawn(async move {
            // 1. Run Start
            let _ = tx_clone.send(NormalizedEvent::RunStart {
                run_id: execute_run_id.clone(),
                agent_id: execute_agent_id,
            });

            let mut accumulated_content = String::new();
            let mut accumulated_tool_calls: Vec<crate::llm::ToolCall> = Vec::new();

            // 2. Execute Orchestrator
            match orchestrator.chat_with_history(messages).await {
                Ok(stream) => {
                    futures::pin_mut!(stream);
                    while let Some(base_event) = stream.next().await {
                        // Map base NormalizedEvent to domain NormalizedEvent with run_id
                        let uar_event = match base_event {
                            crate::normalized::NormalizedEvent::MessageDelta { text } => {
                                accumulated_content.push_str(&text);
                                Some(NormalizedEvent::ChatDelta {
                                    run_id: execute_run_id.clone(),
                                    text_delta: text,
                                })
                            }
                            crate::normalized::NormalizedEvent::ThinkingDelta { text } => {
                                Some(NormalizedEvent::ReasoningDelta {
                                    run_id: execute_run_id.clone(),
                                    text_delta: text,
                                })
                            }
                            crate::normalized::NormalizedEvent::ReasoningDelta { text } => {
                                Some(NormalizedEvent::ReasoningDelta {
                                    run_id: execute_run_id.clone(),
                                    text_delta: text,
                                })
                            }
                            crate::normalized::NormalizedEvent::ToolCallDelta {
                                call_index: _,
                                id,
                                name: _,
                                arguments_delta,
                            } => {
                                if let (Some(tid), Some(delta)) = (id, arguments_delta) {
                                    Some(NormalizedEvent::ToolDelta {
                                        run_id: execute_run_id.clone(),
                                        tool_call_id: tid,
                                        delta: serde_json::Value::String(delta),
                                    })
                                } else {
                                    None
                                }
                            }
                            crate::normalized::NormalizedEvent::ToolCallComplete {
                                call_index: _,
                                id,
                                name,
                                arguments_json,
                            } => {
                                accumulated_tool_calls.push(crate::llm::ToolCall {
                                    id: id.clone(),
                                    call_type: "function".to_string(),
                                    function: crate::llm::ToolCallFunction {
                                        name: name.clone(),
                                        arguments: arguments_json.clone(),
                                    },
                                });

                                Some(NormalizedEvent::ToolStart {
                                    run_id: execute_run_id.clone(),
                                    tool_call_id: id,
                                    tool: name,
                                    input: serde_json::from_str(&arguments_json)
                                        .unwrap_or(serde_json::Value::String(arguments_json)),
                                })
                            }
                            crate::normalized::NormalizedEvent::ToolResult {
                                id,
                                name: _,
                                content,
                                success,
                            } => {
                                if !accumulated_content.is_empty()
                                    || !accumulated_tool_calls.is_empty()
                                {
                                    execution_session.add_assistant_with_tool_calls(
                                        if accumulated_content.is_empty() {
                                            None
                                        } else {
                                            Some(accumulated_content.clone())
                                        },
                                        accumulated_tool_calls.clone(),
                                    );
                                    accumulated_content.clear();
                                    accumulated_tool_calls.clear();
                                }

                                execution_session.add_tool_result(id.clone(), content.clone());

                                Some(NormalizedEvent::ToolEnd {
                                    run_id: execute_run_id.clone(),
                                    tool_call_id: id,
                                    output: serde_json::from_str(&content)
                                        .unwrap_or(serde_json::Value::String(content)),
                                    ok: success,
                                })
                            }
                            crate::normalized::NormalizedEvent::Error { message, code } => {
                                Some(NormalizedEvent::Error {
                                    run_id: execute_run_id.clone(),
                                    message,
                                    code: code.unwrap_or_default(),
                                })
                            }
                            _ => None, // Ignore other events for now
                        };

                        if let Some(evt) = uar_event {
                            let _ = tx_clone.send(evt);
                        }
                    }
                }
                Err(e) => {
                    let _ = tx_clone.send(NormalizedEvent::Error {
                        run_id: execute_run_id.clone(),
                        message: e.to_string(),
                        code: String::new(),
                    });
                }
            }

            if !accumulated_content.is_empty() {
                execution_session.add_assistant_message(accumulated_content);
            }

            let _ = tx_clone.send(NormalizedEvent::RunDone {
                run_id: execute_run_id,
            });
        });

        run_id
    }

    pub async fn subscribe(&self, run_id: &str) -> Option<broadcast::Receiver<NormalizedEvent>> {
        let runs = self.active_runs.read().await;
        runs.get(run_id).map(|(_, tx)| tx.subscribe())
    }

    pub async fn get_run(&self, run_id: &str) -> Option<Run> {
        let runs = self.active_runs.read().await;
        runs.get(run_id).map(|(run, _)| run.clone())
    }
}
