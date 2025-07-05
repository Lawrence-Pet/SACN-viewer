# sACN Desktop Viewer

A cross-platform desktop application for monitoring and sending sACN (Streaming ACN / E1.31) DMX data over IP networks.

## Features

- **Device Discovery**: Automatically discovers sACN devices on the network
- **Live DMX Monitoring**: Real-time visualization of DMX channel values (0-255)
- **Universe Selection**: View any discovered sACN universe
- **DMX Transmission**: Send DMX data to specific universes
- **Network Adapter Selection**: Choose which network interface to use
- **Logging**: Real-time logging of network activity
- **Cross-platform**: Runs on Windows, macOS, and Linux

## Technology Stack

- **Language**: Rust
- **GUI Framework**: egui + eframe
- **Networking**: Official `sacn` crate (v0.10) - Standards-compliant sACN implementation
- **Protocol**: sACN (E1.31) - Streaming ACN over IP (ANSI E1.31-2018)

## Building

### Prerequisites

- Rust 1.82 or later
- Cargo

### Build Instructions

```bash
# Clone the repository
git clone <repository-url>
cd SACN-viewer

# Build the project
cargo build --release

# Run the application
cargo run
```

## Usage

### Starting the Application

```bash
cargo run
```

The application will:

1. Start listening for sACN packets on port 5568
2. Join multicast groups for all sACN universes (1-512)
3. Display a GUI with device discovery, universe viewing, and DMX sending capabilities

### Interface Overview

- **Left Panel**: Shows discovered sACN devices with their IP addresses, source names, and active universes
- **Central Panel**:
  - Network adapter selection dropdown
  - Universe selector dropdown
  - DMX channel grid showing live values (0-255)
  - DMX sender with channel controls
- **Right Panel**: Live logs showing network activity

### Network Adapter Selection

The application allows you to select which network interface to use:

1. The dropdown in the central panel shows all available network adapters
2. Select your preferred adapter (e.g., WiFi, Ethernet)
3. The selection is automatically saved and will be restored on next startup

### Sending DMX Data

1. Set the target universe using the "Send to Universe" field
2. Adjust channel values using the sliders (channels 1-16 are shown)
3. Click "Send DMX" to transmit the data

### Testing

You can test the application using the included test sender:

```bash
# Run the test sender
cargo run --bin test_sender
```

This will send test sACN packets to universe 1 with some sample DMX data.

## sACN Protocol Details

The application uses the official `sacn` crate which implements the sACN (E1.31) protocol:

- **Port**: 5568 (UDP)
- **Multicast Base**: 239.255.x.y (where x.y represents the universe number)
- **Standards Compliance**: ANSI E1.31-2018
- **Universe Range**: 1-63999
- **Channels per Universe**: 512
- **Features**: Universe discovery, source discovery, synchronization support

## Network Requirements

- The application requires multicast networking support
- Firewall may need to allow UDP traffic on port 5568
- Network switches should support IGMP for multicast traffic

## Architecture

```
┌─────────────────────────────┐
│   Cross-Platform UI (egui)  │
├─────────────────────────────┤
│   Application Logic         │
│  (Device Discovery,         │
│   Data Parsing, Control)    │
├─────────────────────────────┤
│   sACN Network Module       │
│ (UDP Listener, Sender)      │
├─────────────────────────────┤
│   Network (Multicast UDP)   │
└─────────────────────────────┘
```

## License

This project is licensed under the MIT License.

## Contributing

Contributions are welcome! Please feel free to submit issues, feature requests, or pull requests.

## Future Enhancements

- [ ] Art-Net protocol support
- [ ] Show data export/import
- [ ] MIDI/OSC trigger integration
- [ ] Time-series recording
- [ ] Priority merging
- [ ] Sync packet support
- [ ] Performance optimizations
- [ ] Advanced filtering options
      Streaming ACN viewer just for fun
