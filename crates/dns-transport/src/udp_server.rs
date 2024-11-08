use dns_core::message::Message;
use std::net::UdpSocket;
use std::sync::Arc;
use std::thread;

pub fn start_udp_server(
    addr: &str,
    handler: Arc<dyn Fn(Message) -> Message + Send + Sync>,
) -> std::io::Result<()> {
    let socket = UdpSocket::bind(addr)?;
    socket.set_nonblocking(true)?;
    let socket = Arc::new(socket);

    thread::spawn({
        let socket = Arc::clone(&socket);
        let handler = handler.clone();
        move || {
            let mut buf = [0u8; 512];
            loop {
                match socket.recv_from(&mut buf) {
                    Ok((size, src)) => {
                        let mut data = &buf[..size];
                        if let Ok(request) = Message::read(&mut data) {
                            let response = handler(request);
                            let mut response_buf = Vec::new();
                            if response.write(&mut response_buf).is_ok() {
                                let _ = socket.send_to(&response_buf, src);
                            }
                        }
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        // Sleep or yield
                        thread::sleep(std::time::Duration::from_millis(100));
                    }
                    Err(e) => {
                        eprintln!("UDP server error: {}", e);
                        break;
                    }
                }
            }
        }
    });

    Ok(())
}
