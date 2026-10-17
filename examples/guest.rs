//! A real WASI command fixture. Build with rustc --target wasm32-wasip2.
use std::io::{Read, Write};

fn main() {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).unwrap();
    match input.trim() {
        "probe" => {
            assert!(std::env::vars().next().is_none());
            assert!(std::fs::read("/toolcapsule-host-probe").is_err());
            assert!(std::fs::write("/toolcapsule-host-probe", b"no").is_err());
            assert!(std::net::TcpStream::connect("127.0.0.1:9").is_err());
            assert!(std::net::UdpSocket::bind("127.0.0.1:0").is_err());
            assert!(
                std::process::Command::new("toolcapsule-no-such-command")
                    .status()
                    .is_err()
            );
            assert_eq!(std::time::SystemTime::now(), std::time::UNIX_EPOCH);
            println!("capabilities-denied");
        }
        "sleep" => std::thread::sleep(std::time::Duration::from_secs(60)),
        "flood" => {
            let _ = std::io::stdout().write_all(&vec![b'x'; 1_000_000]);
        }
        "fail" => std::process::exit(7),
        _ => print!("{}", input.to_uppercase()),
    }
}
