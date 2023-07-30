mod config;
mod state;

use config::ApplicationConfig;
use config_file::FromConfigFile;
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::{OwnedSemaphorePermit, Semaphore},
};

use crate::state::serverstate::ServerState;

async fn manage_socket(
    mut socket: TcpStream,
    permit: OwnedSemaphorePermit,
    server_state: Arc<Mutex<ServerState>>,
) {
    let socket_addr = socket.peer_addr().unwrap();
    println!("Client {} connected.", socket_addr);
    let result = loop {
        let buf_len = match socket.read_u32_le().await {
            Ok(n) => n,
            Err(e) => break format!("failed to read from socket; err = {:?}", e),
        } as usize;

        let mut next_buff: Vec<u8> = vec![0; buf_len];
        let _ = match socket.read(&mut next_buff).await {
            // socket closed
            Ok(n) if n != buf_len => {
                break format!(
                    "Expected {:?} bytes long message. Found {:?} bytes.",
                    buf_len, n
                )
            }
            Ok(n) => n,
            Err(e) => break format!("failed to read from socket; err = {:?}", e),
        };

        let cmd = String::from_utf8_lossy(&next_buff).into_owned();
        println!("Received {} from {:?}", cmd, socket_addr);
        let result = match process_cmd(&cmd, &server_state).await {
            Ok(data) => data,
            Err(message) => message.as_bytes().to_vec(),
        };

        // Write the data back
        if let Err(e) = socket.write_all(&result).await {
            break format!("failed to write to socket; err = {:?}", e);
        }
    };
    server_state.lock().unwrap().current_connections -= 1;
    eprintln!("Closing socket {} due to {}", socket_addr, result);
    drop(permit);
}

async fn process_cmd(cmd: &str, server_state: &Arc<Mutex<ServerState>>) -> Result<Vec<u8>, String> {
    let mut state = server_state.lock().unwrap();
    state.processed_commands += 1;
    return match cmd {
        "info" => {
            let info = state.to_string();
            Ok(info.as_bytes().to_vec())
        }
        "test" => Ok("OK".as_bytes().to_vec()),
        _ => Err(format!("Invalid command {}", cmd).to_owned()),
    };
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app_cfg: ApplicationConfig =
        ApplicationConfig::from_config_file(PathBuf::from("./echors.toml")).unwrap();

    let server_state = Arc::new(Mutex::new(ServerState::new(env!("CARGO_PKG_VERSION"))));
    println!("Starting server. Binding on: {}", &app_cfg.bind);
    let listener: TcpListener = TcpListener::bind(&app_cfg.bind).await?;
    let max_conn_limiter = Arc::new(Semaphore::new(app_cfg.max_connections as usize));
    loop {
        let permit = max_conn_limiter.clone().acquire_owned().await.unwrap();
        match listener.accept().await {
            Ok((mut _socket, _addr)) => {
                println!("New client {:?}", _addr);
                let mut mut_state_data = server_state.lock().unwrap();
                mut_state_data.current_connections += 1;
                mut_state_data.total_connections += 1;
                tokio::spawn(manage_socket(_socket, permit, server_state.clone()));
            }
            Err(e) => println!("{:?}", e),
        }
    }
}
