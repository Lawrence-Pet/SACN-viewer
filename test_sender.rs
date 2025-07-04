use std::net::UdpSocket;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting sACN test sender...");

    let socket = UdpSocket::bind("0.0.0.0:0")?;

    // Create a simple sACN packet for universe 1
    let packet = create_test_packet(1, 255, 128)?;

    // Send to multicast address for universe 1
    let dest = "239.255.0.1:5568";

    loop {
        socket.send_to(&packet, dest)?;
        println!("Sent test packet to universe 1");
        thread::sleep(Duration::from_millis(500));
    }
}

fn create_test_packet(
    universe: u16,
    ch1: u8,
    ch2: u8,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut packet = Vec::new();

    // Root Layer
    packet.extend_from_slice(&[0x00, 0x10]); // Preamble Size
    packet.extend_from_slice(&[0x00, 0x00]); // Post-amble Size
    packet.extend_from_slice(b"ASC-E1.17\x00\x00\x00"); // ACN Packet Identifier
    packet.extend_from_slice(&[0x70, 0x26]); // Flags and Length (638 bytes)
    packet.extend_from_slice(&[0x00, 0x00, 0x00, 0x04]); // Vector

    // Generate a simple CID (16 bytes)
    let cid = [
        0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde,
        0xf0,
    ];
    packet.extend_from_slice(&cid);

    // Framing Layer
    packet.extend_from_slice(&[0x72, 0x58]); // Flags and Length (600 bytes)
    packet.extend_from_slice(&[0x00, 0x00, 0x00, 0x02]); // Vector

    // Source Name (64 bytes)
    let mut source_name = [0u8; 64];
    let name = b"Test sACN Sender";
    source_name[..name.len()].copy_from_slice(name);
    packet.extend_from_slice(&source_name);

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

    // DMX Data (512 bytes)
    let mut dmx_data = [0u8; 512];
    dmx_data[0] = ch1; // Channel 1
    dmx_data[1] = ch2; // Channel 2
    packet.extend_from_slice(&dmx_data);

    Ok(packet)
}
