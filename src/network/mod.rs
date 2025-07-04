use crate::core::{AppState, LogLevel};
use anyhow::Result;
use byteorder::{BigEndian, ReadBytesExt};
use log::{debug, error, info, warn};
use socket2::{Domain, Protocol, Socket, Type};
use std::io::Cursor;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};

const SACN_PORT: u16 = 5568;
const ACN_IDENTIFIER: &[u8] = b"ASC-E1.17\x00\x00\x00";

pub struct SacnNetwork {
    app_state: Arc<RwLock<AppState>>,
}

impl SacnNetwork {
    pub fn new(app_state: Arc<RwLock<AppState>>) -> Self {
        Self { app_state }
    }

    pub async fn start_listener(&self) -> Result<()> {
        info!("Starting sACN network listener");

        // Create socket
        let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
        socket.set_reuse_address(true)?;
        #[cfg(unix)]
        socket.set_reuse_port(true)?;

        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), SACN_PORT);
        socket.bind(&addr.into())?;

        // Join multicast groups for universes 1-63999
        for universe in 1..=63999u16 {
            let multicast_addr = self.universe_to_multicast(universe);
            if let Err(e) = socket.join_multicast_v4(&multicast_addr, &Ipv4Addr::UNSPECIFIED) {
                warn!("Failed to join multicast for universe {}: {}", universe, e);
            }
        }

        let socket: std::net::UdpSocket = socket.into();
        socket.set_nonblocking(true)?;
        let socket = UdpSocket::from_std(socket)?;

        {
            let mut state = self.app_state.write().await;
            state.add_log(
                LogLevel::Info,
                "sACN listener started on port 5568".to_string(),
            );
        }

        // Listen for packets
        let mut buf = [0u8; 1500];
        loop {
            match socket.recv_from(&mut buf).await {
                Ok((len, addr)) => {
                    if let Err(e) = self.handle_packet(&buf[..len], addr).await {
                        debug!("Error handling packet from {}: {}", addr, e);
                    }
                }
                Err(e) => {
                    error!("UDP receive error: {}", e);
                    sleep(Duration::from_millis(100)).await;
                }
            }
        }
    }

    async fn handle_packet(&self, data: &[u8], addr: SocketAddr) -> Result<()> {
        let packet = SacnPacket::parse(data)?;

        {
            let mut state = self.app_state.write().await;
            state.add_log(
                LogLevel::Rx,
                format!(
                    "Received sACN packet from {} for universe {}",
                    addr.ip(),
                    packet.universe
                ),
            );

            state.update_device(
                addr.ip(),
                packet.universe,
                packet.source_name.clone(),
                packet.priority,
            );

            state.update_universe(packet.universe, packet.dmx_data, addr.ip(), packet.sequence);
        }

        Ok(())
    }

    fn universe_to_multicast(&self, universe: u16) -> Ipv4Addr {
        let high_byte = (universe >> 8) as u8;
        let low_byte = (universe & 0xFF) as u8;
        Ipv4Addr::new(239, 255, high_byte, low_byte)
    }

    pub async fn send_dmx(&self, universe: u16, dmx_data: &[u8; 512]) -> Result<()> {
        // Create a simple sACN packet for sending
        let packet = self.create_sacn_packet(universe, dmx_data)?;

        // Send to multicast address
        let multicast_addr = self.universe_to_multicast(universe);
        let dest = SocketAddr::new(IpAddr::V4(multicast_addr), SACN_PORT);

        // Create a new socket for sending
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        socket.send_to(&packet, dest).await?;

        {
            let mut state = self.app_state.write().await;
            state.add_log(
                LogLevel::Tx,
                format!("Sent DMX data to universe {}", universe),
            );
        }

        Ok(())
    }

    fn create_sacn_packet(&self, universe: u16, dmx_data: &[u8; 512]) -> Result<Vec<u8>> {
        let mut packet = Vec::new();

        // Root Layer
        packet.extend_from_slice(&[0x00, 0x10]); // Preamble Size
        packet.extend_from_slice(&[0x00, 0x00]); // Post-amble Size
        packet.extend_from_slice(ACN_IDENTIFIER); // ACN Packet Identifier
        packet.extend_from_slice(&[0x70, 0x26]); // Flags and Length (638 bytes)
        packet.extend_from_slice(&[0x00, 0x00, 0x00, 0x04]); // Vector
        packet.extend_from_slice(uuid::Uuid::new_v4().as_bytes()); // CID

        // Framing Layer
        packet.extend_from_slice(&[0x72, 0x58]); // Flags and Length (600 bytes)
        packet.extend_from_slice(&[0x00, 0x00, 0x00, 0x02]); // Vector
        packet.extend_from_slice(b"sACN Viewer\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00"); // Source Name (64 bytes)
        packet.push(100); // Priority
        packet.extend_from_slice(&[0x00, 0x00]); // Sync Address
        packet.push(0); // Sequence Number
        packet.push(0); // Options
        packet.extend_from_slice(&universe.to_be_bytes()); // Universe

        // DMP Layer
        packet.extend_from_slice(&[0x72, 0x0B]); // Flags and Length (523 bytes)
        packet.push(0x02); // Vector
        packet.push(0xA1); // Address Type & Data Type
        packet.extend_from_slice(&[0x00, 0x00]); // First Property Address
        packet.extend_from_slice(&[0x00, 0x01]); // Address Increment
        packet.extend_from_slice(&[0x02, 0x01]); // Property value count (513)
        packet.push(0x00); // DMX512 START Code
        packet.extend_from_slice(dmx_data); // DMX Data

        Ok(packet)
    }
}

#[derive(Debug, Clone)]
pub struct SacnPacket {
    pub universe: u16,
    pub sequence: u8,
    pub priority: u8,
    pub source_name: String,
    pub dmx_data: [u8; 512],
}

impl SacnPacket {
    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < 126 {
            return Err(anyhow::anyhow!("Packet too short"));
        }

        let mut cursor = Cursor::new(data);

        // Skip preamble and postamble
        cursor.set_position(4);

        // Check ACN identifier
        let mut identifier = [0u8; 12];
        std::io::Read::read_exact(&mut cursor, &mut identifier)?;
        if identifier != ACN_IDENTIFIER {
            return Err(anyhow::anyhow!("Invalid ACN identifier"));
        }

        // Skip to framing layer
        cursor.set_position(38);

        // Read source name
        let mut source_name_bytes = [0u8; 64];
        std::io::Read::read_exact(&mut cursor, &mut source_name_bytes)?;
        let source_name = String::from_utf8_lossy(&source_name_bytes)
            .trim_end_matches('\0')
            .to_string();

        // Read priority
        let priority = cursor.read_u8()?;

        // Skip sync address
        cursor.set_position(cursor.position() + 2);

        // Read sequence
        let sequence = cursor.read_u8()?;

        // Skip options
        cursor.set_position(cursor.position() + 1);

        // Read universe
        let universe = cursor.read_u16::<BigEndian>()?;

        // Skip DMP layer header
        cursor.set_position(cursor.position() + 10);

        // Skip start code
        cursor.set_position(cursor.position() + 1);

        // Read DMX data
        let mut dmx_data = [0u8; 512];
        std::io::Read::read_exact(&mut cursor, &mut dmx_data)?;

        Ok(SacnPacket {
            universe,
            sequence,
            priority,
            source_name,
            dmx_data,
        })
    }
}
