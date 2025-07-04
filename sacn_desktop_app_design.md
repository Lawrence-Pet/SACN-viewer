# sACN Desktop Control Application

## Overview

This application will allow users to:

- Discover sACN (Streaming ACN / E1.31) devices on a local network.
- Monitor and visualize DMX channel output (per universe).
- Send DMX data to devices over sACN (E1.31).
- Operate seamlessly on Linux, Windows, and macOS.

## Key Features

- **Device Discovery**: Multicast listener to detect sACN devices via sACN universe traffic.
- **DMX Visualization**: Display live updates of DMX values (0-255) in a UI table/grid.
- **DMX Transmission**: Send DMX values to specific universes/devices over sACN.
- **Cross-platform UI**: One codebase for all major OSes.
- **Logging and Diagnostics**: Real-time logging of sACN packets, errors, and traffic stats.

## Technology Stack

### Programming Language

- **Rust** or **Python** or **C++** — Each has pros and cons:
  - **Rust**: High performance, modern async networking, cross-platform, safe concurrency.
  - **Python**: Rapid prototyping, rich ecosystem, good for UI + network prototyping.
  - **C++**: Most performant, low-level access, more effort for portability and UI.

For this project, **Rust** is a strong candidate due to:

- Excellent async support (via `tokio`).
- Good cross-platform tooling (`cargo`, `cross`, etc.).
- Safety in network and thread handling.

## Protocols & Standards

- **E1.31 (Streaming ACN)**: A protocol for DMX over IP.
  - Uses **Multicast UDP** over IPv4 (optionally IPv6).
  - Universe-based data packets.

References:

- ANSI E1.31-2018 standard
- [sACN Packet Structure](https://etclabs.github.io/sACN/docs/api/structsacn_1_1_data_packet.html)

## Architecture

### High-Level Diagram

```
+-----------------------------+
|   Cross-Platform UI (egui) |
+------------+----------------
             |
             v
+------------+------------+
|   Application Logic     |
|  (Device Discovery,     |
|   Data Parsing, Control)|
+------------+------------+
             |
             v
+------------+------------+
|   sACN Network Module   |
| (UDP Listener, Sender)  |
+------------+------------+
             |
             v
      +------+------+
      |   Network   |
      +-------------+
```

## Components

### 1. UI Frontend

#### Suggested Framework:

- **egui + eframe (Rust)** – Immediate-mode GUI, fast, cross-platform, native.
- **Alternatives**:
  - **Tauri + Web frontend** (React/Svelte) for hybrid GUI.
  - **Qt (C++)**, **PyQt/PySide (Python)** if you choose C++/Python.

#### Features:

- Device table (discovered devices, IPs, universes)
- Universe viewer (grid of 512 channels)
- DMX sender (sliders or input fields)
- Logging panel (network packets, errors, events)

### 2. sACN Network Module

#### Responsibilities:

- Open UDP socket on port **5568**
- Join multicast groups for each universe (239.255.X.Y)
- Parse incoming sACN packets
- Send valid DMX sACN packets

#### Libraries:

- **Rust**:
  - `tokio` for async networking
  - `socket2` for multicast socket setup
  - Optional: `sacn` crate (if maintained)
- **Python**:
  - `asyncio`, `socket`, `struct`
  - `sacn` module (https://github.com/jens-maus/python-sacn)
- **C++**:
  - Raw sockets or with Boost.Asio

### 3. DMX Visualization Engine

- Channel grid (1–512) per universe
- Color-coded levels (e.g., 0 = black, 255 = white, spectrum in between)
- Realtime updates from network
- Possibly line graph / heatmap view per channel over time

### 4. DMX Transmission Engine

- UI input to change values (slider or numeric)
- Timer-based packet sending (~44Hz max, ~20Hz typical)
- Manual send or continuous send modes

## File and Configuration Layout

```
sacn-desktop/
├── src/
│   ├── main.rs
│   ├── network/
│   │   ├── listener.rs
│   │   └── sender.rs
│   ├── ui/
│   │   └── main_window.rs
│   └── core/
│       ├── parser.rs
│       └── dmx_data.rs
├── assets/
│   └── icons/
├── Cargo.toml
└── README.md
```

## Build System

### Rust + Cargo + Cross

- Easy builds on all platforms.
- Use `cargo cross` for cross-compilation to Windows/macOS/Linux from Linux.

### Deployment

- Use `cargo bundle` (via [cargo-bundle](https://crates.io/crates/cargo-bundle)) or `tauri` for native installers.
- Add platform-specific `.desktop`, `.plist`, or `.exe` bundling as needed.

## Example UI Mockups

**Main Window**

```
+-------------------------------------------------------+
| [ Device Table ]              [ Logs / Events ]       |
| +------------------------+   +--------------------+   |
| | IP       | Universe(s) |   | [INFO] Listening...|   |
| |----------|-------------|   | [RX] Packet from...|   |
| | 10.0.0.2 | 1, 2         |   | [TX] Sent DMX...   |   |
| +------------------------+   +--------------------+   |
|                                                       |
| [ Universe View: 1 ]                                  |
| +---------------------------------------------------+ |
| | [Ch1: 255][Ch2: 100]...[Ch512: 0] (Sliders/Boxes) | |
| +---------------------------------------------------+ |
| [Send] [Auto Send]                                   |
+-------------------------------------------------------+
```

## Future Enhancements

- Support **Art-Net** alongside sACN.
- Export/import show data (per-universe states).
- Add **MIDI or OSC** trigger integration.
- sACN Sync packets and priority merging logic.
- Time-series recording of DMX data.

## References

- ANSI E1.31 - Streaming Architecture for Control Networks (sACN)
- https://etclabs.github.io/sACN/docs/
- https://github.com/ETCLabs/sACN
- https://github.com/jens-maus/python-sacn

## License

Open-source recommended license: **MIT or Apache-2.0**

## Development Roadmap

| Milestone             | Description                          | ETA     |
| --------------------- | ------------------------------------ | ------- |
| Project Bootstrap     | Setup Rust app with egui, UDP socket | 1 week  |
| Packet Parser         | Implement sACN header parsing        | 1 week  |
| Device Discovery      | Join multicast, list senders         | 2 weeks |
| DMX Viewer            | Live channel grid per universe       | 2 weeks |
| DMX Sender            | UI + network logic to send DMX       | 2 weeks |
| Cross-platform builds | Build and test on Win/macOS/Linux    | 1 week  |
| Release Alpha         | v0.1 with basic features             | -       |
