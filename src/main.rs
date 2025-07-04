use anyhow::Result;
use eframe::egui;
use log::info;
use std::sync::Arc;
use tokio::sync::RwLock;

mod core;
mod network;
mod ui;

use core::AppState;
use network::SacnNetwork;
use ui::MainWindow;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    info!("Starting sACN Desktop Viewer");

    let app_state = Arc::new(RwLock::new(AppState::new()));
    let sacn_network = Arc::new(SacnNetwork::new(app_state.clone()));

    // Start the network listener in a background task
    let network_clone = sacn_network.clone();
    tokio::spawn(async move {
        if let Err(e) = network_clone.start_listener().await {
            log::error!("Network listener error: {}", e);
        }
    });

    // Run the GUI
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("sACN Desktop Viewer"),
        ..Default::default()
    };

    eframe::run_native(
        "sACN Desktop Viewer",
        options,
        Box::new(|_cc| Ok(Box::new(MainWindow::new(app_state, sacn_network)))),
    )
    .map_err(|e| anyhow::anyhow!("GUI error: {}", e))
}
