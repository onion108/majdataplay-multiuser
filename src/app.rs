use std::{
    env::{current_dir, current_exe},
    fs::{self, create_dir, exists},
    path::PathBuf,
};

use egui::{Color32, Id, Layout, Rect, Ui, vec2};
use egui_modal::Modal;
use egui_notify::Toasts;
use subprocess::Exec;

use crate::error::Result;

/// Data needed for multipass layout
struct LayoutInfo {
    combo_width: Option<f32>,
    total_height: Option<f32>,
    bgrp_width: Option<f32>,
}

/// Data needed for modal window that adds user
struct ModalState {
    username: String,
    error_desc: Option<String>,
}

impl Default for ModalState {
    fn default() -> Self {
        Self {
            username: "".into(),
            error_desc: None,
        }
    }
}

/// All data needed by the application
pub struct LauncherApp {
    
    /// The directory contains the executable
    executable_dir: PathBuf,

    /// All users loaded
    users: Vec<String>,

    /// Current selected user
    current_user: Option<String>,

    /// Previously selected user
    last_selected_user: Option<String>,

    user_add_modal_state: ModalState,
    layout_info: LayoutInfo,
    toasts: Toasts,
}

impl LauncherApp {
    /// Create a new application instance.
    pub fn new() -> Result<Self> {
        let mut executable_dir = current_exe().or_else(|_| current_dir())?;
        executable_dir.pop();

        let mut result = Self {
            executable_dir,
            users: Vec::new(),
            current_user: None,
            last_selected_user: None,
            user_add_modal_state: ModalState::default(),
            layout_info: LayoutInfo {
                combo_width: None,
                total_height: None,
                bgrp_width: None,
            },
            toasts: Toasts::default(),
        };

        result.load_user_list()?;

        Ok(result)
    }

    /// Loads user list from `UserProfile/`. Every subdirectory under the directory is considered
    /// an user profile.
    fn load_user_list(&mut self) -> Result<()> {
        self.users.clear();
        let user_profile_path = self.executable_dir.join("UserProfile");
        if exists(&user_profile_path).unwrap_or(false) {
            for i in fs::read_dir(&user_profile_path)? {
                let i = i?;
                if let Ok(metadata) = i.metadata() {
                    if metadata.is_dir() {
                        self.users.push(i.file_name().to_string_lossy().to_string());
                    }
                }
            }
        } else {
            create_dir(self.executable_dir.join("UserProfile"))?;
        }

        if !self.users.is_empty() {
            self.current_user = Some(self.users[0].clone());
        }

        Ok(())
    }

    /// Check if user exists.
    fn user_exists(&self, name: &str) -> bool {
        let user_profile_path = self.executable_dir.join("UserProfile");
        exists(user_profile_path.join(name)).unwrap_or(false)
    }

    /// Create a new user.
    fn create_user(&mut self, name: &str) -> Result<()> {
        let user_profile_path = self.executable_dir.join("UserProfile");
        if !exists(user_profile_path.join(name)).unwrap_or(false) {
            create_dir(user_profile_path.join(name))?;
            self.users.push(name.to_owned());
        }
        Ok(())
    }

    /// Load given file from user directory to global.
    fn load_user_file(&self, name: &str, filename: &str) -> Result<()> {
        if name.is_empty() || !self.user_exists(name) {
            return Err(crate::error::LauncherError::UserNotFound(name.into()));
        }

        let profile_path = self.executable_dir.join("UserProfile").join(name);
        if exists(profile_path.join(filename)).unwrap_or(false) {
            fs::copy(
                profile_path.join(filename),
                self.executable_dir.join(filename),
            )?;
        } else {
            if exists(self.executable_dir.join(filename)).unwrap_or(false) {
                fs::remove_file(self.executable_dir.join(filename))?;
            }
        }
        Ok(())
    }

    /// Save given file form global to user directory.
    fn save_user_file(&self, name: &str, filename: &str) -> Result<()> {
        if name.is_empty() || !self.user_exists(name) {
            return Err(crate::error::LauncherError::UserNotFound(name.into()));
        }

        let profile_path = self.executable_dir.join("UserProfile").join(name);
        if exists(self.executable_dir.join(filename)).unwrap_or(false) {
            fs::copy(
                self.executable_dir.join(filename),
                profile_path.join(filename),
            )?;
        } else {
            if exists(profile_path.join(filename)).unwrap_or(false) {
                fs::remove_file(profile_path.join(filename))?;
            }
        }
        Ok(())
    }

    /// Load user data to global.
    fn load_user_profile(&self, name: &str) -> Result<()> {
        self.load_user_file(name, "settings.json")
            .and_then(|_| self.load_user_file(name, "MajDatabase.db.db.db.db.db.db.db.db.db.db.db.db.db.db.db.db.db.db.db.db.db.db.db.db.db.db.db.db"))
    }

    /// Save user data to global.
    fn save_user_profile(&self, name: &str) -> Result<()> {
        self.save_user_file(name, "settings.json")
            .and_then(|_| self.load_user_file(name, "MajDatabase.db.db.db.db.db.db.db.db.db.db.db.db.db.db.db.db.db.db.db.db.db.db.db.db.db.db.db.db"))
    }

    /// The UI of adding user.
    fn user_add_modal(&mut self, _ctx: &egui::Context, modal: &Modal, ui: &mut Ui) {
        modal.title(ui, "Add user");
        ui.vertical(|ui| {
            ui.text_edit_singleline(&mut self.user_add_modal_state.username);
            if let Some(desc) = &self.user_add_modal_state.error_desc {
                ui.colored_label(Color32::RED, format!("* {}", desc));
            }
            ui.horizontal(|ui| {
                if ui.button("Cancel").clicked() {
                    modal.close();
                }
                if ui.button("Add").clicked() {
                    let username = self.user_add_modal_state.username.clone();
                    if username.is_empty() {
                        self.user_add_modal_state.error_desc =
                            Some("Please input user name.".into());
                        return;
                    }

                    if self.user_exists(&username) {
                        self.user_add_modal_state.error_desc =
                            Some(format!("User {} exists", username));
                    } else {
                        if let Err(err) = self.create_user(&username) {
                            self.user_add_modal_state.error_desc =
                                Some(format!("Failed to create user {} due to {}", username, err));
                        } else {
                            modal.close();
                        }
                    }
                }
            });
        });
    }

    /// Launch MajdataPlay.
    fn launch_majdata(&self) -> Result<()> {
        if let Some(current_user) = &self.current_user {
            self.load_user_profile(current_user)?;
            Exec::cmd(self.executable_dir.join("MajdataPlay.exe"))
                .cwd(&self.executable_dir)
                .detached()
                .popen()?;
            Ok(())
        } else {
            Err(crate::error::LauncherError::NoUserPresentOnLaunch)
        }
    }
}

impl eframe::App for LauncherApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_pixels_per_point(3.);

        // Indicate of discarding current render frame (for layout calculation).
        let mut discard_this = false;

        egui::CentralPanel::default().show(ctx, |ui| {
            // Calculate layout to center entire UI vertically.
            ui.advance_cursor_after_rect(egui::Rect::from_two_pos(
                ui.cursor().min,
                egui::pos2(
                    ui.cursor().min.x,
                    (ui.available_height() - self.layout_info.total_height.unwrap_or(0.)) / 2.,
                ),
            ));
            let begin = ui.cursor().min.y;

            ui.with_layout(
                Layout::top_down(egui::Align::Center).with_main_align(egui::Align::Center),
                |ui| {
                    ui.heading("MajdataPlay MultiUser Launcher");
                    ui.advance_cursor_after_rect(Rect::from_two_pos(
                        ui.cursor().min,
                        ui.cursor().min + vec2(0., 5.),
                    ));

                    let ir = egui::Area::new(Id::new("what"))
                        .current_pos(
                            ui.next_widget_position()
                                - vec2(self.layout_info.combo_width.unwrap_or(0.) / 2., 0.),
                        )
                        .show(ctx, |ui| {
                            let ir = egui::ComboBox::from_id_salt(0)
                                .selected_text(format!(
                                    "{}",
                                    if let Some(user) = &self.current_user {
                                        user
                                    } else {
                                        "--No User--"
                                    }
                                ))
                                .show_ui(ui, |ui| {
                                    for user in &self.users {
                                        ui.selectable_value(
                                            &mut self.current_user,
                                            Some(user.clone()),
                                            user,
                                        );
                                    }
                                });
                            if let Some(width) = self.layout_info.combo_width {
                                if width != ir.response.rect.width() {
                                    self.layout_info.combo_width = Some(ir.response.rect.width());
                                    discard_this = true;
                                }
                            } else {
                                self.layout_info.combo_width = Some(ir.response.rect.width());
                                discard_this = true;
                            }
                        });
                    ui.advance_cursor_after_rect(ir.response.rect);

                    // The button group below the ComboBox.
                    ui.horizontal(|ui| {
                        // Also some calculation to center entire button group.
                        ui.advance_cursor_after_rect(Rect::from_two_pos(
                            ui.cursor().min,
                            ui.cursor().min
                                + vec2(
                                    (ui.available_width()
                                        - self.layout_info.bgrp_width.unwrap_or(0.))
                                        / 2.,
                                    0.,
                                ),
                        ));
                        let begin = ui.cursor().min.x;

                        let modal = Modal::new(ctx, "add_user");
                        modal.show(|ui| {
                            self.user_add_modal(ctx, &modal, ui);
                        });

                        if ui.button("Add User").clicked() {
                            modal.open();
                        }
                        if ui.button("Start MajdataPlay").clicked() {
                            if let Err(err) = self.launch_majdata() {
                                self.toasts
                                    .error(format!("Failed to launch MajdataPlay: {}", err));
                            }
                        }

                        let end = ui.cursor().min.x;
                        if let Some(last_width) = self.layout_info.bgrp_width {
                            if last_width != end - begin {
                                self.layout_info.bgrp_width = Some(end - begin);
                                discard_this = true;
                            }
                        } else {
                            self.layout_info.bgrp_width = Some(end - begin);
                            discard_this = true;
                        }
                    });
                },
            );
            let end = ui.cursor().min.y;

            if let Some(height) = self.layout_info.total_height {
                if height != end - begin {
                    self.layout_info.total_height = Some(end - begin);
                    discard_this = true;
                }
            } else {
                self.layout_info.total_height = Some(end - begin);
                discard_this = true;
            }
        });

        if discard_this {
            ctx.request_discard("Recalculate layout");
        }

        // Update the profile if changed.
        'update_profile: for _ in 0..1 { // Why don't Rust have do...while loops !? That sucks.
            if self.last_selected_user != self.current_user {
                if let Some(last_user) = &self.last_selected_user {
                    if let Err(err) = self.save_user_profile(&last_user) {
                        self.toasts
                            .error(format!("Failed to save user profile: {}", err));
                        self.current_user = self.last_selected_user.clone();
                        break 'update_profile;
                    }
                }

                if let Some(current_user) = &self.current_user {
                    if let Err(err) = self.load_user_profile(&current_user) {
                        self.toasts
                            .error(format!("Failed to load user profile: {}", err));
                        self.current_user = self.last_selected_user.clone();
                        break 'update_profile;
                    }
                }

                self.last_selected_user = self.current_user.clone();
            }
        }

        self.toasts.show(ctx);
    }
}
