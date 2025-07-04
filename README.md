# sACN Desktop Viewer

A cross-platform desktop application for monitoring and sending sACN (Streaming ACN / E1.31) DMX data over IP networks.

## Features

- **Device Discovery**: Automatically discovers sACN devices on the network
- **Live DMX Monitoring**: Real-time visualization of DMX channel values (0-255)
- **Universe Selection**: View any discovered sACN universe
- **DMX Transmission**: Send DMX data to specific universes
- **Logging**: Real-time logging of network activity
- **Cross-platform**: Runs on Windows, macOS, and Linux

## Technology Stack

- **Language**: Rust
- **GUI Framework**: egui + eframe
- **Networking**: tokio with UDP multicast support
- **Protocol**: sACN (E1.31) - Streaming ACN over IP

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
2. Join multicast groups for all sACN universes (1-63999)
3. Display a GUI with device discovery, universe viewing, and DMX sending capabilities

### Interface Overview

- **Left Panel**: Shows discovered sACN devices with their IP addresses, source names, and active universes
- **Central Panel**:
  - Universe selector dropdown
  - DMX channel grid showing live values (0-255)
  - DMX sender with channel controls
- **Right Panel**: Live logs showing network activity

### Sending DMX Data

1. Set the target universe using the "Send to Universe" field
2. Adjust channel values using the sliders (channels 1-16 are shown)
3. Click "Send DMX" to transmit the data

### Testing

You can test the application using the included test sender:

```bash
# Compile and run the test sender
rustc test_sender.rs -o test_sender
./test_sender
```

This will send test sACN packets to universe 1 with some sample DMX data.

## sACN Protocol Details

The application implements the sACN (E1.31) protocol for DMX over IP:

- **Port**: 5568 (UDP)
- **Multicast Base**: 239.255.x.y (where x.y represents the universe number)
- **Packet Structure**: ACN Root Layer → Framing Layer → DMP Layer → DMX Data
- **Universe Range**: 1-63999
- **Channels per Universe**: 512

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
