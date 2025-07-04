use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SacnDevice {
    pub ip: IpAddr,
    pub universes: Vec<u16>,
    pub last_seen: DateTime<Utc>,
    pub source_name: String,
    pub priority: u8,
}

#[derive(Debug, Clone)]
pub struct UniverseData {
    pub universe: u16,
    pub channels: [u8; 512],
    pub last_updated: DateTime<Utc>,
    pub source_ip: IpAddr,
    pub sequence: u8,
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub message: String,
}

#[derive(Debug, Clone)]
pub enum LogLevel {
    Info,
    Warning,
    Error,
    Rx,
    Tx,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warning => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
            LogLevel::Rx => write!(f, "RX"),
            LogLevel::Tx => write!(f, "TX"),
        }
    }
}

#[derive(Debug)]
pub struct AppState {
    pub devices: HashMap<IpAddr, SacnDevice>,
    pub universes: HashMap<u16, UniverseData>,
    pub logs: Vec<LogEntry>,
    pub selected_universe: Option<u16>,
    pub auto_send_enabled: bool,
    pub send_rate: u32, // packets per second
}

impl AppState {
    pub fn new() -> Self {
        Self {
            devices: HashMap::new(),
            universes: HashMap::new(),
            logs: Vec::new(),
            selected_universe: None,
            auto_send_enabled: false,
            send_rate: 20, // 20 Hz default
        }
    }

    pub fn add_log(&mut self, level: LogLevel, message: String) {
        self.logs.push(LogEntry {
            timestamp: Utc::now(),
            level,
            message,
        });

        // Keep only last 1000 log entries
        if self.logs.len() > 1000 {
            self.logs.remove(0);
        }
    }

    pub fn update_device(&mut self, ip: IpAddr, universe: u16, source_name: String, priority: u8) {
        let device = self.devices.entry(ip).or_insert_with(|| SacnDevice {
            ip,
            universes: Vec::new(),
            last_seen: Utc::now(),
            source_name: source_name.clone(),
            priority,
        });

        device.last_seen = Utc::now();
        device.source_name = source_name;
        device.priority = priority;

        if !device.universes.contains(&universe) {
            device.universes.push(universe);
            device.universes.sort();
        }
    }

    pub fn update_universe(
        &mut self,
        universe: u16,
        channels: [u8; 512],
        source_ip: IpAddr,
        sequence: u8,
    ) {
        self.universes.insert(
            universe,
            UniverseData {
                universe,
                channels,
                last_updated: Utc::now(),
                source_ip,
                sequence,
            },
        );
    }
}
