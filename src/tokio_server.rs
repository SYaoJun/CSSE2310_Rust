use http::http_request::HttpRequest;
use tokio::{io::AsyncReadExt, net::TcpListener};

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("localhost:8080")
        .await
        .expect("TcpListener bind failed");
    loop {
        if let Ok((mut socket, addr)) = listener.accept().await {
            println!("{addr}");
            let mut buffer = String::new();
            socket.read_to_string(&mut buffer).await.unwrap();
            println!("{:#?}", HttpRequest::from(buffer.as_str()));
        }
    }
}