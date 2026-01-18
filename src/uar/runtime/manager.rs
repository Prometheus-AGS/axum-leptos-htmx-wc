use crate::llm::{LlmSettings, Message, MessageRole, Orchestrator};
use crate::mcp::registry::McpRegistry;
use crate::session::SessionStore;
use crate::uar::domain::{
    artifact::AgentArtifact,
    context::ContextConfig,
    events::{NormalizedEvent, StatePatchOp},
    runs::{Run, RunStatus},
};
use crate::uar::runtime::context::manager::ContextManager;
use crate::uar::runtime::matching::{ClassifierConfig, IntentClassifier, create_classifier};
use crate::uar::runtime::skills::SkillRegistry;
use futures::StreamExt;
use std::{
    collections::{HashMap, VecDeque},
    fmt::Write,
    sync::Arc,
};
use tokio::sync::{Mutex, RwLock, broadcast};
use tracing::instrument;
use uuid::Uuid;

const EVENT_HISTORY_LIMIT: usize = 512;

#[derive(Clone, Debug)]
pub struct StreamEvent {
    pub id: u64,
    pub event: NormalizedEvent,
}

#[derive(Debug)]
struct EventHistory {
    next_id: u64,
    buffer: VecDeque<StreamEvent>,
}

#[derive(Debug)]
struct RunStreamState {
    run: Run,
    sender: broadcast::Sender<StreamEvent>,
    history: Arc<Mutex<EventHistory>>,
}

#[derive(Clone, Debug)]
struct RunEventEmitter {
    sender: broadcast::Sender<StreamEvent>,
    history: Arc<Mutex<EventHistory>>,
}

impl RunEventEmitter {
    async fn emit(&self, event: NormalizedEvent) {
        let mut history = self.history.lock().await;
        let id = history.next_id;
        history.next_id = history.next_id.saturating_add(1);

        let stream_event = StreamEvent { id, event };
        history.buffer.push_back(stream_event.clone());
        if history.buffer.len() > EVENT_HISTORY_LIMIT {
            history.buffer.pop_front();
        }

        let _ = self.sender.send(stream_event);
    }
}

type ActiveRunMap = HashMap<String, RunStreamState>;

#[derive(Clone)]
pub struct RunManager {
    // Map run_id -> (Run metadata, broadcast sender)
    active_runs: Arc<RwLock<ActiveRunMap>>,
    settings: LlmSettings,
    global_mcp: Arc<McpRegistry>,
    sessions: SessionStore,
    skills: Arc<RwLock<SkillRegistry>>,
    vector_matcher: Arc<crate::uar::runtime::matching::VectorMatcher>,
    tag_matcher: Arc<crate::uar::runtime::matching::TagMatcher>,
    context_manager: Arc<ContextManager>,
    /// Intent classifier for skill matching
    intent_classifier: Arc<dyn IntentClassifier>,
    /// Classifier configuration
    classifier_config: ClassifierConfig,
    // Persistence layer (optional)
    pub persistence: Option<Arc<dyn crate::uar::persistence::PersistenceLayer>>,
}

impl std::fmt::Debug for RunManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RunManager")
            .field("active_runs_count", &"<locked>")
            .field("settings", &self.settings)
            .field("classifier_config", &self.classifier_config)
            .finish_non_exhaustive()
    }
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
        Self::with_classifier_config(
            settings,
            global_mcp,
            sessions,
            skills,
            vector_matcher,
            persistence,
            ClassifierConfig::default(),
        )
        .await
    }

    /// Creates a new RunManager with a custom classifier configuration.
    pub async fn with_classifier_config(
        settings: LlmSettings,
        global_mcp: Arc<McpRegistry>,
        sessions: SessionStore,
        skills: Arc<RwLock<SkillRegistry>>,
        vector_matcher: Arc<crate::uar::runtime::matching::VectorMatcher>,
        persistence: Option<Arc<dyn crate::uar::persistence::PersistenceLayer>>,
        classifier_config: ClassifierConfig,
    ) -> Self {
        // Initialize vector matcher if not already (caller should ideally do this)
        if let Err(e) = vector_matcher.initialize().await {
            tracing::error!("Failed to initialize VectorMatcher: {:?}", e);
        }

        let tag_matcher = Arc::new(crate::uar::runtime::matching::TagMatcher::new());
        let context_manager = Arc::new(ContextManager::new(ContextConfig::default()));

        // Create intent classifier based on config
        let intent_classifier: Arc<dyn IntentClassifier> =
            Arc::from(create_classifier(&classifier_config));

        // Build the classifier index from existing skills
        {
            let skills_registry = skills.read().await;
            if let Err(e) = intent_classifier.rebuild_index(&skills_registry).await {
                tracing::error!("Failed to build intent classifier index: {:?}", e);
            }
        }

        tracing::info!(
            backend = ?classifier_config.backend,
            topk = classifier_config.topk,
            accept_threshold = classifier_config.accept_threshold,
            "Intent classifier initialized"
        );

        Self {
            active_runs: Arc::new(RwLock::new(HashMap::new())),
            settings,
            global_mcp,
            sessions,
            skills,
            vector_matcher,
            tag_matcher,
            context_manager,
            intent_classifier,
            classifier_config,
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
        let (tx, _) = broadcast::channel(256); // Buffer size 256
        let history = Arc::new(Mutex::new(EventHistory {
            next_id: 1,
            buffer: VecDeque::with_capacity(EVENT_HISTORY_LIMIT),
        }));
        let emitter = RunEventEmitter {
            sender: tx.clone(),
            history: Arc::clone(&history),
        };

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
            runs.insert(
                run_id.clone(),
                RunStreamState {
                    run,
                    sender: tx.clone(),
                    history: Arc::clone(&history),
                },
            );
        }

        // 3. Prepare Messages
        // We prioritize the Artifact's system prompt.
        let mut messages = Vec::new();
        let mut system_prompt = artifact.prompt.system.clone();

        // RAG Retrieval - scoped to agent's configured knowledge bases
        if artifact.memory.kb.enabled && let Some(db) = &self.persistence {
            match self.vector_matcher.embed_batch(vec![input.clone()]).await {
                Ok(embeddings) => {
                    if let Some(query_vec) = embeddings.first() {
                        // Get agent's configured KBs (or use all if empty)
                        let kb_names = &artifact.memory.kb.knowledge_bases;

                        let search_result = if kb_names.is_empty() {
                            // No specific KBs configured - search all
                            db.search_knowledge(query_vec, 3, 0.7).await
                        } else {
                            // Resolve KB names to IDs and search scoped
                            let mut kb_ids = Vec::new();
                            for name in kb_names {
                                if let Ok(Some(kb)) = db.get_knowledge_base_by_name(name).await {
                                    kb_ids.push(kb.id);
                                } else {
                                    tracing::warn!("Knowledge base not found: {}", name);
                                }
                            }

                            if kb_ids.is_empty() {
                                // All configured KBs were not found - fallback to all
                                tracing::warn!("No configured knowledge bases found, searching all");
                                db.search_knowledge(query_vec, 3, 0.7).await
                            } else {
                                let kb_id_refs: Vec<&str> =
                                    kb_ids.iter().map(String::as_str).collect();
                                db.search_knowledge_scoped(&kb_id_refs, query_vec, 3, 0.7).await
                            }
                        };

                        match search_result {
                            Ok(matches) => {
                                if !matches.is_empty() {
                                    system_prompt.push_str("\n\n[RELEVANT KNOWLEDGE]\n");
                                    for m in matches {
                                        let _ = writeln!(system_prompt, "- {}", m.chunk.content);
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

        // SKILL INJECTION: Intent Classification Pipeline
        let skills_registry = self.skills.read().await;

        // Use the configured intent classifier
        let classification_result = self
            .intent_classifier
            .classify(&input, &[], &skills_registry)
            .await;

        let matched_skills: Vec<_> = match classification_result {
            Ok(result) => {
                // Log classification scores for debugging/tuning
                tracing::debug!(
                    scores = ?result.scores.iter().map(|s| (&s.label, s.score)).collect::<Vec<_>>(),
                    out_of_scope = result.out_of_scope,
                    "Intent classification complete"
                );

                // Apply thresholds from config
                if result.should_accept(
                    self.classifier_config.accept_threshold,
                    self.classifier_config.margin_threshold,
                ) {
                    // Return skills that have associated skill data
                    result
                        .scores
                        .into_iter()
                        .filter_map(|score| score.skill)
                        .collect()
                } else if result.out_of_scope {
                    tracing::debug!("Query appears out-of-scope, no skills matched");
                    Vec::new()
                } else {
                    // Below threshold but not out-of-scope - still include top matches
                    // but log a warning for potential tuning
                    tracing::debug!(
                        top_score = ?result.scores.first().map(|s| s.score),
                        threshold = self.classifier_config.accept_threshold,
                        "Classification below threshold, including top matches anyway"
                    );
                    result
                        .scores
                        .into_iter()
                        .filter_map(|score| score.skill)
                        .collect()
                }
            }
            Err(e) => {
                tracing::error!("Intent classification failed: {:?}", e);
                // Fallback to legacy tag+vector matching
                let mut fallback_skills = HashMap::new();

                if let Ok(matches) = crate::uar::domain::matching::SkillMatcher::match_skills(
                    self.tag_matcher.as_ref(),
                    &input,
                    &skills_registry,
                )
                .await
                {
                    for m in matches {
                        fallback_skills.insert(m.skill_id.clone(), m.skill);
                    }
                }

                if let Ok(matches) = crate::uar::domain::matching::SkillMatcher::match_skills(
                    self.vector_matcher.as_ref(),
                    &input,
                    &skills_registry,
                )
                .await
                {
                    for m in matches {
                        fallback_skills.entry(m.skill_id.clone()).or_insert(m.skill);
                    }
                }

                fallback_skills.into_values().collect()
            }
        };

        let sorted_skills: Vec<_> = matched_skills.iter().collect();
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
                        tracing::error!("Failed to init tools for skill {}: {:?}", skill.title, e);
                    }
                }
            }
        }

        messages.push(Message {
            role: MessageRole::System,
            content: crate::llm::MessageContent::text(system_prompt),
            tool_call_id: None,
            tool_calls: None,
        });
        messages.extend(session.messages());

        // Context Management
        let (optimized_messages, context_action) =
            self.context_manager.apply(messages, 128_000).await;
        let messages = optimized_messages;
        if let Some(act) = context_action {
            emitter.emit(NormalizedEvent::ContextAction(act)).await;
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
        let emitter = emitter.clone();
        let execution_session = session.clone();

        tokio::spawn(async move {
            // 1. Run Start
            emitter
                .emit(NormalizedEvent::RunStart {
                run_id: execute_run_id.clone(),
                agent_id: execute_agent_id,
            })
            .await;

            emitter
                .emit(NormalizedEvent::StatePatch {
                    run_id: execute_run_id.clone(),
                    patch: vec![StatePatchOp {
                        op: "replace".to_string(),
                        path: "/run".to_string(),
                        value: Some(serde_json::json!({
                            "run_id": execute_run_id.clone(),
                            "conversation_id": execution_session.id(),
                            "status": "running"
                        })),
                    }],
                })
                .await;

            let mut accumulated_content = String::new();
            let mut accumulated_tool_calls: Vec<crate::llm::ToolCall> = Vec::new();
            let mut tool_call_indices: HashMap<String, usize> = HashMap::new();
            let mut tool_call_names: HashMap<String, String> = HashMap::new();

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
                                call_index,
                                id,
                                name,
                                arguments_delta,
                            } => {
                                if let (Some(tid), Some(delta)) = (id, arguments_delta) {
                                    tool_call_indices.insert(tid.clone(), call_index);
                                    if let Some(tool_name) = name {
                                        tool_call_names.insert(tid.clone(), tool_name);
                                    }
                                    Some(NormalizedEvent::ToolDelta {
                                        run_id: execute_run_id.clone(),
                                        call_index,
                                        tool_call_id: tid,
                                        delta: serde_json::Value::String(delta),
                                    })
                                } else {
                                    None
                                }
                            }
                            crate::normalized::NormalizedEvent::ToolCallComplete {
                                call_index,
                                id,
                                name,
                                arguments_json,
                            } => {
                                tool_call_indices.insert(id.clone(), call_index);
                                tool_call_names.insert(id.clone(), name.clone());
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
                                    call_index,
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
                                let call_index = tool_call_indices.get(&id).copied().unwrap_or(0);
                                let tool = tool_call_names
                                    .get(&id)
                                    .cloned()
                                    .unwrap_or_else(|| "tool".to_string());

                                Some(NormalizedEvent::ToolEnd {
                                    run_id: execute_run_id.clone(),
                                    call_index,
                                    tool_call_id: id,
                                    tool,
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
                            emitter.emit(evt).await;
                        }
                    }
                }
                Err(e) => {
                    emitter
                        .emit(NormalizedEvent::Error {
                        run_id: execute_run_id.clone(),
                        message: e.to_string(),
                        code: String::new(),
                    })
                    .await;
                }
            }

            if !accumulated_content.is_empty() {
                execution_session.add_assistant_message(accumulated_content);
            }

            emitter
                .emit(NormalizedEvent::RunDone {
                run_id: execute_run_id,
            })
            .await;
        });

        run_id
    }

    pub async fn subscribe(&self, run_id: &str) -> Option<broadcast::Receiver<StreamEvent>> {
        let runs = self.active_runs.read().await;
        runs.get(run_id).map(|state| state.sender.subscribe())
    }

    pub async fn history_since(
        &self,
        run_id: &str,
        last_event_id: Option<u64>,
    ) -> Option<Vec<StreamEvent>> {
        let runs = self.active_runs.read().await;
        let state = runs.get(run_id)?;
        let history = state.history.lock().await;
        let events = history
            .buffer
            .iter()
            .filter(|event| last_event_id.is_none_or(|id| event.id > id))
            .cloned()
            .collect();
        Some(events)
    }

    pub async fn get_run(&self, run_id: &str) -> Option<Run> {
        let runs = self.active_runs.read().await;
        runs.get(run_id).map(|state| state.run.clone())
    }
}
