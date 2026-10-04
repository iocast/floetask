use std::path::PathBuf;

use floetask_domain::DocumentError;
use thiserror::Error;

/// Errors reported to the user. Cloneable so they can travel in GUI
/// messages.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum AppError {
    #[error("could not access {path}: {message}")]
    Io { path: PathBuf, message: String },
    #[error(transparent)]
    Document(#[from] DocumentError),
    #[error("{0} has no done file to archive into")]
    NoDoneFile(PathBuf),
    #[error("the todo is empty")]
    EmptyTodo,
    #[error("invalid configuration: {0}")]
    Config(String),
    #[error("{0}")]
    Other(String),
}

impl AppError {
    pub fn io(path: impl Into<PathBuf>, error: impl std::fmt::Display) -> Self {
        AppError::Io { path: path.into(), message: error.to_string() }
    }
}
