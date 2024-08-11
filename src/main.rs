pub mod command;
pub mod network;
pub mod paxos_message;
pub mod paxos_node;
pub mod server;

use crate::paxos_node::PaxosNode;
use crate::server::{send_paxos_message, start_server};
use std::sync::{Arc, Mutex};
use tokio::task;

fn read_topology(path: &str) -> Vec<String> {
    let file = std::fs::read_to_string(path).expect("Failed to read the file");
    file.lines().map(|s| s.trim().to_string()).collect()
}

#[tokio::main]
async fn main() {
    let topology = read_topology("topology.txt");
    let node_addr = std::env::args().nth(1).expect("Expected a node address");
    let control_addr = std::env::args().nth(2).expect("Expected a control address");

    let node_ix = topology
        .iter()
        .position(|x| x == &node_addr)
        .expect("Node not found in topology") as u64;

    let node = Arc::new(Mutex::new(PaxosNode::new(node_ix)));
    let jh_srv = task::spawn(start_server(Arc::clone(&node), node_addr));
    let jh_control_srv = task::spawn(server::start_control_server(
        Arc::clone(&node),
        control_addr,
        topology,
    ));

    jh_srv.await.unwrap();
    jh_control_srv.await.unwrap();

    // let jh = task::spawn();
    // jh.await.unwrap();

    // let node1 = Arc::new(Mutex::new(PaxosNode::new(1)));
    // let node2 = Arc::new(Mutex::new(PaxosNode::new(2)));

    // let node1_addr = "127.0.0.1:8081";
    // let node2_addr = "127.0.0.1:8082";

    // task::spawn(start_server(Arc::clone(&node1), node1_addr));
    // task::spawn(start_server(Arc::clone(&node2), node2_addr));

    // let prepare_message =
    //     node1
    //         .lock()
    //         .unwrap()
    //         .start_proposal(1, "key".to_string(), "value".to_string());
    // println!("Sending a prepare message");
    // if let Some(promise_response) =
    //     send_paxos_message(Arc::clone(&node1), node2_addr, prepare_message)
    //         .await
    //         .unwrap()
    // {
    //     if let Some(accept_message) = node1.lock().unwrap().handle_promise(promise_response) {
    //         if let Some(accepted_response) =
    //             send_paxos_message(Arc::clone(&node1), node2_addr, accept_message)
    //                 .await
    //                 .unwrap()
    //         {
    //             node1.lock().unwrap().handle_accepted(accepted_response);
    //         } else {
    //             println!("Failed to handle accepted");
    //         }
    //     } else {
    //         println!("Failed to handle promise");
    //     }
    // } else {
    //     println!("Failed to send proposal request");
    // }
}
