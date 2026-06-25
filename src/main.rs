use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
fn handle_client(mut stream: TcpStream) {
    println!("New connection: {}", stream.peer_addr().unwrap());
    let mut buffer = [0; 512];
    match stream.read(&mut buffer) {
        Ok(0) => {
            println!("Client disconnected");
        }
        Ok(bytes_read) => {
            let raw_command = String::from_utf8_lossy(&buffer[..bytes_read]);
            println!("Received: {}", raw_command);

        let body = match raw_command.find("\r\n\r\n") {
                Some(index) => &raw_command[index + 4..], 
                None => &raw_command,                     
            };

          let response = format!(
                "HTTP/1.1 200 OK\r\n\
                Content-Type: text/plain\r\n\
                Content-Length: {}\r\n\
                Connection: close\r\n\
                \r\n\
                {}",
                body.len(),
                body
            );
            if let Err(e) = stream.write(response.as_bytes()) {
                println!("Failed to send response: {}", e);
            }
        }
        Err(e) => {
            println!("Error reading from client: {}", e);
        }
    };
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
