#[macro_use]
extern crate colour;

use serde_json::Value;
use ximu3::connection::*;
use ximu3::connection_type::*;
use ximu3::port_scanner::*;

pub struct Device {
    connection: Connection,
}

impl Device {
    pub fn new() -> Device {
        blue_ln!("Please connect USB");

        loop {
            for device in PortScanner::scan_filter(ConnectionType::Usb) {
                if device.device_name.contains("BON") {
                    continue;
                }

                let connection = Connection::new(&device.connection_info);

                if connection.open().is_ok() {
                    prnt_ln!("Connected to {}", connection.get_info());

                    return Device { connection };
                }
            }
        }
    }

    pub fn send_command(&self, command: &str) -> Result<String, ()> {
        prnt_ln!("Sending command {}", command);

        let response = self.connection.send_commands(vec![command], 0, 5000);

        if response.len() == 0 {
            return Err(());
        }

        return Ok(response[0].clone());
    }

    pub fn disconnect(&self) {
        blue_ln!("Please disconnect USB");

        let port_name = self.connection.get_info().to_string().replace("USB - ", "");

        while PortScanner::get_port_names().contains(&port_name) {
            std::thread::sleep(std::time::Duration::from_millis(1000));
        }
    }
}

fn main() {
    prnt_ln!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

    loop {
        let device = Device::new();

        match device.send_command("{\"test\":null}") {
            Ok(response) => {
                let response: Value = serde_json::from_str(&response).unwrap();

                if let Some(object) = response["test"].as_object() {
                    for (key, value) in object {
                        white!("{}", format!("{:<width$}", key, width = 32));

                        if let Some(value) = value.as_str() {
                            const PASSED: &str = "Passed";

                            if value == PASSED {
                                green_ln!("{}", PASSED);
                            } else {
                                red_ln!("{}", value);
                            }
                        } else {
                            red_ln!("Invalid response to self-test command");
                        }
                    }
                } else {
                    red_ln!("Invalid response to self-test command");
                }
            }
            Err(_) => red_ln!("No response to self-test command"),
        }

        device.disconnect();
    }
}
