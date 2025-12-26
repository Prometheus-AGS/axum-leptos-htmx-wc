//! Leiden Community Detection Algorithm
//!
//! Implements the Leiden algorithm for detecting communities in knowledge graphs.
//! Uses petgraph for graph representation and iterative optimization.

use crate::uar::domain::graph::{Community, Entity, Relationship};
use anyhow::Result;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

// =============================================================================
// Leiden Community Detector
// =============================================================================

/// Parameters for community detection.
#[derive(Debug, Clone)]
pub struct LeidenConfig {
    /// Resolution parameter (higher = smaller communities)
    pub resolution: f64,
    /// Maximum iterations for optimization
    pub max_iterations: usize,
    /// Minimum improvement threshold to continue
    pub min_improvement: f64,
    /// Random seed for reproducibility
    pub seed: Option<u64>,
}

impl Default for LeidenConfig {
    fn default() -> Self {
        Self {
            resolution: 1.0,
            max_iterations: 100,
            min_improvement: 0.0001,
            seed: None,
        }
    }
}

/// Community detector using the Leiden algorithm.
#[derive(Debug)]
pub struct LeidenCommunityDetector {
    config: LeidenConfig,
}

impl LeidenCommunityDetector {
    /// Create a new Leiden detector with default config.
    pub fn new() -> Self {
        Self {
            config: LeidenConfig::default(),
        }
    }

    /// Create with custom configuration.
    pub fn with_config(config: LeidenConfig) -> Self {
        Self { config }
    }

    /// Detect communities in a knowledge graph.
    ///
    /// Returns a list of communities, each containing a set of entity IDs.
    pub fn detect_communities(
        &self,
        entities: &[Entity],
        relationships: &[Relationship],
    ) -> Result<Vec<Community>> {
        if entities.is_empty() {
            return Ok(Vec::new());
        }

        // Build graph from entities and relationships
        let (graph, id_to_node, _node_to_id) = self.build_graph(entities, relationships);

        // Initialize: each node in its own community
        let mut communities: Vec<HashSet<NodeIndex>> = graph
            .node_indices()
            .map(|n| {
                let mut s = HashSet::new();
                s.insert(n);
                s
            })
            .collect();

        // Leiden optimization loop
        let mut improved = true;
        let mut iterations = 0;

        while improved && iterations < self.config.max_iterations {
            improved = false;
            iterations += 1;

            // Phase 1: Local moving of nodes
            for node in graph.node_indices() {
                let current_comm = self.find_community(&communities, node);

                // Calculate modularity gain for moving to each neighbor's community
                let mut best_gain = 0.0;
                let mut best_comm = current_comm;

                for edge in graph.edges(node) {
                    let neighbor = edge.target();
                    let neighbor_comm = self.find_community(&communities, neighbor);

                    if neighbor_comm != current_comm {
                        let gain = self.calculate_modularity_gain(
                            &graph,
                            node,
                            &communities[current_comm],
                            &communities[neighbor_comm],
                        );

                        if gain > best_gain {
                            best_gain = gain;
                            best_comm = neighbor_comm;
                        }
                    }
                }

                // Move node if improvement found
                if best_comm != current_comm && best_gain > self.config.min_improvement {
                    communities[current_comm].remove(&node);
                    communities[best_comm].insert(node);
                    improved = true;
                }
            }

            // Phase 2: Refine communities (simplified - full Leiden includes additional refinement)
            self.refine_communities(&mut communities);
        }

        // Convert to Community structs
        let now = chrono::Utc::now().to_rfc3339();
        let entity_map: HashMap<NodeIndex, String> = id_to_node
            .iter()
            .map(|(id, &node)| (node, id.clone()))
            .collect();

        let result: Vec<Community> = communities
            .into_iter()
            .filter(|c| !c.is_empty())
            .enumerate()
            .map(|(level, members)| {
                let entity_ids: Vec<String> = members
                    .iter()
                    .filter_map(|n| entity_map.get(n).cloned())
                    .collect();

                Community {
                    id: Uuid::new_v4().to_string(),
                    level: level as u32,
                    entity_ids,
                    summary: String::new(), // To be filled by LLM later
                    embedding: Vec::new(),  // To be filled by embedding model
                    created_at: now.clone(),
                }
            })
            .collect();

        tracing::info!(
            "Leiden detected {} communities in {} iterations",
            result.len(),
            iterations
        );

        Ok(result)
    }

    /// Build a petgraph DiGraph from entities and relationships.
    fn build_graph(
        &self,
        entities: &[Entity],
        relationships: &[Relationship],
    ) -> (
        DiGraph<String, f32>,
        HashMap<String, NodeIndex>,
        HashMap<NodeIndex, String>,
    ) {
        let mut graph = DiGraph::new();
        let mut id_to_node = HashMap::new();
        let mut node_to_id = HashMap::new();

        // Add nodes
        for entity in entities {
            let node = graph.add_node(entity.id.clone());
            id_to_node.insert(entity.id.clone(), node);
            node_to_id.insert(node, entity.id.clone());
        }

        // Add edges
        for rel in relationships {
            if let (Some(&source), Some(&target)) = (
                id_to_node.get(&rel.source_id),
                id_to_node.get(&rel.target_id),
            ) {
                graph.add_edge(source, target, rel.weight);
            }
        }

        (graph, id_to_node, node_to_id)
    }

    /// Find which community a node belongs to.
    fn find_community(&self, communities: &[HashSet<NodeIndex>], node: NodeIndex) -> usize {
        for (i, comm) in communities.iter().enumerate() {
            if comm.contains(&node) {
                return i;
            }
        }
        0 // Should not happen
    }

    /// Calculate modularity gain for moving a node.
    fn calculate_modularity_gain(
        &self,
        graph: &DiGraph<String, f32>,
        node: NodeIndex,
        from_comm: &HashSet<NodeIndex>,
        to_comm: &HashSet<NodeIndex>,
    ) -> f64 {
        // Simplified modularity calculation
        // Full implementation would use weighted edges and proper normalization

        let in_edges_to: f64 = graph
            .edges(node)
            .filter(|e| to_comm.contains(&e.target()))
            .map(|e| *e.weight() as f64)
            .sum();

        let in_edges_from: f64 = graph
            .edges(node)
            .filter(|e| from_comm.contains(&e.target()) && e.target() != node)
            .map(|e| *e.weight() as f64)
            .sum();

        (in_edges_to - in_edges_from) * self.config.resolution
    }

    /// Refine communities by removing empty ones.
    fn refine_communities(&self, communities: &mut Vec<HashSet<NodeIndex>>) {
        communities.retain(|c| !c.is_empty());
    }
}

impl Default for LeidenCommunityDetector {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::uar::domain::graph::EntityType;

    #[test]
    fn test_empty_graph() {
        let detector = LeidenCommunityDetector::new();
        let result = detector.detect_communities(&[], &[]).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_single_entity() {
        let detector = LeidenCommunityDetector::new();
        let entities = vec![Entity {
            id: "e1".to_string(),
            canonical_name: "Test".to_string(),
            entity_type: EntityType::Concept,
            description: None,
            embedding: Vec::new(),
            source_chunk_ids: Vec::new(),
            created_at: "2024-01-01".to_string(),
        }];

        let result = detector.detect_communities(&entities, &[]).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].entity_ids.len(), 1);
    }
}
