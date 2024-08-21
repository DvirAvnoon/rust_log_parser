use serialport::SerialPort;
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::Duration;

pub fn start_serial_thread(port_name: &str) -> Receiver<String> {
    let (tx, rx) = mpsc::channel();

    let port_name = port_name.to_string();
    thread::spawn(move || {
        let mut port = serialport::new(port_name, 9600)
            .timeout(Duration::from_millis(10))
            .open()
            .expect("Failed to open port");

        loop {
            let mut buf: Vec<u8> = vec![0; 32];
            match port.read(&mut buf) {
                Ok(bytes_read) => {
                    let data = String::from_utf8_lossy(&buf[..bytes_read]).to_string();
                    tx.send(data).expect("Failed to send data");
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => (),
                Err(e) => eprintln!("Error: {:?}", e),
            }
        }
    });

    rx
}
