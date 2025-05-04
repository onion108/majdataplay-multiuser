use thiserror::Error;

#[derive(Debug, Error)]
pub enum LauncherError {
    #[error("EGUI Error: {0}")]
    EframeError(#[from] eframe::Error),

    #[error("IO Error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("Failed to launch process: {0}")]
    PopenError(#[from] subprocess::PopenError),

    #[error("User not found: {0}")]
    UserNotFound(String),

    #[error("No user present when trying to launch majdata")]
    NoUserPresentOnLaunch,
}

pub type Result<T> = std::result::Result<T, LauncherError>;
