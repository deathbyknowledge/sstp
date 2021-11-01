use std::str;
use std::error::Error;
use tokio_util::compat::TokioAsyncReadCompatExt;
use soketto::handshake::{Client, ServerResponse};
use crate::messages::Message;

pub struct Sender {}

impl Sender {
    pub fn new() -> Self {
        Sender {}
    }

    pub async fn send(&self, filename: &str) -> Result<(), Box<dyn Error>> {
        let socket = tokio::net::TcpStream::connect("127.0.0.1:8004").await?;

        let mut client = Client::new(socket.compat(), "127.0.0.1:8004", "/");

        let (mut sender, mut receiver) = match client.handshake().await? {
            ServerResponse::Accepted { .. } => client.into_builder().finish(),
            ServerResponse::Redirect { .. } => unimplemented!("f"),
            ServerResponse::Rejected { .. } => unimplemented!("f"),
        };


        let message = Message::new_send(filename);
        let message_text = serde_json::to_string(&message)?;
        
        sender.send_text(message_text).await?;

        let mut data = Vec::new();
        receiver.receive_data(&mut data).await?;

        let message: Message = serde_json::from_str(str::from_utf8(&data).unwrap())?;
        if message.is_ready() {
            sender.send_text(filename).await?;
        }
        sender.flush().await?;
        println!("{}", str::from_utf8(&data).unwrap());

        Ok(())
    }

    pub async fn get(&self, code: &str) -> Result<(), Box<dyn Error>> {
        let socket = tokio::net::TcpStream::connect("127.0.0.1:8004").await?;

        let mut client = Client::new(socket.compat(), "127.0.0.1:8004", "/");

        let (mut sender, mut receiver) = match client.handshake().await? {
            ServerResponse::Accepted { .. } => client.into_builder().finish(),
            ServerResponse::Redirect { .. } => unimplemented!("f"),
            ServerResponse::Rejected { .. } => unimplemented!("f"),
        };


        let message = Message::new_get(code);
        let message_text = serde_json::to_string(&message)?;
        
        sender.send_text(message_text).await?;
        sender.flush().await?;

        let mut data = Vec::new();
        receiver.receive_data(&mut data).await?;
        println!("Shared this: {}", str::from_utf8(&data).unwrap());

        Ok(())
    }
}
