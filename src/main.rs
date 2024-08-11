mod command;
mod network;
mod paxos_message;
mod paxos_node;
mod server;

use crate::paxos_node::PaxosNode;
use crate::server::{send_paxos_message, start_server};
use std::sync::{Arc, Mutex};
use tokio::task;

#[tokio::main]
async fn main() {
    let node1 = Arc::new(Mutex::new(PaxosNode::new(1)));
    let node2 = Arc::new(Mutex::new(PaxosNode::new(2)));

    let node1_addr = "127.0.0.1:8081";
    let node2_addr = "127.0.0.1:8082";

    task::spawn(start_server(Arc::clone(&node1), node1_addr));
    task::spawn(start_server(Arc::clone(&node2), node2_addr));

    let prepare_message =
        node1
            .lock()
            .unwrap()
            .start_proposal(1, "key".to_string(), "value".to_string());
    println!("Sending a prepare message");
    if let Some(promise_response) =
        send_paxos_message(Arc::clone(&node1), node2_addr, prepare_message)
            .await
            .unwrap()
    {
        if let Some(accept_message) = node1.lock().unwrap().handle_promise(promise_response) {
            if let Some(accepted_response) =
                send_paxos_message(Arc::clone(&node1), node2_addr, accept_message)
                    .await
                    .unwrap()
            {
                node1.lock().unwrap().handle_accepted(accepted_response);
            } else {
                println!("Failed to handle accepted");
            }
        } else {
            println!("Failed to handle promise");
        }
    } else {
        println!("Failed to send proposal request");
    }
}
