use std::{
    error::Error,
    ffi::OsStr,
    fmt::{Display, Formatter},
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use forge_core::{TaskGraph, VerifiedOrchestratorState};
use serde::{Deserialize, Serialize};

use crate::ObjectiveView;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObjectiveSnapshot {
    pub view: ObjectiveView,
    pub graph: TaskGraph,
    pub orchestrator: VerifiedOrchestratorState,
}

pub trait ObjectiveStore: Send + Sync {
    fn load_all(&self) -> Result<Vec<ObjectiveSnapshot>, StoreError>;

    fn get(&self, id: &str) -> Result<Option<ObjectiveSnapshot>, StoreError>;

    fn put(&self, snapshot: &ObjectiveSnapshot) -> Result<(), StoreError>;
}

#[derive(Debug)]
pub enum StoreError {
    Io(std::io::Error),
    Json(serde_json::Error),
    InvalidObjectiveId,
}

impl Display for StoreError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "objective store I/O error: {error}"),
            Self::Json(error) => write!(formatter, "objective store JSON error: {error}"),
            Self::InvalidObjectiveId => formatter.write_str("invalid objective id"),
        }
    }
}

impl Error for StoreError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Json(error) => Some(error),
            Self::InvalidObjectiveId => None,
        }
    }
}

impl From<std::io::Error> for StoreError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for StoreError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

#[derive(Debug, Clone)]
pub struct FileObjectiveStore {
    root: PathBuf,
}

impl FileObjectiveStore {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    fn final_path(&self, id: &str) -> Result<PathBuf, StoreError> {
        validate_objective_id(id)?;
        Ok(self.root.join(format!("{id}.json")))
    }

    fn temp_path(&self, id: &str) -> Result<PathBuf, StoreError> {
        validate_objective_id(id)?;
        Ok(self.root.join(format!("{id}.json.tmp")))
    }

    fn read_snapshot(path: &Path) -> Result<ObjectiveSnapshot, StoreError> {
        let bytes = fs::read(path)?;
        Ok(serde_json::from_slice(&bytes)?)
    }
}

impl ObjectiveStore for FileObjectiveStore {
    fn load_all(&self) -> Result<Vec<ObjectiveSnapshot>, StoreError> {
        if !self.root.exists() {
            return Ok(Vec::new());
        }

        let mut paths = fs::read_dir(&self.root)?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| path.is_file() && path.extension() == Some(OsStr::new("json")))
            .collect::<Vec<_>>();
        paths.sort();

        paths.iter().map(|path| Self::read_snapshot(path)).collect()
    }

    fn get(&self, id: &str) -> Result<Option<ObjectiveSnapshot>, StoreError> {
        let path = self.final_path(id)?;
        if !path.exists() {
            return Ok(None);
        }
        Self::read_snapshot(&path).map(Some)
    }

    fn put(&self, snapshot: &ObjectiveSnapshot) -> Result<(), StoreError> {
        fs::create_dir_all(&self.root)?;
        let final_path = self.final_path(&snapshot.view.id)?;
        let temp_path = self.temp_path(&snapshot.view.id)?;
        let bytes = serde_json::to_vec_pretty(snapshot)?;

        let mut file = File::create(&temp_path)?;
        file.write_all(&bytes)?;
        file.sync_all()?;
        drop(file);
        fs::rename(temp_path, final_path)?;
        Ok(())
    }
}

fn validate_objective_id(id: &str) -> Result<(), StoreError> {
    if id.is_empty()
        || !id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        return Err(StoreError::InvalidObjectiveId);
    }
    Ok(())
}
