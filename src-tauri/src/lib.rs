use lazy_static::lazy_static;
use mavlink::{
    common::{MavCmd, MavMessage, COMMAND_LONG_DATA},
    connect, MavConnection,
};
use num_traits::FromPrimitive;
use std::{
    sync::{Arc, Mutex},
    thread::sleep,
    time::{Duration, Instant},
};

use tauri::Emitter;

// The global connection
lazy_static! {
    static ref GLOBAL_CONNECTION: Arc<Mutex<Option<Box<dyn MavConnection<MavMessage> + Send + Sync>>>> =
        Arc::new(Mutex::new(None));
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![my_custom_command])
        .invoke_handler(tauri::generate_handler![
            send_mavlink_message,
            create_connection,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn my_custom_command() -> String {
    "I was invoked from JavaScript!".to_string()
}

// Establish connection and start logging messages
#[tauri::command]
async fn create_connection(connection_string: String) -> Result<String, String> {
    match connect::<MavMessage>(&connection_string) {
        Ok(conn) => {
            let mut global_conn = GLOBAL_CONNECTION.lock().unwrap();
            *global_conn = Some(conn);
            println!("Connected to {}", connection_string);
            Ok("Connected successfully".to_string())
        }
        Err(e) => Err(format!("Connection failed: {:?}", e)),
    }
}

fn get_command_from_id(command_id: u16) -> Option<MavCmd> {
    println!("Received command_id: {}", command_id);
    let mav_cmd = FromPrimitive::from_u16(command_id); // Converts u16 to MavCmd if it matches

    match mav_cmd {
        Some(cmd) => {
            println!("Matched MavCmd: {:?}", cmd);
            Some(cmd)
        }
        None => {
            println!(
                "Invalid command_id: {} - No match found in MavCmd",
                command_id
            );
            None
        }
    }
}

#[tauri::command]
async fn send_mavlink_message(
    window: tauri::Window,
    target_system: u8,
    target_component: u8,
    command_id: u16,
    param1: f32,
    param2: f32,
    param3: f32,
    param4: f32,
    param5: f32,
    param6: f32,
    param7: f32,
) -> Result<String, String> {
    // Get the global connection
    let mut global_conn = GLOBAL_CONNECTION.lock().unwrap();

    // Check if the connection is initialized
    let connection = match &mut *global_conn {
        Some(conn) => conn.as_mut(),
        None => {
            window.emit("log", "Connection not established").unwrap();
            return Err("Connection not established".to_string());
        }
    };

    // Ensure we are receiving messages
    window.emit("log", "Waiting for HEARTBEAT...").unwrap();

    let mut heartbeat_received = false;
    let timeout = Duration::from_secs(5);
    let start_time = Instant::now();

    while !heartbeat_received && start_time.elapsed() < timeout {
        match connection.recv() {
            Ok((header, received_msg)) => {
                if let MavMessage::HEARTBEAT(_heartbeat) = received_msg {
                    window
                        .emit(
                            "log",
                            format!(
                                "Received HEARTBEAT from system {}, component {}",
                                header.system_id, header.component_id
                            ),
                        )
                        .unwrap();
                    heartbeat_received = true;
                }
            }
            Err(e) => {
                window
                    .emit("log", format!("Error receiving message: {}", e))
                    .unwrap();
            }
        }
    }

    if !heartbeat_received {
        window
            .emit("log", "Did not receive HEARTBEAT within timeout.")
            .unwrap();
    }

    // Proceed to send the command
    if let Some(mav_command) = get_command_from_id(command_id) {
        window
            .emit(
                "log",
                format!(
                    "Converted command_id {} to mav_command {:?}",
                    command_id, mav_command
                ),
            )
            .unwrap();

        window
            .emit(
                "log",
                format!("This is the target component {:?}", target_component),
            )
            .unwrap();
        window
            .emit(
                "log",
                format!("This is the target system {:?}", target_system),
            )
            .unwrap();
        window
            .emit("log", format!("This is mav command {:?}", mav_command))
            .unwrap();

        let msg_data = COMMAND_LONG_DATA {
            target_system,
            target_component,
            command: mav_command,
            confirmation: 0,
            param1,
            param2,
            param3,
            param4,
            param5,
            param6,
            param7,
        };

        let msg = MavMessage::COMMAND_LONG(msg_data);

        // Send command
        connection.send_default(&msg).map_err(|e| {
            let error_msg = format!("Failed to send message: {}", e);
            window.emit("log", &error_msg).unwrap();
            error_msg
        })?;
        window.emit("log", "Message sent successfully").unwrap();

        // Wait for COMMAND_ACK
        let timeout = Duration::from_secs(15);
        let start_time = Instant::now();

        loop {
            if start_time.elapsed() >= timeout {
                return Err("No COMMAND_ACK received within timeout.".to_string());
            }

            // Receive message
            match connection.recv() {
                Ok((_header, received_msg)) => {
                    // Check for COMMAND_ACK
                    if let MavMessage::COMMAND_ACK(command_ack) = received_msg {
                        window
                            .emit("log", format!("Received COMMAND_ACK: {:?}", command_ack))
                            .unwrap();
                        return Ok(format!("Command acknowledged: {:?}", command_ack));
                    }

                    // Check for STATUSTEXT
                    if let MavMessage::STATUSTEXT(statustext) = received_msg {
                        window
                            .emit("log", format!("Received STATUSTEXT: {:?}", statustext))
                            .unwrap();
                    }
                }
                Err(e) => {
                    window
                        .emit("log", format!("Error receiving message: {}", e))
                        .unwrap();
                }
            }

            // Sleep to prevent busy-waiting
            sleep(Duration::from_millis(100));
        }
    } else {
        window.emit("log", "Invalid command_id provided.").unwrap();
        return Err("Invalid command_id provided.".to_string());
    }
}
