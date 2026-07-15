use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelEntry {
    pub id: Uuid,
    pub name: String,
    pub version: String,
    pub artifact_hash: String,
    pub registered_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyEntry {
    pub id: Uuid,
    pub name: String,
    pub version: String,
    pub artifact_hash: String,
    pub registered_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct Registry {
    models: Arc<RwLock<HashMap<Uuid, ModelEntry>>>,
    policies: Arc<RwLock<HashMap<Uuid, PolicyEntry>>>,
}

impl Registry {
    pub fn new() -> Self {
        Self {
            models: Arc::new(RwLock::new(HashMap::new())),
            policies: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn register_model(&self, entry: ModelEntry) -> Uuid {
        let id = entry.id;
        self.models.write().unwrap().insert(id, entry);
        id
    }

    pub fn register_policy(&self, entry: PolicyEntry) -> Uuid {
        let id = entry.id;
        self.policies.write().unwrap().insert(id, entry);
        id
    }

    pub fn get_model(&self, id: &Uuid) -> Option<ModelEntry> {
        self.models.read().unwrap().get(id).cloned()
    }

    pub fn get_policy(&self, id: &Uuid) -> Option<PolicyEntry> {
        self.policies.read().unwrap().get(id).cloned()
    }
}
