use std::str;
use std::error::Error;
use std::fs;
use tokio_util::compat::TokioAsyncReadCompatExt;
use soketto::handshake::{Client, ServerResponse};
use crate::messages::{Message, ContentMessage};

pub struct Sender {}

impl Sender {

    pub fn new() -> Self {
        Sender {}
    }

    pub async fn send(&self, filename: &str) -> Result<(), Box<dyn Error>> {
        let socket = tokio::net::TcpStream::connect("138.68.103.243:8004").await?;

        let mut client = Client::new(socket.compat(), "138.68.103.243:8004", "/");

        let (mut sender, mut receiver) = match client.handshake().await? {
            ServerResponse::Accepted { .. } => client.into_builder().finish(),
            ServerResponse::Redirect { .. } => unimplemented!("f"),
            ServerResponse::Rejected { .. } => unimplemented!("f"),
        };

        let message = Message::new_send(filename.to_string());
        let message_text = serde_json::to_string(&message)?;
        
        sender.send_text(message_text).await?;

        let mut data = Vec::new();
        receiver.receive_data(&mut data).await?;

        let message: Message = serde_json::from_slice(&data)?;
        if let Message::Ready = message {
            let data = fs::read(filename)?;
            let message = Message::new_content(filename.to_string(), data);
            let msg = serde_json::to_vec(&message)?;
            sender.send_binary(msg).await?;
        } else {
            panic!("Expected ReadyMessage. Got something else.");
        }
        sender.flush().await?;

        Ok(())
    }

    pub async fn get(&self, code: &str) -> Result<(), Box<dyn Error>> {
        let socket = tokio::net::TcpStream::connect("138.68.103.243:8004").await?;

        let mut client = Client::new(socket.compat(), "138.68.103.243:8004", "/");

        let (mut sender, mut receiver) = match client.handshake().await? {
            ServerResponse::Accepted { .. } => client.into_builder().finish(),
            ServerResponse::Redirect { .. } => unimplemented!("f"),
            ServerResponse::Rejected { .. } => unimplemented!("f"),
        };

        let message = Message::new_get(code.to_string());
        let message_text = serde_json::to_string(&message)?;
        
        sender.send_text(message_text).await?;
        sender.flush().await?;

        let mut data = Vec::new();
        receiver.receive_data(&mut data).await?;
        println!("Recieved data. Starting fs op");
        let content_message: ContentMessage = serde_json::from_slice(&data)?;
        fs::write(content_message.filename, content_message.content)?;
        Ok(())
    }
}
