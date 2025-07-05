// Test sACN sender using the sacn crate
// This is a working test sender that uses the same sacn crate as the main application

use sacn::packet::ACN_SDT_MULTICAST_PORT;
use sacn::source::SacnSource;
use std::net::{IpAddr, SocketAddr};
use std::thread::sleep;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting sACN test sender using sacn crate...");
    println!("This test sender uses the same sacn crate as the main application");
    println!("for better compatibility and standards compliance.");
    println!("");

    // Create a local address for the sender
    let local_addr = SocketAddr::new(IpAddr::V4([0, 0, 0, 0].into()), ACN_SDT_MULTICAST_PORT + 1);

    println!("Creating sACN source on {}", local_addr);
    let mut src = SacnSource::with_ip("Test Source", local_addr)?;

    let universe = 1u16;
    println!("Registering universe {}", universe);
    src.register_universe(universe)?;

    println!("Sending test data...");

    // Send some test data
    for i in 0..10 {
        // Create test data - start code (0) + some DMX channels
        let mut data = vec![0u8]; // DMX start code

        // Add some animated test data
        for channel in 0..16 {
            let value = ((i * 10 + channel) % 256) as u8;
            data.push(value);
        }

        // Pad with zeros to make it look more realistic
        while data.len() < 100 {
            data.push(0);
        }

        println!("Sending packet {} with {} bytes", i + 1, data.len());
        src.send(&[universe], &data, Some(100), None, None)?;

        sleep(Duration::from_millis(500));
    }

    println!("Test complete!");
    println!("");
    println!("The main sACN viewer application now uses the professional sacn crate");
    println!("which provides:");
    println!("- Standards-compliant sACN implementation (ANSI E1.31-2018)");
    println!("- Automatic universe discovery");
    println!("- Source discovery and tracking");
    println!("- Synchronization support");
    println!("- Robust error handling");
    println!("- Cross-platform compatibility");

    Ok(())
}
