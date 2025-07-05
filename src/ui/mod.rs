use crate::core::{AppState, LogLevel};
use crate::network::SacnNetwork;
use eframe::egui;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct MainWindow {
    app_state: Arc<RwLock<AppState>>,
    network: Arc<SacnNetwork>,
    dmx_send_values: [u8; 512],
    send_universe: u16,
    show_hex: bool,
}

impl MainWindow {
    pub fn new(app_state: Arc<RwLock<AppState>>, network: Arc<SacnNetwork>) -> Self {
        Self {
            app_state,
            network,
            dmx_send_values: [0; 512],
            send_universe: 1,
            show_hex: false,
        }
    }
}

impl eframe::App for MainWindow {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Request repaint for live updates
        ctx.request_repaint();

        // Top panel with controls
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("sACN Desktop Viewer");
                ui.separator();

                // Network adapter selection
                if let Ok(mut state) = self.app_state.try_write() {
                    ui.label("Network Adapter:");
                    let selected_text = state.selected_adapter.as_deref().unwrap_or("Auto");

                    egui::ComboBox::from_id_source("adapter_combo")
                        .selected_text(selected_text)
                        .show_ui(ui, |ui| {
                            if ui
                                .selectable_value(&mut state.selected_adapter, None, "Auto")
                                .clicked()
                            {
                                state.update_adapter_selection(None);
                            }

                            for adapter in &state.network_adapters.clone() {
                                if ui
                                    .selectable_value(
                                        &mut state.selected_adapter,
                                        Some(adapter.name.clone()),
                                        &adapter.description,
                                    )
                                    .clicked()
                                {
                                    state.update_adapter_selection(Some(adapter.name.clone()));
                                }
                            }
                        });

                    if ui.button("Refresh").clicked() {
                        state.refresh_network_adapters();
                    }
                }

                ui.separator();
                ui.label("Universe:");
                ui.add(egui::DragValue::new(&mut self.send_universe).range(1..=63999));
                ui.separator();
                ui.checkbox(&mut self.show_hex, "Show Hex");
            });
        });

        // Left panel for devices
        egui::SidePanel::left("left_panel")
            .default_width(300.0)
            .show(ctx, |ui| {
                ui.heading("Network Status");

                if let Ok(state) = self.app_state.try_read() {
                    ui.group(|ui| {
                        ui.label("Selected Adapter:");
                        if let Some(ref adapter_name) = state.selected_adapter {
                            if let Some(adapter) = state
                                .network_adapters
                                .iter()
                                .find(|a| a.name == *adapter_name)
                            {
                                ui.label(format!("• {} ({})", adapter.name, adapter.ip));
                            } else {
                                ui.colored_label(egui::Color32::RED, "• Adapter not found");
                            }
                        } else {
                            ui.label("• Auto-select");
                        }

                        ui.separator();
                        ui.label("Available Adapters:");
                        for adapter in &state.network_adapters {
                            let color = if adapter.is_available {
                                egui::Color32::GREEN
                            } else {
                                egui::Color32::RED
                            };
                            ui.colored_label(color, format!("• {}", adapter.description));
                        }
                    });
                }

                ui.separator();
                ui.heading("Discovered Devices");

                if let Ok(state) = self.app_state.try_read() {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for (ip, device) in &state.devices {
                            ui.group(|ui| {
                                ui.label(format!("IP: {}", ip));
                                ui.label(format!("Source: {}", device.source_name));
                                ui.label(format!("Priority: {}", device.priority));
                                ui.label(format!("Universes: {:?}", device.universes));
                                ui.label(format!(
                                    "Last seen: {}",
                                    device.last_seen.format("%H:%M:%S")
                                ));
                            });
                        }
                    });
                }
            });

        // Right panel for logs
        egui::SidePanel::right("right_panel")
            .default_width(300.0)
            .show(ctx, |ui| {
                ui.heading("Logs");

                if let Ok(state) = self.app_state.try_read() {
                    egui::ScrollArea::vertical()
                        .stick_to_bottom(true)
                        .show(ui, |ui| {
                            for log in state.logs.iter().rev().take(100) {
                                let color = match log.level {
                                    LogLevel::Info => egui::Color32::WHITE,
                                    LogLevel::Warning => egui::Color32::YELLOW,
                                    LogLevel::Error => egui::Color32::RED,
                                    LogLevel::Rx => egui::Color32::GREEN,
                                    LogLevel::Tx => egui::Color32::BLUE,
                                };

                                ui.horizontal(|ui| {
                                    ui.colored_label(color, format!("[{}]", log.level));
                                    ui.label(format!(
                                        "{}: {}",
                                        log.timestamp.format("%H:%M:%S"),
                                        log.message
                                    ));
                                });
                            }
                        });
                }
            });

        // Central panel for universe view
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Universe View");
                ui.separator();

                if let Ok(mut state) = self.app_state.try_write() {
                    egui::ComboBox::from_label("Select Universe")
                        .selected_text(
                            state
                                .selected_universe
                                .map_or("None".to_string(), |u| u.to_string()),
                        )
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut state.selected_universe, None, "None");

                            let mut universes: Vec<u16> = state.universes.keys().cloned().collect();
                            universes.sort();

                            for universe in universes {
                                ui.selectable_value(
                                    &mut state.selected_universe,
                                    Some(universe),
                                    universe.to_string(),
                                );
                            }
                        });
                }
            });

            ui.separator();

            if let Ok(state) = self.app_state.try_read() {
                if let Some(selected_universe) = state.selected_universe {
                    if let Some(universe_data) = state.universes.get(&selected_universe) {
                        ui.label(format!(
                            "Universe {} - Source: {} - Last Updated: {}",
                            universe_data.universe,
                            universe_data.source_ip,
                            universe_data.last_updated.format("%H:%M:%S%.3f")
                        ));

                        // DMX channel grid
                        egui::ScrollArea::both().show(ui, |ui| {
                            egui::Grid::new("dmx_grid")
                                .num_columns(16)
                                .striped(true)
                                .show(ui, |ui| {
                                    for (i, &value) in universe_data.channels.iter().enumerate() {
                                        let channel = i + 1;

                                        let color = if value == 0 {
                                            egui::Color32::BLACK
                                        } else {
                                            let intensity = value as f32 / 255.0;
                                            egui::Color32::from_gray((intensity * 255.0) as u8)
                                        };

                                        let text = if self.show_hex {
                                            format!("{:02X}", value)
                                        } else {
                                            format!("{}", value)
                                        };

                                        ui.colored_label(color, format!("{}:{}", channel, text));

                                        if i % 16 == 15 {
                                            ui.end_row();
                                        }
                                    }
                                });
                        });
                    }
                }
            }

            ui.separator();

            // DMX Sender section
            ui.heading("DMX Sender");

            ui.horizontal(|ui| {
                ui.label("Send to Universe:");
                ui.add(egui::DragValue::new(&mut self.send_universe).range(1..=63999));

                if ui.button("Send DMX").clicked() {
                    let network = self.network.clone();
                    let universe = self.send_universe;
                    let dmx_data = self.dmx_send_values;

                    tokio::spawn(async move {
                        if let Err(e) = network.send_dmx(universe, &dmx_data).await {
                            log::error!("Failed to send DMX: {}", e);
                        }
                    });
                }
            });

            // Simple channel controls (first 16 channels)
            ui.label("Channel Controls (1-16):");
            egui::Grid::new("channel_controls")
                .num_columns(4)
                .show(ui, |ui| {
                    for i in 0..16 {
                        ui.vertical(|ui| {
                            ui.label(format!("Ch {}", i + 1));
                            ui.add(
                                egui::Slider::new(&mut self.dmx_send_values[i], 0..=255)
                                    .orientation(egui::SliderOrientation::Vertical),
                            );
                        });

                        if i % 4 == 3 {
                            ui.end_row();
                        }
                    }
                });
        });
    }
}
