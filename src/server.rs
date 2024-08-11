use crate::network::{receive_message_with_retry, send_message_with_retry};
use crate::paxos_message::PaxosMessage;
use crate::paxos_node::PaxosNode;

use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpStream};

const MAX_RETRIES: u32 = 5;

pub async fn start_server(node: Arc<Mutex<PaxosNode>>, addr: &str) -> tokio::io::Result<()> {
    println!("Starting node {:?} at {:?}", node, addr);
    let listener = TcpListener::bind(addr).await?;
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
