use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkAdapter {
    pub name: String,
    pub ip: IpAddr,
    pub description: String,
    pub is_available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub selected_adapter: Option<String>, // adapter name
    pub window_size: Option<(f32, f32)>,
    pub auto_send_enabled: bool,
    pub send_rate: u32,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            selected_adapter: None,
            window_size: None,
            auto_send_enabled: false,
            send_rate: 20,
        }
    }
}

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
    pub network_adapters: Vec<NetworkAdapter>,
    pub selected_adapter: Option<String>,
    pub settings: AppSettings,
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
            network_adapters: Vec::new(),
            selected_adapter: None,
            settings: AppSettings::default(),
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

    pub fn load_settings(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(config_dir) =
            directories::ProjectDirs::from("com", "sacn-viewer", "sACN Viewer")
        {
            let config_path = config_dir.config_dir().join("settings.json");
            if config_path.exists() {
                let contents = std::fs::read_to_string(&config_path)?;
                let settings: AppSettings = serde_json::from_str(&contents)?;
                self.settings = settings;
                self.selected_adapter = self.settings.selected_adapter.clone();
                self.auto_send_enabled = self.settings.auto_send_enabled;
                self.send_rate = self.settings.send_rate;
                self.add_log(LogLevel::Info, "Settings loaded successfully".to_string());
            }
        }
        Ok(())
    }

    pub fn save_settings(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(config_dir) =
            directories::ProjectDirs::from("com", "sacn-viewer", "sACN Viewer")
        {
            std::fs::create_dir_all(config_dir.config_dir())?;
            let config_path = config_dir.config_dir().join("settings.json");
            let contents = serde_json::to_string_pretty(&self.settings)?;
            std::fs::write(&config_path, contents)?;
        }
        Ok(())
    }

    pub fn update_adapter_selection(&mut self, adapter_name: Option<String>) {
        self.selected_adapter = adapter_name.clone();
        self.settings.selected_adapter = adapter_name;
        if let Err(e) = self.save_settings() {
            self.add_log(LogLevel::Warning, format!("Failed to save settings: {}", e));
        }
    }

    pub fn refresh_network_adapters(&mut self) {
        match if_addrs::get_if_addrs() {
            Ok(interfaces) => {
                self.network_adapters.clear();
                for interface in interfaces {
                    if !interface.is_loopback() {
                        let adapter = NetworkAdapter {
                            name: interface.name.clone(),
                            ip: interface.ip(),
                            description: format!("{} ({})", interface.name, interface.ip()),
                            is_available: true,
                        };
                        self.network_adapters.push(adapter);
                    }
                }
                self.add_log(
                    LogLevel::Info,
                    format!("Found {} network adapters", self.network_adapters.len()),
                );
            }
            Err(e) => {
                self.add_log(
                    LogLevel::Error,
                    format!("Failed to enumerate network adapters: {}", e),
                );
            }
        }
    }

    pub fn get_selected_adapter_ip(&self) -> Option<IpAddr> {
        if let Some(ref adapter_name) = self.selected_adapter {
            self.network_adapters
                .iter()
                .find(|adapter| adapter.name == *adapter_name)
                .map(|adapter| adapter.ip)
        } else {
            // Return first available adapter if none selected
            self.network_adapters.first().map(|adapter| adapter.ip)
        }
    }
}
