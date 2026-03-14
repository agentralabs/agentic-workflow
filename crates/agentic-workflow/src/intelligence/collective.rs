use std::collections::HashMap;

use chrono::Utc;
use uuid::Uuid;
use serde::Serialize;

/// Workflow collective — community sharing of workflow patterns.
pub struct CollectiveEngine {
    shared_items: HashMap<String, CollectiveItem>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CollectiveItem {
    pub id: String,
    pub name: String,
    pub description: String,
    pub workflow_definition: serde_json::Value,
    pub author: String,
    pub tags: Vec<String>,
    pub rating: f64,
    pub rating_count: u32,
    pub download_count: u64,
    pub shared_at: chrono::DateTime<chrono::Utc>,
    pub privacy_verified: bool,
}

impl CollectiveEngine {
    pub fn new() -> Self {
        Self {
            shared_items: HashMap::new(),
        }
    }

    /// Share a workflow with the collective.
    pub fn share(
        &mut self,
        name: &str,
        description: &str,
        workflow_definition: serde_json::Value,
        author: &str,
        tags: Vec<String>,
    ) -> String {
        let id = Uuid::new_v4().to_string();
        let item = CollectiveItem {
            id: id.clone(),
            name: name.to_string(),
            description: description.to_string(),
            workflow_definition,
            author: author.to_string(),
            tags,
            rating: 0.0,
            rating_count: 0,
            download_count: 0,
            shared_at: Utc::now(),
            privacy_verified: false,
        };

        self.shared_items.insert(id.clone(), item);
        id
    }

    /// Search community workflows.
    pub fn search(&self, query: &str) -> Vec<&CollectiveItem> {
        let query_lower = query.to_lowercase();
        self.shared_items
            .values()
            .filter(|item| {
                item.name.to_lowercase().contains(&query_lower)
                    || item.description.to_lowercase().contains(&query_lower)
                    || item.tags.iter().any(|t| t.to_lowercase().contains(&query_lower))
            })
            .collect()
    }

    /// Get a collective item.
    pub fn get(&self, id: &str) -> Option<&CollectiveItem> {
        self.shared_items.get(id)
    }

    /// Apply (download) a community workflow.
    pub fn apply(&mut self, id: &str) -> Option<serde_json::Value> {
        if let Some(item) = self.shared_items.get_mut(id) {
            item.download_count += 1;
            Some(item.workflow_definition.clone())
        } else {
            None
        }
    }

    /// Rate a community workflow.
    pub fn rate(&mut self, id: &str, rating: f64) -> bool {
        if let Some(item) = self.shared_items.get_mut(id) {
            let total = item.rating * item.rating_count as f64 + rating;
            item.rating_count += 1;
            item.rating = total / item.rating_count as f64;
            true
        } else {
            false
        }
    }

    /// Verify no private data in a shared workflow.
    pub fn verify_privacy(&mut self, id: &str) -> bool {
        if let Some(item) = self.shared_items.get_mut(id) {
            // Privacy check: ensure no secrets, tokens, or PII in definition
            let def_str = item.workflow_definition.to_string().to_lowercase();
            let suspicious = ["password", "secret", "token", "api_key", "private_key"];
            let clean = !suspicious.iter().any(|s| def_str.contains(s));
            item.privacy_verified = clean;
            clean
        } else {
            false
        }
    }

    /// List all shared items.
    pub fn list_all(&self) -> Vec<&CollectiveItem> {
        self.shared_items.values().collect()
    }
}

impl Default for CollectiveEngine {
    fn default() -> Self {
        Self::new()
    }
}
