use dns_core::message::Message;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::thread;

pub fn start_tcp_server(
    addr: &str,
    handler: Arc<dyn Fn(Message) -> Message + Send + Sync>,
) -> std::io::Result<()> {
    let listener = TcpListener::bind(addr)?;
    let handler = handler.clone();

    thread::spawn(move || {
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let handler = Arc::clone(&handler);
                    thread::spawn(move || {
                        handle_client(stream, handler);
                    });
                }
                Err(e) => {
                    eprintln!("TCP server connection error: {}", e);
                }
            }
        }
    });

    Ok(())
}

fn handle_client(mut stream: TcpStream, handler: Arc<dyn Fn(Message) -> Message + Send + Sync>) {
    loop {

        let mut len_buf = [0u8; 2];
        if let Err(_) = stream.read_exact(&mut len_buf) {
            break;
        }
        let len = u16::from_be_bytes(len_buf) as usize;
        let mut msg_buf = vec![0u8; len];
        if let Err(_) = stream.read_exact(&mut msg_buf) {
            break;
        }
        let mut data = &msg_buf[..];
        if let Ok(request) = Message::read(&mut data) {
            let response = handler(request);
            let mut response_buf = Vec::new();
            if response.write(&mut response_buf).is_ok() {
                let response_len = response_buf.len() as u16;
                let _ = stream.write_all(&response_len.to_be_bytes());
                let _ = stream.write_all(&response_buf);
            }
        } else {
            break;
        }
    }
}
