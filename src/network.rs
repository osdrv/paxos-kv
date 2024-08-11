use crate::paxos_message::PaxosMessage;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};

pub async fn send_message(stream: &mut TcpStream, message: PaxosMessage) -> tokio::io::Result<()> {
    let serialized = serde_json::to_vec(&message).unwrap();
    let length = serialized.len() as u32;
    let length_bytes = length.to_be_bytes(); // Convert length to big-endian byte array

    // Send length followed by the serialized message
    let send_len_res = timeout(Duration::from_secs(5), stream.write_all(&length_bytes)).await;
    let send_payload_res = timeout(Duration::from_secs(5), stream.write_all(&serialized)).await;

    if send_len_res.is_err() {
        return Err(tokio::io::Error::new(
            tokio::io::ErrorKind::TimedOut,
            "Timed out while sending message length",
        ));
    }
    if send_payload_res.is_err() {
        return Err(tokio::io::Error::new(
            tokio::io::ErrorKind::TimedOut,
            "Timed out while sending message payload",
        ));
    }
    Ok(())
}

pub async fn send_message_with_retry(
    stream: &mut TcpStream,
    message: PaxosMessage,
    retries: u32,
) -> tokio::io::Result<()> {
    let mut attempt = 0;
    loop {
        match send_message(stream, message.clone()).await {
            Ok(_) => break,
            Err(e) => {
                if attempt >= retries {
                    return Err(e);
                }
                attempt += 1;
            }
        }
    }
    Ok(())
}

pub async fn receive_message(stream: &mut TcpStream) -> tokio::io::Result<PaxosMessage> {
    let mut length_bytes = [0u8; 4]; // 4 bytes to store the message length
    let read_len_res = timeout(Duration::from_secs(5), stream.read_exact(&mut length_bytes)).await; // Read exactly 4 bytes

    if read_len_res.is_err() {
        return Err(tokio::io::Error::new(
            tokio::io::ErrorKind::TimedOut,
            "Timed out while reading message length",
        ));
    }

    let length = u32::from_be_bytes(length_bytes) as usize; // Convert bytes to usize
    let mut buffer = vec![0u8; length]; // Allocate buffer for the message
    let read_msg_res = timeout(Duration::from_secs(5), stream.read_exact(&mut buffer)).await; // Read the exact number of bytes

    if read_msg_res.is_err() {
        return Err(tokio::io::Error::new(
            tokio::io::ErrorKind::TimedOut,
            "Timed out while reading message",
        ));
    }

    let message: PaxosMessage = serde_json::from_slice(&buffer).unwrap();
    Ok(message)
}

pub async fn receive_message_with_retry(
    stream: &mut TcpStream,
    retries: u32,
) -> tokio::io::Result<PaxosMessage> {
    let mut attempt = 0;
    loop {
        match receive_message(stream).await {
            Ok(message) => return Ok(message),
            Err(e)
                if e.kind() == tokio::io::ErrorKind::TimedOut
                    || e.kind() == tokio::io::ErrorKind::ConnectionRefused =>
            {
                if attempt >= retries {
                    return Err(e);
                }
                attempt += 1;
            }
            Err(e) => return Err(e),
        }
    }
}
