#[macro_use]
extern crate colour;

use serde_json::Value;
use std::io::{self};
use std::ops::Drop;
use ximu3::connection::*;
use ximu3::port_scanner::*;

pub struct Device {
    connection: Connection,
}

impl Device {
    pub fn new() -> Device {
        blue_ln!("Please connect USB");

        loop {
            for device in PortScanner::scan_filter(PortType::Usb) {
                if matches!(device.device_name.as_str(), "x-IMU3 BON" | "x-IMU3 Thermometer") {
                    continue;
                }

                let connection = Connection::new(&device.connection_info);

                if connection.open().is_ok() {
                    println!("Connected to {}", device);

                    return Device { connection };
                }
            }
        }
    }

    pub fn send_command(&self, key: &str, value: Option<&str>) -> Result<String, ()> {
        let command = format!("{{\"{}\":{}}}", key, value.unwrap_or("null"));

        let response = self.connection.send_commands(vec![command.as_str()], 0, 15000);

        if response.len() == 0 {
            red_ln!("No response to {}", command);
            return Err(());
        }

        Ok(response[0].clone())
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        blue_ln!("Please disconnect USB");

        let port_name = self.connection.get_info().to_string().replace("USB ", "");

        self.connection.close();

        while PortScanner::get_port_names().contains(&port_name) {
            std::thread::sleep(std::time::Duration::from_millis(1000));
        }
    }
}

fn main() {
    println!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

    // Get hardware version
    blue_ln!("Please enter hardware version");

    let version = &mut String::new();

    io::stdin().read_line(version).ok();

    let version = version.trim();

    // Repeat for each device
    loop {
        let _ = (|| {
            let device = Device::new();

            // Write hardware version
            if version.is_empty() == false {
                device.send_command("factory", None)?;
                device.send_command("hardware_version", Some(format!("\"{}\"", version).as_str()))?;
                device.send_command("apply", None)?;
                device.send_command("save", None)?;
            }

            // Send self-test command
            let response = device.send_command("test", None)?;

            // Parse self-test response
            let response: Value = serde_json::from_str(&response).unwrap();

            if let Some(object) = response["test"].as_object() {
                for (key, value) in object {
                    white!("{:<width$}", key, width = 32);

                    if let Some(value) = value.as_str() {
                        if value == "Passed" {
                            green_ln!("{}", value);
                        } else {
                            red_ln!("{}", value);
                        }
                    } else {
                        red_ln!("Unable to parse self-test response");
                        break;
                    }
                }
            } else {
                red_ln!("Unable to parse self-test response");
            }

            Ok::<(), ()>(())
        })();
    }
}
