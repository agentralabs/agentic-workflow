use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::Mutex;

use agentic_workflow::engine::store::WorkflowStore;

/// Autonomic profile for session behavior.
#[derive(Debug, Clone)]
pub enum AutonomicProfile {
    /// Desktop: 30s autosave, moderate maintenance
    Desktop,
    /// Server: 15s autosave, aggressive maintenance
    Server,
    /// Terminal: 60s autosave, minimal maintenance
    Terminal,
}

impl AutonomicProfile {
    pub fn autosave_interval(&self) -> Duration {
        match self {
            Self::Desktop => Duration::from_secs(30),
            Self::Server => Duration::from_secs(15),
            Self::Terminal => Duration::from_secs(60),
        }
    }

    pub fn maintenance_interval(&self) -> Duration {
        match self {
            Self::Desktop => Duration::from_secs(300),
            Self::Server => Duration::from_secs(120),
            Self::Terminal => Duration::from_secs(600),
        }
    }
}

/// Session manager — wraps WorkflowStore with lifecycle management.
pub struct SessionManager {
    store: WorkflowStore,
    session_id: String,
    profile: AutonomicProfile,
    started_at: Instant,
    last_save: Instant,
    mutation_count: u64,
    data_path: PathBuf,
}

impl SessionManager {
    /// Open a session with a workflow store at the given path.
    pub fn open(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let path = path.as_ref().to_path_buf();
        let store = WorkflowStore::open(&path)
            .map_err(|e| anyhow::anyhow!("Failed to open store: {}", e))?;

        let session_id = uuid::Uuid::new_v4().to_string();
        let now = Instant::now();

        eprintln!("SessionManager: opened session {} at {}", session_id, path.display());

        Ok(Self {
            store,
            session_id,
            profile: AutonomicProfile::Desktop,
            started_at: now,
            last_save: now,
            mutation_count: 0,
            data_path: path,
        })
    }

    /// Open in-memory session (testing).
    pub fn open_memory() -> Self {
        Self {
            store: WorkflowStore::open_memory(),
            session_id: uuid::Uuid::new_v4().to_string(),
            profile: AutonomicProfile::Desktop,
            started_at: Instant::now(),
            last_save: Instant::now(),
            mutation_count: 0,
            data_path: PathBuf::new(),
        }
    }

    /// Get mutable access to the store.
    pub fn store_mut(&mut self) -> &mut WorkflowStore {
        self.mutation_count += 1;
        &mut self.store
    }

    /// Get read access to the store.
    pub fn store(&self) -> &WorkflowStore {
        &self.store
    }

    /// Get session ID.
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Set autonomic profile.
    pub fn set_profile(&mut self, profile: AutonomicProfile) {
        self.profile = profile;
    }

    /// Get mutation count since session start.
    pub fn mutation_count(&self) -> u64 {
        self.mutation_count
    }

    /// Get session uptime.
    pub fn uptime(&self) -> Duration {
        self.started_at.elapsed()
    }

    /// Run periodic maintenance tick (autosave if needed).
    pub fn maintenance_tick(&mut self) -> anyhow::Result<()> {
        let since_save = self.last_save.elapsed();

        if since_save >= self.profile.autosave_interval() && self.store.is_dirty() {
            self.store.save()
                .map_err(|e| anyhow::anyhow!("Autosave failed: {}", e))?;
            self.last_save = Instant::now();
            eprintln!(
                "SessionManager: autosave ({} mutations, {}s since last save)",
                self.mutation_count,
                since_save.as_secs()
            );
        }

        Ok(())
    }

    /// Force save.
    pub fn force_save(&mut self) -> anyhow::Result<()> {
        self.store.save()
            .map_err(|e| anyhow::anyhow!("Save failed: {}", e))?;
        self.last_save = Instant::now();
        Ok(())
    }

    /// Get session stats.
    pub fn stats(&self) -> serde_json::Value {
        serde_json::json!({
            "session_id": self.session_id,
            "workflow_count": self.store.count(),
            "mutation_count": self.mutation_count,
            "uptime_secs": self.uptime().as_secs(),
            "data_path": self.data_path.display().to_string(),
            "profile": format!("{:?}", self.profile),
            "is_dirty": self.store.is_dirty(),
        })
    }

    /// Shutdown session gracefully.
    pub fn shutdown(&mut self) -> anyhow::Result<()> {
        if self.store.is_dirty() {
            self.force_save()?;
        }
        eprintln!(
            "SessionManager: shutdown session {} ({} mutations, {}s uptime)",
            self.session_id, self.mutation_count, self.uptime().as_secs()
        );
        Ok(())
    }
}

impl Drop for SessionManager {
    fn drop(&mut self) {
        if let Err(e) = self.shutdown() {
            eprintln!("SessionManager: shutdown error on drop: {}", e);
        }
    }
}

/// Create a shared session manager for use in MCP server.
pub fn create_shared_session(path: impl AsRef<Path>) -> anyhow::Result<Arc<Mutex<SessionManager>>> {
    let manager = SessionManager::open(path)?;
    Ok(Arc::new(Mutex::new(manager)))
}

/// Spawn autosave background task.
pub async fn spawn_autosave(session: Arc<Mutex<SessionManager>>) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(10));
        loop {
            interval.tick().await;
            let mut mgr = session.lock().await;
            if let Err(e) = mgr.maintenance_tick() {
                eprintln!("Autosave tick error: {}", e);
            }
        }
    });
}
