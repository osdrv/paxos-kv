use crate::command::Command;
use crate::network::{receive_message_with_retry, send_message_with_retry};
use crate::paxos_message::PaxosMessage;
use crate::paxos_node::PaxosNode;

use std::str::FromStr;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

const MAX_RETRIES: u32 = 5;

async fn node_get(
    node: Arc<Mutex<PaxosNode>>,
    topology: Vec<String>,
    key: String,
) -> tokio::io::Result<Option<String>> {
    // TODO: implement the get operation
    println!("TODO: GET key={}", key);
    return Ok(None);
}

async fn node_put(
    node: Arc<Mutex<PaxosNode>>,
    topology: Vec<String>,
    key: String,
    value: String,
) -> tokio::io::Result<()> {
    // TODO: implement the put operation
    println!("TODO: PUT key={} value={}", key, value);
    return Ok(());
}

pub async fn handle_command(
    stream: TcpStream,
    node: Arc<Mutex<PaxosNode>>,
    topology: Vec<String>,
) -> tokio::io::Result<()> {
    let mut reader = BufReader::new(stream);
    let mut buffer = String::new();

    loop {
        buffer.clear();
        let bytes_read = reader.read_line(&mut buffer).await?;
        if bytes_read == 0 {
            break;
        }
        let node = Arc::clone(&node);

        match Command::from_str(&buffer) {
            Ok(Command::Get(key)) => {
                let result = node_get(node, topology.clone(), key).await;
                let response = match result {
                    Ok(Some(value)) => value,
                    Ok(None) => "".to_string(),
                    Err(e) => format!("Error: {:?}", e),
                };
                reader.write_all(response.as_bytes()).await?;
                reader.write_all(b"\n").await?;
            }
            Ok(Command::Put(key, value)) => {
                //let result = node.lock().unwrap().put(key, value);
                let result = node_put(node, topology.clone(), key, value).await;
                let response = match result {
                    Ok(_) => "".to_string(),
                    Err(e) => format!("Error: {:?}", e),
                };
                reader.write_all(response.as_bytes()).await?;
                reader.write_all(b"\n").await?;
            }
            Ok(Command::Invalid(command)) => {
                let response = format!("Invalid command: {}", command);
                reader.write_all(response.as_bytes()).await?;
                reader.write_all(b"\n").await?;
            }
            Err(_) => {
                let response = "Error parsing command".to_string();
                reader.write_all(response.as_bytes()).await?;
                reader.write_all(b"\n").await?;
            }
        }
    }

    Ok(())
}

pub async fn start_control_server(
    node: Arc<Mutex<PaxosNode>>,
    control_addr: String,
    topology: Vec<String>,
) -> tokio::io::Result<()> {
    let listener = TcpListener::bind(control_addr.as_str()).await?;
    println!("Started control server on {}", control_addr);

    loop {
        let (stream, _) = listener.accept().await?;
        let node = Arc::clone(&node);
        let topology = topology.clone();

        tokio::spawn(async move {
            if let Err(r) = handle_command(stream, node, topology).await {
                println!("Error handling command: {:?}", r);
            }
        });
    }
}

pub async fn start_server(node: Arc<Mutex<PaxosNode>>, addr: String) -> tokio::io::Result<()> {
    println!("Starting node {:?} at {:?}", node, addr);
    let listener = TcpListener::bind(addr.as_str()).await?;
    println!("Started a listener at {:?}", addr);

    loop {
        let (mut socket, _) = listener.accept().await?;
        let node = Arc::clone(&node);

        println!("Spawning a new task to handle connections at {:?}", addr);
        tokio::spawn(async move {
            let message = receive_message_with_retry(&mut socket, MAX_RETRIES)
                .await
                .unwrap();
            println!("Server received a new message: {:?}", message);
            let response = process_message(node, message).await;
            println!("Got a response: {:?}", response);

            if let Some(response) = response {
                send_message_with_retry(&mut socket, response, MAX_RETRIES)
                    .await
                    .unwrap();
            }
        });
    }
}

pub async fn process_message(
    node: Arc<Mutex<PaxosNode>>,
    message: PaxosMessage,
) -> Option<PaxosMessage> {
    let mut node = node.lock().unwrap();

    match message {
        PaxosMessage::Prepare { .. } => node.handle_prepare(message),
        PaxosMessage::Promise { .. } => node.handle_promise(message),
        PaxosMessage::Accept { .. } => node.handle_accept(message),
        PaxosMessage::Accepted { .. } => node.handle_accepted(message),
        PaxosMessage::Learn { .. } => {
            node.handle_learn(message);
            None
        }
    }
}

pub async fn send_paxos_message(
    _node: Arc<Mutex<PaxosNode>>,
    addr: &str,
    message: PaxosMessage,
) -> tokio::io::Result<Option<PaxosMessage>> {
    println!("sending a message to addr {:?}", addr);
    let mut stream = TcpStream::connect(addr).await?;
    send_message_with_retry(&mut stream, message, MAX_RETRIES).await?;
    println!("message was sent to {:?}", addr);
    let response = receive_message_with_retry(&mut stream, MAX_RETRIES).await?;
    println!("received a response: {:?}", response);
    Ok(Some(response))
}

impl FromStr for Command {
    type Err = ();

    fn from_str(input: &str) -> Result<Command, Self::Err> {
        let parts: Vec<&str> = input.trim().splitn(3, ' ').collect();
        match parts.as_slice() {
            ["get", key] => Ok(Command::Get(key.to_string())),
            ["put", key, value] => Ok(Command::Put(key.to_string(), value.to_string())),
            _ => Ok(Command::Invalid(input.to_string())),
        }
    }
}
