mod config;
mod state;

use config::ApplicationConfig;
use config_file::FromConfigFile;
use parking_lot::RwLock;
use state::datastate::DataState;
use state::datastate::DataType;
use std::mem;
use std::{path::PathBuf, sync::Arc};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

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

        let cmd = String::from_utf8_lossy(&next_buff).into_owned();
        println!("Received {} from {:?}", cmd, socket_addr);
        let mut result: Vec<u8> = Vec::new();
        match process_cmd(&cmd, &server_state, &data_state).await {
            Ok(mut data) => {
                result.push(CommandResult::OK as u8);
                result.append(&mut data);
            }
            Err(message) => {
                result.push(CommandResult::ERR as u8);
                result.append(&mut message.as_bytes().to_vec());
            }
        };

        // Write the data back
        if let Err(e) = socket.write_all(&result).await {
            break format!("failed to write to socket; err = {:?}", e);
        }
    };
    {
        let mut mut_ser_state = server_state.write();
        mut_ser_state.current_connections -= 1;
    }
    eprintln!("Closing socket {} due to {}", socket_addr, result);
}

async fn process_cmd(
    cmd: &str,
    server_state: &Arc<RwLock<ServerState>>,
    data_state: &Arc<DataState>,
) -> Result<Vec<u8>, String> {
    let str_state;
    {
        let mut state = server_state.write();
        state.processed_commands += 1;
        str_state = state.to_string();
    }
    return match cmd {
        "info" => {
            let info = str_state.to_string();
            Ok(info.as_bytes().to_vec())
        }
        "test" => Ok("OK".as_bytes().to_vec()),
        _ => {
            let cmd_start = cmd.split_once(' ').map(|(p, _)| p).unwrap_or(cmd);
            // ToDo: change to lexer
            let cmd_params: Vec<&str> = cmd
                .split_ascii_whitespace()
                .enumerate()
                .filter_map(|(i, p)| (i != 0).then(|| p)) // skip first element, the command
                .collect();
            match cmd_start {
                "set" => {
                    if cmd_params.len() != 2 {
                        return Err(format!("Command {} requires 2 paramters", cmd).to_owned());
                    } else {
                        {
                            let key = *cmd_params.get(0).unwrap();
                            {
                                let read_state = data_state.data.read();
                                if !read_state.contains_key(key) {
                                    drop(read_state);
                                    {
                                        let mut write_state = data_state.data.write();
                                        write_state.insert(
                                            key.to_owned(),
                                            RwLock::new(DataType::String(
                                                (*cmd_params.get(1).unwrap()).to_owned(),
                                            )),
                                        );
                                    }
                                } else {
                                    let mut value_lock = read_state.get(key).unwrap().write();
                                    {
                                        let _ = mem::replace(
                                            &mut *value_lock,
                                            DataType::String(
                                                (*cmd_params.get(1).unwrap()).to_owned(),
                                            ),
                                        );
                                    }
                                }
                            }
                        }
                        Ok("OK".as_bytes().to_vec())
                    }
                }
                "get" => {
                    if cmd_params.len() != 1 {
                        return Err(format!("Command {} requires 1 paramters", cmd).to_owned());
                    } else {
                        let key = *cmd_params.get(0).unwrap();
                        let read_state = data_state.data.read();
                        if !read_state.contains_key(key) {
                            Err("Key not found".to_owned())
                        } else {
                            let val_lock = read_state.get(key).unwrap();
                            let val_read = val_lock.read();
                            let result = match &*val_read {
                                DataType::String(v) => Ok(v.as_bytes().to_vec()),
                                DataType::Number(v) => Ok(v.to_vec()),
                                DataType::List(_) => Err("Cannot get list".to_owned()),
                            };

                            return result;
                        }
                    }
                }
                _ => Err(format!("Invalid command {}", cmd_start).to_owned()),
            }
        }
    };
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
