# Network Adapter Selection Implementation

## Overview

Added comprehensive network adapter selection functionality to the sACN viewer with persistence across application restarts.

## Features Added

### 1. Network Adapter Discovery

- **Automatic Detection**: Discovers all available network interfaces on startup
- **Interface Filtering**: Excludes loopback interfaces, shows only real network adapters
- **Real-time Status**: Shows adapter availability and IP addresses
- **Refresh Capability**: Manual refresh button to re-scan network adapters

### 2. Adapter Selection

- **GUI Selection**: Dropdown combo box in the top toolbar for adapter selection
- **Auto-select Mode**: Falls back to system default if no adapter is selected
- **Persistent Selection**: Remembers selected adapter across application restarts
- **Visual Feedback**: Shows currently selected adapter in the network status panel

### 3. Network Binding

- **Listener Binding**: sACN listener binds to the selected network adapter
- **Multicast Joining**: Joins multicast groups on the specific adapter interface
- **Send Binding**: DMX transmission uses the selected adapter for outgoing packets
- **Fallback Handling**: Gracefully handles adapter unavailability

### 4. Settings Persistence

- **JSON Configuration**: Saves settings to user's config directory
- **Cross-platform**: Uses platform-appropriate config directories
- **Automatic Saving**: Settings are saved immediately when changed
- **Error Handling**: Graceful handling of settings load/save failures

## Technical Implementation

### Dependencies Added

```toml
if-addrs = "0.13"      # Network interface enumeration
directories = "5.0"     # Cross-platform directory paths
```

### Core Data Structures

```rust
pub struct NetworkAdapter {
    pub name: String,           // Interface name (e.g., "eth0", "wlan0")
    pub ip: IpAddr,            // Interface IP address
    pub description: String,    // Human-readable description
    pub is_available: bool,     // Current availability status
}

pub struct AppSettings {
    pub selected_adapter: Option<String>,  // Saved adapter name
    pub window_size: Option<(f32, f32)>,   // Future: window size
    pub auto_send_enabled: bool,           // Future: auto-send mode
    pub send_rate: u32,                    // Future: send rate
}
```

### Key Methods

- `refresh_network_adapters()`: Scans for available network interfaces
- `update_adapter_selection()`: Updates and persists adapter selection
- `get_selected_adapter_ip()`: Returns IP of selected adapter
- `load_settings()` / `save_settings()`: Persistent configuration

## User Interface Changes

### Top Toolbar

- Added network adapter selection dropdown
- Added refresh button for re-scanning adapters
- Shows current selection with "Auto" as default

### Left Panel - Network Status

- **Selected Adapter**: Shows currently active adapter
- **Available Adapters**: Lists all discovered adapters with status
- **Color Coding**: Green for available, red for unavailable adapters

## Configuration Storage

### Location

- **macOS**: `~/Library/Application Support/com.sacn-viewer.sACN Viewer/settings.json`
- **Linux**: `~/.config/sacn-viewer/settings.json`
- **Windows**: `%APPDATA%\sacn-viewer\sACN Viewer\settings.json`

### Format

```json
{
  "selected_adapter": "en0",
  "window_size": [1200.0, 800.0],
  "auto_send_enabled": false,
  "send_rate": 20
}
```

## Network Behavior

### Multicast Handling

- Joins sACN multicast groups (239.255.x.y) on selected interface
- Handles interface-specific multicast membership
- Logs multicast join failures for debugging

### Packet Transmission

- Binds outgoing sockets to selected adapter
- Ensures packets are sent from correct interface
- Maintains consistency between listening and sending

## Error Handling

- Graceful fallback to system default if selected adapter unavailable
- Logs adapter selection changes and errors
- Continues operation even if settings cannot be saved
- User-friendly error messages in the application log

## Usage Instructions

1. **Select Adapter**: Use the dropdown in the top toolbar
2. **Auto Mode**: Leave as "Auto" to use system default
3. **Refresh**: Click "Refresh" if network configuration changes
4. **Status Check**: Monitor the Network Status panel for adapter availability
5. **Persistence**: Selection is automatically saved and restored

## Benefits

- **Multi-homed Systems**: Proper operation on systems with multiple network interfaces
- **WiFi/Ethernet**: Easy switching between wireless and wired connections
- **Network Isolation**: Ensures sACN traffic uses the intended network segment
- **User Control**: Full control over network interface selection
- **Reliability**: Persistent settings survive application restarts

This implementation provides professional-grade network adapter control suitable for lighting systems that require precise network interface management.
