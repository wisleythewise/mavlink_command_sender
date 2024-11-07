use lazy_static::lazy_static;
use mavlink::{
    common::{MavCmd, MavMessage, COMMAND_LONG_DATA},
    connect, MavConnection,
};
use num_traits::FromPrimitive;
use std::sync::Arc;
use tauri::Emitter;
use tokio::{
    sync::{mpsc, Mutex},
    time::{timeout, Duration},
};

lazy_static! {
    static ref GLOBAL_CONNECTION: Arc<Mutex<Option<Box<dyn MavConnection<MavMessage> + Send + Sync>>>> =
        Arc::new(Mutex::new(None));
    static ref CHANNEL_TX: Arc<Mutex<Option<mpsc::Sender<MavMessage>>>> =
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
        .invoke_handler(tauri::generate_handler![
            send_mavlink_message,
            create_connection,
            backend_rev,
            my_custom_command
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn my_custom_command() -> String {
    "I was invoked from JavaScript!".to_string()
}

// Backend setup function to initialize the channel and receiver task
#[tauri::command]
async fn backend_rev(window: tauri::Window) {
    let (tx, rx) = mpsc::channel(100);
    *CHANNEL_TX.lock().await = Some(tx);

    // Start the receiver task
    tokio::spawn(async move {
        receiver_task(rx, window).await;
    });
}

// Receiver Task: Handles all received messages and forwards them as needed
async fn receiver_task(mut rx: mpsc::Receiver<MavMessage>, window: tauri::Window) {
    while let Some(message) = rx.recv().await {
        match message {
            MavMessage::HEARTBEAT(_) => {
                window.emit("log", format!("Received HEARTBEAT")).unwrap();
            }
            MavMessage::COMMAND_ACK(ack) => {
                window
                    .emit("log", format!("Received COMMAND_ACK: {:?}", ack))
                    .unwrap();
            }
            MavMessage::STATUSTEXT(statustext) => {
                window
                    .emit("log", format!("Received STATUSTEXT: {:?}", statustext))
                    .unwrap();
            }
            _ => {
                window
                    .emit("debug_log", format!("Debug: {:?}", message))
                    .unwrap();
            }
        }
    }
}

// Establish a connection and start forwarding received messages to the channel
#[tauri::command]
async fn create_connection(connection_string: String) -> Result<String, String> {
    match connect::<MavMessage>(&connection_string) {
        Ok(conn) => {
            let mut global_conn = GLOBAL_CONNECTION.lock().await;
            *global_conn = Some(conn);

            // Clone Arc references for use within async task
            let conn_clone = GLOBAL_CONNECTION.clone();
            let tx_clone = CHANNEL_TX.lock().await.clone();
            tokio::spawn(async move {
                connection_recv_loop(conn_clone, tx_clone).await;
            });

            Ok("Connected successfully".to_string())
        }
        Err(e) => Err(format!("Connection failed: {:?}", e)),
    }
}

// Task that continuously receives messages from the connection
async fn connection_recv_loop(
    conn: Arc<Mutex<Option<Box<dyn MavConnection<MavMessage> + Send + Sync>>>>,
    tx: Option<mpsc::Sender<MavMessage>>,
) {
    if let Some(tx) = tx {
        while let Some(connection) = &*conn.lock().await {
            match connection.recv() {
                Ok((_header, msg)) => {
                    tx.send(msg).await.unwrap();
                }
                Err(e) => {
                    println!("Error receiving message: {:?}", e);
                }
            }
        }
    }
}

// Main command handler to send a MAVLink message
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
    // Guard clause: Check if the connection is established
    let connection = GLOBAL_CONNECTION.lock().await;
    let connection = match &*connection {
        Some(conn) => conn,
        None => return Err("Connection not established".to_string()),
    };

    // Guard clause: Validate the command ID
    let mav_command = match get_command_from_id(command_id) {
        Some(command) => command,
        None => return Err("Invalid command_id provided.".to_string()),
    };

    // Prepare the MAVLink message data
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

    // Send the command
    connection.send_default(&msg).map_err(|e| {
        let error_msg = format!("Failed to send message: {}", e);
        window.emit("log", &error_msg).unwrap();
        error_msg
    })?;

    window.emit("log", "Message sent successfully").unwrap();
    Ok("Message sent successfully".to_string())
}

async fn await_ack() -> Option<MavMessage> {
    // This function could be more complete in handling channel messages
    None
}

fn get_command_from_id(command_id: u16) -> Option<MavCmd> {
    FromPrimitive::from_u16(command_id)
}
