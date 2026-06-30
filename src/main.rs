mod resp;

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

fn handle_client(mut stream: TcpStream) {
    println!("New connection: {}", stream.peer_addr().unwrap());
    loop {
        let mut buffer = [0; 512];
        match stream.read(&mut buffer) {
            Ok(0) => {
                println!("Client disconnected");
                break;
            }
            Ok(bytes_read) => {
                let raw_command = String::from_utf8_lossy(&buffer[..bytes_read]);
                println!("Received: {:?}", raw_command);
                let raw_bytes: &[u8] = &buffer[..bytes_read];
                let content = match resp::decode(raw_bytes) {
                    Ok(res) => format!("+{:?}\r\n", res),
                    Err(e) => format!("-ERR {}\r\n", e),
                };

                let response = format!(
                    "HTTP/1.1 200 OK\r\n\
                Content-Type: text/plain\r\n\
                Content-Length: {}\r\n\
                Connection: keep-alive\r\n\
                \r\n\
                {}",
                    content.len(),
                    content
                );
                if let Err(e) = stream.write(response.as_bytes()) {
                    println!("Failed to send response: {}", e);
                    break;
                }
                if let Err(e) = stream.flush() {
                    println!("Flush failed: {}", e);
                    break;
                }
            }
            Err(e) => {
                println!("Error reading from client: {}", e);
                break;
            }
        };
    }
}
fn main() {
    let listener = TcpListener::bind("127.0.0.1:7379").expect("Could not bind to port 7379");
    for stream in listener.incoming() {
        match stream {
            Ok(client_stream) => {
                handle_client(client_stream);
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
}
