use std::io::{Read, Write};
use std::net::{TcpStream};


fn main() {
    let addr = "127.0.0.1:8080";
    let mut stream = TcpStream::connect(addr).unwrap();
    stream.write(b"hello world from client").unwrap();
    let mut buffer = [0 as u8; 1024];
    stream.read(&mut buffer).unwrap();
    let response = String::from_utf8_lossy(&buffer);
    println!("Response: {}", response);
}