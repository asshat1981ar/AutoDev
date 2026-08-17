use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::BufWriter,
    path::{Path, PathBuf},
    sync::RwLock,
};

use forge_core::{Evidence, TaskGraph, VerifiedOrchestratorState};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::ObjectiveView;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObjectiveSnapshot {
    pub view: ObjectiveView,
    pub graph: TaskGraph,
    pub orchestrator: VerifiedOrchestratorState,
    #[serde(default)]
    pub evidence: Vec<Evidence>,
}

pub trait ObjectiveStore: Send + Sync + 'static {
    fn load_all(&self) -> Result<Vec<ObjectiveSnapshot>, StoreError>;
    fn get(&self, id: &str) -> Result<Option<ObjectiveSnapshot>, StoreError>;
    fn put(&self, snapshot: &ObjectiveSnapshot) -> Result<(), StoreError>;
}

#[derive(Default)]
pub struct InMemoryObjectiveStore {
    snapshots: RwLock<BTreeMap<String, ObjectiveSnapshot>>,
}

impl ObjectiveStore for InMemoryObjectiveStore {
    fn load_all(&self) -> Result<Vec<ObjectiveSnapshot>, StoreError> {
        let snapshots = self.snapshots.read().map_err(|_| StoreError::Poisoned)?;
        Ok(snapshots.values().cloned().collect())
    }

    fn get(&self, id: &str) -> Result<Option<ObjectiveSnapshot>, StoreError> {
        validate_id(id)?;
        let snapshots = self.snapshots.read().map_err(|_| StoreError::Poisoned)?;
        Ok(snapshots.get(id).cloned())
    }

    fn put(&self, snapshot: &ObjectiveSnapshot) -> Result<(), StoreError> {
        validate_id(&snapshot.view.id)?;
        let mut snapshots = self.snapshots.write().map_err(|_| StoreError::Poisoned)?;
        snapshots.insert(snapshot.view.id.clone(), snapshot.clone());
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct FileObjectiveStore {
    root: PathBuf,
}

impl FileObjectiveStore {
    pub fn open(root: impl Into<PathBuf>) -> Result<Self, StoreError> {
        let root = root.into();
        fs::create_dir_all(&root)?;
        let store = Self { root };
        store.load_all()?;
        Ok(store)
    }

    fn path_for(&self, id: &str) -> Result<PathBuf, StoreError> {
        validate_id(id)?;
        Ok(self.root.join(format!("{id}.json")))
    }

    fn temp_path_for(&self, id: &str) -> Result<PathBuf, StoreError> {
        validate_id(id)?;
        Ok(self.root.join(format!(".{id}.json.tmp")))
    }

    fn read_path(&self, path: &Path) -> Result<ObjectiveSnapshot, StoreError> {
        let file = File::open(path)?;
        serde_json::from_reader(file).map_err(|source| StoreError::Corrupt {
            path: path.to_path_buf(),
            source,
        })
    }
}

impl ObjectiveStore for FileObjectiveStore {
    fn load_all(&self) -> Result<Vec<ObjectiveSnapshot>, StoreError> {
        let mut snapshots = Vec::new();
        for entry in fs::read_dir(&self.root)? {
            let path = entry?.path();
            if path.extension().and_then(|value| value.to_str()) != Some("json") {
                continue;
            }
            snapshots.push(self.read_path(&path)?);
        }
        snapshots.sort_by(|left, right| left.view.id.cmp(&right.view.id));
        Ok(snapshots)
    }

    fn get(&self, id: &str) -> Result<Option<ObjectiveSnapshot>, StoreError> {
        let path = self.path_for(id)?;
        if !path.exists() {
            return Ok(None);
        }
        self.read_path(&path).map(Some)
    }

    fn put(&self, snapshot: &ObjectiveSnapshot) -> Result<(), StoreError> {
        let path = self.path_for(&snapshot.view.id)?;
        let temp = self.temp_path_for(&snapshot.view.id)?;
        fs::create_dir_all(&self.root)?;

        {
            let file = File::create(&temp)?;
            let mut writer = BufWriter::new(file);
            serde_json::to_writer_pretty(&mut writer, snapshot)?;
            use std::io::Write;
            writer.flush()?;
            writer.get_ref().sync_all()?;
        }

        fs::rename(&temp, &path)?;
        File::open(&self.root)?.sync_all()?;
        Ok(())
    }
}

fn validate_id(id: &str) -> Result<(), StoreError> {
    let valid = !id.is_empty()
        && id != "."
        && id != ".."
        && id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'));
    if valid {
        Ok(())
    } else {
        Err(StoreError::InvalidId(id.to_string()))
    }
}

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("invalid objective id '{0}'")]
    InvalidId(String),
    #[error("objective store lock is poisoned")]
    Poisoned,
    #[error("objective snapshot at '{}' is corrupt: {source}", path.display())]
    Corrupt {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Serialize(#[from] serde_json::Error),
}
