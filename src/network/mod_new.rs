use crate::core::{AppState, LogLevel};
use anyhow::Result;
use log::{debug, error, info, warn};
use sacn::receive::SacnReceiver;
use sacn::source::SacnSource;
use sacn::packet::ACN_SDT_MULTICAST_PORT;
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
        Self { 
            app_state,
        }
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
                    state.add_log(
                        LogLevel::Info,
                        format!("sACN receiver created on {}", addr),
                    );
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
                format!("sACN listener started on {} port {}", bind_ip, ACN_SDT_MULTICAST_PORT),
            );
        }

        // Main receive loop
        loop {
            let timeout = Some(Duration::from_millis(100));
            match receiver.recv(timeout) {
                Ok(data_packets) => {
                    for packet in data_packets {
                        if let Err(e) = self.handle_data_packet(packet).await {
                            debug!("Error handling data packet: {}", e);
                        }
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
                        }
                        sacn::error::errors::ErrorKind::Timeout => {
                            // Normal timeout, continue receiving
                            continue;
                        }
                        _ => {
                            debug!("sACN receive error: {:?}", e);
                            sleep(Duration::from_millis(100)).await;
                        }
                    }
                }
            }
        }
    }

    async fn handle_data_packet(&self, packet: sacn::packet::DataPacket) -> Result<()> {
        let universe = packet.universe();
        let source_name = packet.source_name().to_string();
        let priority = packet.priority();
        let data = packet.data();
        let sequence = packet.sequence_number();

        // Convert the source CID to an IP (we'll use a placeholder for now)
        // In a real implementation, you might need to track source IPs separately
        let source_ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)); // Placeholder

        // Ensure we have 512 channels (skip start code if present)
        let mut channels = [0u8; 512];
        let dmx_data = if data.len() > 0 && data[0] == 0 {
            // Skip start code
            &data[1..]
        } else {
            data
        };
        let data_len = std::cmp::min(dmx_data.len(), 512);
        channels[..data_len].copy_from_slice(&dmx_data[..data_len]);

        {
            let mut state = self.app_state.write().await;
            
            state.add_log(
                LogLevel::Rx,
                format!("Received sACN data from {} for universe {}", source_name, universe),
            );
            
            state.update_device(
                source_ip,
                universe,
                source_name,
                priority,
            );
            
            state.update_universe(
                universe,
                channels,
                source_ip,
                sequence,
            );
        }

        Ok(())
    }

    pub async fn send_dmx(&self, universe: u16, dmx_data: &[u8; 512]) -> Result<()> {
        // Get the selected adapter IP for binding
        let bind_ip = {
            let state = self.app_state.read().await;
            state
                .get_selected_adapter_ip()
                .unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED))
        };

        let bind_addr = SocketAddr::new(bind_ip, ACN_SDT_MULTICAST_PORT + 1);
        
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
                    format!("Sent DMX data to universe {}", universe),
                );
                Ok(())
            }
            Err(e) => {
                let mut state = self.app_state.write().await;
                state.add_log(
                    LogLevel::Error,
                    format!("Failed to send DMX data: {}", e),
                );
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
