use crate::core::{AppState, LogLevel};
use anyhow::Result;
use log::{debug, info};
use sacn::packet::ACN_SDT_MULTICAST_PORT;
use sacn::receive::{DMXData, SacnReceiver};
use sacn::source::SacnSource;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::sleep;

pub struct SacnNetwork {
    app_state: Arc<RwLock<AppState>>,
}

impl SacnNetwork {
    pub fn new(app_state: Arc<RwLock<AppState>>) -> Self {
        Self { app_state }
    }

    pub async fn start_listener(&self) -> Result<()> {
        info!("Starting sACN network listener");

        // Get the selected adapter IP
        let bind_ip = {
            let state = self.app_state.read().await;
            state
                .get_selected_adapter_ip()
                .unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED))
        };

        let addr = SocketAddr::new(bind_ip, ACN_SDT_MULTICAST_PORT);

        // Create the sACN receiver with universe discovery enabled
        let mut receiver = match SacnReceiver::with_ip(addr, None) {
            Ok(receiver) => {
                {
                    let mut state = self.app_state.write().await;
                    state.add_log(LogLevel::Info, format!("sACN receiver created on {}", addr));
                }
                receiver
            }
            Err(e) => {
                {
                    let mut state = self.app_state.write().await;
                    state.add_log(
                        LogLevel::Error,
                        format!("Failed to create sACN receiver: {}", e),
                    );
                }
                return Err(anyhow::anyhow!("Failed to create sACN receiver: {}", e));
            }
        };

        // Enable source discovery announcements
        receiver.set_announce_source_discovery(true);

        // Listen for all universes (we'll register them as they're discovered)
        // Start with a common set of universes
        let common_universes: Vec<u16> = (1..=512).collect();
        if let Err(e) = receiver.listen_universes(&common_universes) {
            let mut state = self.app_state.write().await;
            state.add_log(
                LogLevel::Warning,
                format!("Failed to register some universes: {}", e),
            );
        }

        {
            let mut state = self.app_state.write().await;
            state.add_log(
                LogLevel::Info,
                format!(
                    "sACN listener started on {} port {}",
                    bind_ip, ACN_SDT_MULTICAST_PORT
                ),
            );
        }

        // Main receive loop
        loop {
            let timeout = Some(Duration::from_millis(100));
            match receiver.recv(timeout) {
                Ok(packets) => {
                    // Process received packets
                    for packet in packets {
                        self.handle_packet(packet).await;
                    }
                }
                Err(e) => {
                    match e.kind() {
                        sacn::error::errors::ErrorKind::SourceDiscovered(source_name) => {
                            let mut state = self.app_state.write().await;
                            state.add_log(
                                LogLevel::Info,
                                format!("Source discovered: {}", source_name),
                            );
                            state.update_device(ip, universe, source_name, priority);
                        }
                        _ => {
                            // Handle other errors including timeouts
                            debug!("sACN receive error: {:?}", e);
                            sleep(Duration::from_millis(100)).await;
                        }
                    }
                }
            }
        }
    }

    async fn handle_packet(&self, packet: DMXData) {
        let mut state = self.app_state.write().await;

        // Log the received packet
        state.add_log(
            LogLevel::Rx,
            format!(
                "Received DMX data on universe {}: {} channels",
                packet.universe,
                packet.values.len()
            ),
        );

        // Convert Vec<u8> to [u8; 512], padding with zeros if needed
        let mut channels = [0u8; 512];
        let copy_len = std::cmp::min(packet.values.len(), 512);
        if copy_len > 0 {
            channels[..copy_len].copy_from_slice(&packet.values[..copy_len]);
        }

        // Update universe data
        state.update_universe(
            packet.universe,
            channels,
            // We don't have direct access to source IP from DMXData,
            // so we'll use a placeholder for now
            IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            0, // sequence not available in DMXData
        );

        // Update device information if we have source CID
        if let Some(src_cid) = packet.src_cid {
            let source_name = format!("Source-{}", src_cid);
            state.update_device(
                IpAddr::V4(Ipv4Addr::UNSPECIFIED), // placeholder
                packet.universe,
                source_name,
                packet.priority,
            );
        }
    }

    pub async fn send_dmx(&self, universe: u16, dmx_data: &[u8; 512]) -> Result<()> {
        // Get the selected adapter IP for binding
        let bind_ip = {
            let state = self.app_state.read().await;
            state
                .get_selected_adapter_ip()
                .unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED))
        };

        // Use a different port for sending to avoid conflicts
        let bind_addr = SocketAddr::new(bind_ip, 0); // Let the OS choose a port

        // Create a new source for sending
        let mut source = match SacnSource::with_ip("sACN Viewer", bind_addr) {
            Ok(source) => source,
            Err(e) => {
                let mut state = self.app_state.write().await;
                state.add_log(
                    LogLevel::Error,
                    format!("Failed to create sACN source: {}", e),
                );
                return Err(anyhow::anyhow!("Failed to create sACN source: {}", e));
            }
        };

        // Register the universe
        if let Err(e) = source.register_universe(universe) {
            let mut state = self.app_state.write().await;
            state.add_log(
                LogLevel::Error,
                format!("Failed to register universe {}: {}", universe, e),
            );
            return Err(anyhow::anyhow!("Failed to register universe: {}", e));
        }

        // Convert dmx_data to Vec<u8> with start code
        let mut data = vec![0u8]; // DMX start code
        data.extend_from_slice(dmx_data);

        // Send the data
        let priority = Some(100u8);
        let dst_ip = None; // Use multicast
        let sync_uni = None; // No synchronization

        match source.send(&[universe], &data, priority, dst_ip, sync_uni) {
            Ok(_) => {
                let mut state = self.app_state.write().await;
                state.add_log(
                    LogLevel::Tx,
                    format!(
                        "Sent DMX data to universe {}: {} channels",
                        universe,
                        dmx_data.len()
                    ),
                );
                Ok(())
            }
            Err(e) => {
                let mut state = self.app_state.write().await;
                state.add_log(LogLevel::Error, format!("Failed to send DMX data: {}", e));
                Err(anyhow::anyhow!("Failed to send DMX data: {}", e))
            }
        }
    }

    pub async fn get_discovered_sources(&self) -> Vec<String> {
        // This would need to be implemented to track discovered sources
        // For now, return an empty list
        Vec::new()
    }
}
