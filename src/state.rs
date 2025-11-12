//! State management for JCL

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// State backend trait
pub trait StateBackend: Send + Sync {
    /// Load state
    fn load(&self) -> Result<State>;

    /// Save state
    fn save(&self, state: &State) -> Result<()>;

    /// Acquire lock
    fn lock(&self) -> Result<StateLock>;

    /// Release lock
    fn unlock(&self, lock: StateLock) -> Result<()>;
}

/// JCL state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    pub version: u32,
    pub serial: u64,
    pub resources: HashMap<String, ResourceInstance>,
    pub outputs: HashMap<String, String>,
}

impl State {
    /// Create a new empty state
    pub fn new() -> Self {
        Self {
            version: 1,
            serial: 0,
            resources: HashMap::new(),
            outputs: HashMap::new(),
        }
    }

    /// Get a resource by address
    pub fn get_resource(&self, address: &str) -> Option<&ResourceInstance> {
        self.resources.get(address)
    }

    /// Add or update a resource
    pub fn set_resource(&mut self, address: String, instance: ResourceInstance) {
        self.resources.insert(address, instance);
        self.serial += 1;
    }

    /// Remove a resource
    pub fn remove_resource(&mut self, address: &str) -> Option<ResourceInstance> {
        self.serial += 1;
        self.resources.remove(address)
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

/// A resource instance in state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceInstance {
    pub resource_type: String,
    pub provider: String,
    pub id: String,
    pub attributes: HashMap<String, String>,
    pub dependencies: Vec<String>,
}

/// State lock
pub struct StateLock {
    pub id: String,
    pub holder: String,
    pub acquired_at: std::time::SystemTime,
}

/// Local file state backend
pub struct LocalBackend {
    path: PathBuf,
}

impl LocalBackend {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    fn lock_path(&self) -> PathBuf {
        self.path.with_extension("lock")
    }
}

impl StateBackend for LocalBackend {
    fn load(&self) -> Result<State> {
        if !self.path.exists() {
            return Ok(State::new());
        }

        let content = std::fs::read_to_string(&self.path)?;
        let state: State = serde_json::from_str(&content)?;
        Ok(state)
    }

    fn save(&self, state: &State) -> Result<()> {
        let content = serde_json::to_string_pretty(state)?;
        std::fs::write(&self.path, content)?;
        Ok(())
    }

    fn lock(&self) -> Result<StateLock> {
        let lock_path = self.lock_path();

        // Simple file-based locking (not distributed)
        if lock_path.exists() {
            anyhow::bail!("State is already locked");
        }

        let lock = StateLock {
            id: uuid::Uuid::new_v4().to_string(),
            holder: "local".to_string(),
            acquired_at: std::time::SystemTime::now(),
        };

        let lock_content = serde_json::to_string(&lock)?;
        std::fs::write(&lock_path, lock_content)?;

        Ok(lock)
    }

    fn unlock(&self, _lock: StateLock) -> Result<()> {
        let lock_path = self.lock_path();
        if lock_path.exists() {
            std::fs::remove_file(&lock_path)?;
        }
        Ok(())
    }
}

/// S3 state backend (stub)
pub struct S3Backend {
    bucket: String,
    key: String,
}

impl S3Backend {
    pub fn new(bucket: String, key: String) -> Self {
        Self { bucket, key }
    }
}

impl StateBackend for S3Backend {
    fn load(&self) -> Result<State> {
        // TODO: Implement S3 state loading
        Ok(State::new())
    }

    fn save(&self, _state: &State) -> Result<()> {
        // TODO: Implement S3 state saving
        Ok(())
    }

    fn lock(&self) -> Result<StateLock> {
        // TODO: Implement DynamoDB-based locking
        Ok(StateLock {
            id: uuid::Uuid::new_v4().to_string(),
            holder: "s3".to_string(),
            acquired_at: std::time::SystemTime::now(),
        })
    }

    fn unlock(&self, _lock: StateLock) -> Result<()> {
        // TODO: Implement DynamoDB lock release
        Ok(())
    }
}

// Add uuid crate to Cargo.toml
