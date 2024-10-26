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

use mavlink::{
    common::{MavMessage, COMMAND_LONG_DATA},
    connect,
};

#[tauri::command]
async fn create_connection(connection_string: String) -> Result<String, String> {
    match connect::<MavMessage>(&connection_string) {
        Ok(_) => Ok("Connected successfully".to_string()),
        Err(e) => Err(format!("Connection failed: {}", e)),
    }
}

#[tauri::command]
async fn send_mavlink_message(
    target_system: u8,
    target_component: u8,
    _message_id: u16,
) -> Result<String, String> {
    // Connect to the MAVLink device
    let connection_str = "tcpout:refly-015.local:5760";
    let connection = match connect(connection_str) {
        Ok(conn) => conn,
        Err(e) => return Err(format!("Connection error: {}", e)),
    };

    // Populate the COMMAND_LONG_DATA struct with parameters
    let msg_data = COMMAND_LONG_DATA {
        target_system,
        target_component,
        command: mavlink::common::MavCmd::MAV_CMD_ACTUATOR_TEST,
        confirmation: 0,
        param1: 0.0,
        param2: 0.0,
        param3: 0.0,
        param4: 0.0,
        param5: 0.0,
        param6: 0.0,
        param7: 0.0,
    };

    // Create the MAVLink message using the COMMAND_LONG variant
    let msg = MavMessage::COMMAND_LONG(msg_data);

    // Send the message
    if let Err(e) = connection.send_default(&msg) {
        print!("Failed to send message: {}", e);
        return Err(format!("Failed to send message: {}", e));
    }

    Ok("MAVLink message sent successfully".to_string())
}
