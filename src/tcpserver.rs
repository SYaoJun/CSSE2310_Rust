use std::net::TcpListener;
use std::io::{Read, Write};
fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        let mut buffer = [0 as u8; 1024];
        stream.read(&mut buffer).unwrap();
        let request = String::from_utf8_lossy(&buffer);
        println!("Request: {}", request);
        stream.write(b"hello world from server").unwrap();
    }
}