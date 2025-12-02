use tokio::{io::AsyncWriteExt, net::TcpStream};

#[tokio::main]
async fn main() {
    if let Ok(mut socket) = TcpStream::connect("localhost:8080").await {
        let message = String::from("value");
        socket.write_all(message.as_bytes()).await.unwrap();
    };
    println!("Hello, world!");
}