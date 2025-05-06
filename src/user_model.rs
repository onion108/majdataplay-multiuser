use std::{fs, fs::exists, path::PathBuf};

use crate::error::{LauncherError, Result};

pub struct UserManager {
    profile_dir: PathBuf,
    global_dir: PathBuf,

    /// All users loaded
    pub users: Vec<String>,
}

impl UserManager {
    pub fn new(profile: impl Into<PathBuf>, global: impl Into<PathBuf>) -> Self {
        Self {
            profile_dir: profile.into(),
            global_dir: global.into(),
            users: Vec::new(),
        }
    }

    /// Load user list form profile path.
    pub fn load_user_list(&mut self) -> Result<()> {
        self.users.clear();
        if exists(&self.profile_dir).unwrap_or(false) {
            for i in fs::read_dir(&self.profile_dir)?.filter_map(|x| x.ok()) {
                let Ok(metadata) = i.metadata() else { continue };
                if metadata.is_dir() {
                    self.users.push(i.file_name().to_string_lossy().to_string());
                }
            }
        } else {
            fs::create_dir(&self.profile_dir)?;
        }
        Ok(())
    }

    /// Check if given user exists.
    pub fn user_exists(&self, name: &str) -> bool {
        exists(self.profile_dir.join(name)).unwrap_or(false)
    }

    /// Create a new user.
    pub fn create_user(&mut self, name: &str) -> Result<()> {
        if !self.user_exists(name) {
            fs::create_dir(self.profile_dir.join(name))?;
        }
        Ok(())
    }

    /// Sync give file from user directory to global directory.
    pub fn sync_file_user2global(&self, name: &str, filename: &str) -> Result<()> {
        if name.is_empty() || !self.user_exists(name) {
            return Err(LauncherError::UserNotFound(name.into()));
        }

        let local_path = self.profile_dir.join(name).join(filename);
        let global_path = self.global_dir.join(filename);
        if exists(&local_path).unwrap_or(false) {
            fs::copy(&local_path, &global_path)?;
        } else {
            // Delete if the file isn't present in the user directory.
            if exists(&global_path).unwrap_or(false) {
                fs::remove_file(&global_path)?;
            }
        }
        Ok(())
    }

    pub fn sync_file_global2user(&self, name: &str, filename: &str) -> Result<()> {
        if name.is_empty() || !self.user_exists(name) {
            return Err(LauncherError::UserNotFound(name.into()));
        }

        let local_path = self.profile_dir.join(name).join(filename);
        let global_path = self.global_dir.join(filename);
        if exists(&global_path).unwrap_or(false) {
            fs::copy(&global_path, &local_path)?;
        } else {
            // Delete if the file isn't present in the global directory
            if exists(&local_path).unwrap_or(false) {
                fs::remove_file(&local_path)?;
            }
        }
        Ok(())
    }

    /// Sync global game data with given user's data.
    pub fn sync_global(&self, name: &str) -> Result<()> {
        self.sync_file_user2global(name, "settings.json")
            .and_then(|_| self.sync_file_user2global(name, "MajDatabase.db.db.db.db.db.db.db.db.db.db.db.db.db.db.db.db.db.db.db.db.db.db.db.db.db.db.db.db"))
    }

    /// Sync given user's game data with global's data.
    pub fn sync_user(&self, name: &str) -> Result<()> {
        self.sync_file_global2user(name, "settings.json")
            .and_then(|_| self.sync_file_global2user(name, "MajDatabase.db.db.db.db.db.db.db.db.db.db.db.db.db.db.db.db.db.db.db.db.db.db.db.db.db.db.db.db"))
    }
}
