use eframe::egui;
use regex::Regex;
use std::sync::mpsc::Receiver;
use std::thread;
use std::time::Duration;

struct SerialApp {
    serial_rx: Receiver<String>,
    log: String,
    file_log: String,
    marker: String,
}

impl SerialApp {
    fn new(serial_rx: Receiver<String>) -> Self {
        Self {
            serial_rx,
            log: String::new(),
            file_log: String::new(),
            marker: String::new(),
        }
    }

    fn process_file_content(&mut self, content: String) {
        let cleaned_data = remove_ansi_escape_codes(&content);
        self.file_log.push_str(&cleaned_data);
    }
    
}

fn highlight_text(ui: &mut egui::Ui, text: &str, marker: &str) {
    if marker.is_empty() {
        ui.label(text); // If the marker is empty, just display the text
        return;
    }
    println!("Marker: {}", marker);
    let mut start = 0;
    while let Some(pos) = text[start..].find(marker) {
        let end = start + pos;
        ui.label(&text[start..end]);
        ui.label(
            egui::RichText::new(marker)
                .background_color(egui::Color32::YELLOW)
                .monospace(),
        );
        start = end + marker.len();
    }
    if start < text.len() {
        ui.label(&text[start..]);
    }
}




impl eframe::App for SerialApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        while let Ok(data) = self.serial_rx.try_recv() {
            let cleaned_data = remove_ansi_escape_codes(&data);
            self.log.push_str(&cleaned_data);
        }

    // Handle file dropsif !ctx.input().raw.dropped_files.is_empty() {
        for file in &ctx.input(|i| i.raw.dropped_files.clone()) {
            if let Some(path) = &file.path {
                if let Ok(content) = std::fs::read_to_string(path) {
                    self.process_file_content(content);
                }
            }
        }
    


        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Clear").clicked() {
                    self.log.clear();
                    self.file_log.clear();
                }
                ui.label("Mark:");
                ui.text_edit_singleline(&mut self.marker); // Text box for marker input

            });

            ui.separator(); // A separator line between the button and the log

            egui::ScrollArea::vertical().show(ui, |ui| {
                // Highlight and display serial log
                highlight_text(ui, &self.log, &self.marker);
                let mut start = 0;
                if start < self.log.len() {
                    ui.label(&self.log[start..]); // Remaining text
                }
                ui.separator(); // A separator line between serial log and file log

                highlight_text(ui, &self.file_log, &self.marker);

                start = 0;
                // while let Some(pos) = self.file_log[start..].find("DEBUG") {
                //     let end = start + pos;
                //     ui.label(&self.file_log[start..end]);
                //     ui.label(
                //         egui::RichText::new("DEBUG")
                //             .background_color(egui::Color32::YELLOW)
                //             .monospace(),
                //     );
                //     start = end + "DEBUG".len();
                // }
                if start < self.file_log.len() {
                    ui.label(&self.file_log[start..]);
                }

            });
        });
        ctx.request_repaint(); // Request to repaint the UI for the next frame
    }
}




fn main() -> Result<(), Box<dyn std::error::Error>> {
    let serial_rx = start_serial_thread("COM3"); // Use COM3 as the serial port

    let app = SerialApp::new(serial_rx);

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Serial Port GUI",
        native_options,
        Box::new(|_cc| Ok(Box::new(app))),
    )?;

    Ok(())
}

fn start_serial_thread(port_name: &str) -> Receiver<String> {
    use serialport::SerialPort;
    use std::sync::mpsc::{self, Receiver};
    use std::io::Read;

    let (tx, rx) = mpsc::channel();

    let port_name = port_name.to_string();
    thread::spawn(move || {
        let mut port = serialport::new(port_name, 115200)  // Set baud rate to 115200
            .timeout(Duration::from_millis(10)) // Short timeout for non-blocking reads
            .open()
            .expect("Failed to open port");

        loop {
            let mut buf: Vec<u8> = vec![0; 32];
            match port.read(&mut buf) {
                Ok(bytes_read) => {
                    if bytes_read > 0 {
                        let data = String::from_utf8_lossy(&buf[..bytes_read]).to_string();
                        tx.send(data).expect("Failed to send data");
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => (),
                Err(e) => eprintln!("Error: {:?}", e),
            }
            thread::sleep(Duration::from_millis(1)); // Small sleep to prevent busy-waiting
        }
    });

    rx
}

fn remove_ansi_escape_codes(text: &str) -> String {
    // Regex to match ANSI escape codes
    let ansi_escape_regex = Regex::new(r"\x1b\[[0-?]*[ -/]*[@-~]").unwrap();
    ansi_escape_regex.replace_all(text, "").to_string()
}
