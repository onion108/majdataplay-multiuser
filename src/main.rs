#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::anyhow;
use eframe::App;
use egui::{ViewportBuilder, vec2};
use majdataplay_multiuser::{app::LauncherApp, error::Result};

fn main() -> Result<()> {
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default().with_inner_size(vec2(400., 250.)),
        ..Default::default()
    };
    eframe::run_native(
        "MajdataPlay MultiUser Launcher",
        options,
        Box::new(|_| {
            LauncherApp::new()
                .map(|x| Box::new(x) as Box<dyn App>)
                .map_err(|x| anyhow!("Initialization failed: {}", x.to_string()).into())
        }),
    )
    .map_err(|err| err.into())
}
