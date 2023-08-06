mod commands;
mod config;
mod state;

use commands::commands::Command;
use config::ApplicationConfig;
use config_file::FromConfigFile;
use parking_lot::RwLock;
use state::datastate::DataState;
use std::{path::PathBuf, sync::Arc};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

use crate::commands::parser::Parser;
use crate::state::serverstate::ServerState;
#[repr(u8)]
pub enum CommandResult {
    OK = 1,
    ERR = 2,
}
async fn manage_socket(
    mut socket: TcpStream,
    server_state: Arc<RwLock<ServerState>>,
    data_state: Arc<DataState>,
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

        //let cmd = String::from_utf8_lossy(&next_buff).into_owned();
        let command_result = Parser::parse(&next_buff);
        let mut response: Vec<u8> = Vec::new();
        match command_result {
            Ok(cmd) => match process_cmd(&cmd, &server_state, &data_state).await {
                Ok(data) => {
                    response.push(CommandResult::OK as u8);
                    response.append(&mut (data.to_vec()));
                }
                Err(message) => {
                    response.push(CommandResult::ERR as u8);
                    response.append(&mut message.as_bytes().to_vec());
                }
            },
            Err(()) => {
                response.push(CommandResult::ERR as u8);
                response.append(&mut "Could not process command".as_bytes().to_vec());
            }
        }

        // Write the data back
        if let Err(e) = socket.write_all(&response).await {
            break format!("failed to write to socket; err = {:?}", e);
        }
    };
    {
        let mut mut_ser_state = server_state.write();
        mut_ser_state.current_connections -= 1;
    }
    eprintln!("Closing socket {} due to {}", socket_addr, result);
}

async fn process_cmd<'a>(
    cmd: &Command<'_>,
    server_state: &Arc<RwLock<ServerState>>,
    data_state: &Arc<DataState>,
) -> Result<Vec<u8>, String> {
    let result = cmd.execute(data_state, server_state);
    if !result.is_err() {
        let mut state = server_state.write();
        state.processed_commands += 1;
    }
    match result {
        Ok(o) => match o {
            Some(o) => Ok(o),
            None => Ok("OK".as_bytes().to_vec()),
        },
        Err(e) => Err(e),
    }
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app_cfg: ApplicationConfig =
        ApplicationConfig::from_config_file(PathBuf::from("./echors.toml")).unwrap();
    let server_state = Arc::new(RwLock::new(ServerState::new(env!("CARGO_PKG_VERSION"))));
    let data_state = Arc::new(DataState::new());
    println!("Starting server. Binding on: {}", &app_cfg.bind);
    let listener: TcpListener = TcpListener::bind(&app_cfg.bind).await?;
    //let max_conn_limiter = Arc::new(Semaphore::new(app_cfg.max_connections as usize));
    loop {
        // let permit = max_conn_limiter.clone().acquire_owned().await.unwrap();
        match listener.accept().await {
            Ok((mut _socket, _addr)) => {
                let current_connections;
                {
                    let ro_state_data = server_state.read();
                    current_connections = ro_state_data.current_connections;
                }
                if current_connections >= app_cfg.max_connections as u32 {
                    println!("Dropping {:?} due to max_conn limitation", _addr);
                    drop(_socket);
                } else {
                    {
                        let mut mut_state_data = server_state.write();
                        mut_state_data.current_connections += 1;
                        mut_state_data.total_connections += 1;
                        drop(mut_state_data);
                    }
                    tokio::spawn(manage_socket(
                        _socket,
                        server_state.clone(),
                        data_state.clone(),
                    ));
                }
            }
            Err(e) => println!("{:?}", e),
        }
    }
}
