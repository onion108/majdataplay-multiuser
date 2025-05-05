use std::{
    env::{current_dir, current_exe},
    fs::{self, create_dir, exists},
    num::NonZero,
    path::PathBuf,
};

use egui::{vec2, Color32, FontDefinitions, Key, Layout, Rect, Ui};
use egui_modal::Modal;
use egui_notify::Toasts;
use subprocess::Exec;

use crate::{
    error::Result,
    font::load_system_fonts,
    layout::{discard_layout_on_need, hori_centered, vert_centered},
};

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

    exit_on_launch: bool,

    ui_zoom: f32,

    user_add_modal_state: ModalState,
    toasts: Toasts,

    ctx_initialized: bool,
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
            toasts: Toasts::default(),
            ctx_initialized: false,
            exit_on_launch: false,
            ui_zoom: 2.5,
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
    fn launch_majdata(&self, ctx: &egui::Context, test_mode: bool) -> Result<()> {
        if let Some(current_user) = &self.current_user {
            self.load_user_profile(current_user)?;
            let mut cmd =
                Exec::cmd(self.executable_dir.join("MajdataPlay.exe")).cwd(&self.executable_dir);
            if test_mode {
                cmd = cmd.arg("--test-mode");
            }
            cmd.detached().popen()?;
            if self.exit_on_launch {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            Ok(())
        } else {
            Err(crate::error::LauncherError::NoUserPresentOnLaunch)
        }
    }

    /// Initialize `egui` context.
    fn init_ctx(&mut self, ctx: &egui::Context) {
        ctx.set_pixels_per_point(self.ui_zoom);

        let fonts = load_system_fonts(FontDefinitions::default());
        ctx.set_fonts(fonts);
        ctx.options_mut(|opt| {
            opt.max_passes = NonZero::new(10).unwrap();
        });
    }
}

impl eframe::App for LauncherApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.ctx_initialized {
            self.init_ctx(ctx);
            self.ctx_initialized = true;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            vert_centered("body", ui, ctx, |ui| {
                ui.with_layout(
                    Layout::top_down(egui::Align::Center).with_main_align(egui::Align::Center),
                    |ui| {
                        ui.heading("MajdataPlay MultiUser Launcher");
                        ui.advance_cursor_after_rect(Rect::from_two_pos(
                            ui.cursor().min,
                            ui.cursor().min + vec2(0., 5.),
                        ));

                        hori_centered("combo_center", ui, ctx, |ui| {
                            egui::ComboBox::from_id_salt("combo_user")
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
                        });
                        ui.advance_cursor_after_rect(Rect::from_two_pos(
                            ui.cursor().min,
                            ui.cursor().min + vec2(0., 5.),
                        ));

                        // The button group below the ComboBox.
                        hori_centered("btn_grp", ui, ctx, |ui| {
                            let modal = Modal::new(ctx, "add_user");
                            modal.show(|ui| {
                                self.user_add_modal(ctx, &modal, ui);
                            });

                            if ui.button("Add User").clicked() {
                                modal.open();
                            }
                            if ui.button("Start MajdataPlay").clicked() {
                                if let Err(err) = self.launch_majdata(ctx, false) {
                                    self.toasts
                                        .error(format!("Failed to launch MajdataPlay: {}", err));
                                }
                            }
                            if ui.button("Start MajdataPlay In Test Mode").clicked() {
                                if let Err(err) = self.launch_majdata(ctx, true) {
                                    self.toasts
                                        .error(format!("Failed to launch MajdataPlay: {}", err));
                                }
                            }
                        });

                        hori_centered("checkbox_exit_on_launch", ui, ctx, |ui| {
                            ui.checkbox(&mut self.exit_on_launch, "Exit on launch");
                        });
                    },
                );
            });
        });

        // Update the profile if changed.
        'update_profile: for _ in 0..1 {
            // Why don't Rust have do...while loops !? That sucks.
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
        discard_layout_on_need(ctx);

        let (modifiers, delta) = ctx.input(|inp| (inp.modifiers, inp.raw_scroll_delta.y));
        if modifiers.command {
            self.ui_zoom = (self.ui_zoom + delta * 0.01).clamp(0.1, 10.0);
            ctx.set_pixels_per_point(self.ui_zoom);
        }
    }
}
