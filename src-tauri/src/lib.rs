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
            println!("Connection not established");
            return Err("Connection not established".to_string());
        }
    };

    // Ensure we are receiving messages
    println!("Waiting for HEARTBEAT...");
    let mut heartbeat_received = false;
    let timeout = Duration::from_secs(5);
    let start_time = Instant::now();

    while !heartbeat_received && start_time.elapsed() < timeout {
        match connection.recv() {
            Ok((header, received_msg)) => {
                if let MavMessage::HEARTBEAT(heartbeat) = received_msg {
                    println!(
                        "Received HEARTBEAT from system {}, component {}",
                        header.system_id, header.component_id
                    );
                    heartbeat_received = true;
                } else {
                    println!(
                        "Received message from system {}, component {}: {:?}",
                        header.system_id, header.component_id, received_msg
                    );
                }
            }
            Err(e) => {
                println!("Error receiving message: {}", e);
            }
        }

        // Sleep to prevent busy-waiting
        sleep(Duration::from_millis(100));
    }

    if !heartbeat_received {
        println!("Did not receive HEARTBEAT within timeout.");
    }

    // Proceed to send the command
    if let Some(mav_command) = get_command_from_id(command_id) {
        println!(
            "Converted command_id {} to mav_command {:?}",
            command_id, mav_command
        );

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
        connection
            .send_default(&msg)
            .map_err(|e| format!("Failed to send message: {}", e))?;
        println!("Message sent successfully");

        // Wait for COMMAND_ACK
        let timeout = Duration::from_secs(15);
        let start_time = Instant::now();

        loop {
            if start_time.elapsed() >= timeout {
                return Err("No COMMAND_ACK received within timeout.".to_string());
            }

            // Receive message
            match connection.recv() {
                Ok((header, received_msg)) => {
                    println!(
                        "Received message from system {}, component {}: {:?}",
                        header.system_id, header.component_id, received_msg
                    );

                    // Check for COMMAND_ACK
                    if let MavMessage::COMMAND_ACK(command_ack) = received_msg {
                        println!("Received COMMAND_ACK: {:?}", command_ack);
                        return Ok(format!("Command acknowledged: {:?}", command_ack));
                    }

                    // Check for STATUSTEXT
                    if let MavMessage::STATUSTEXT(statustext) = received_msg {
                        println!("Received STATUSTEXT: {:?}", statustext);
                    }
                }
                Err(e) => {
                    println!("Error receiving message: {}", e);
                }
            }

            // Sleep to prevent busy-waiting
            sleep(Duration::from_millis(100));
        }
    } else {
        return Err("Invalid command_id provided.".to_string());
    }
}
