use std::str;
use std::error::Error;
use std::fs;
use tokio_util::compat::{TokioAsyncReadCompatExt, Compat};
use soketto::handshake::{Client, ServerResponse};
use soketto::connection::{Sender as SenderSk, Receiver as ReceiverSk};
use crate::messages::{Message, ContentMessage};
use tokio::net::TcpStream;

pub struct Sender {}

impl Sender {

    pub fn new() -> Self {
        Sender {}
    }

    pub async fn send(&self, filename: &str) -> Result<(), Box<dyn Error>> {

        let (mut sender, mut receiver) = start_ws_conn().await?;
        let data = fs::read(filename)?;

        let message = Message::new_send(filename.to_string(), data.len());
        let message_text = serde_json::to_string(&message)?;
        
        sender.send_text(message_text).await?;

        let mut data = Vec::new();
        receiver.receive_data(&mut data).await?;

        let message: Message = serde_json::from_slice(&data)?;
        if let Message::Ready = message {
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
        
        let (mut sender, mut receiver) = start_ws_conn().await?;

        let message = Message::new_get(code.to_string());
        let message_text = serde_json::to_string(&message)?;
        
        sender.send_text(message_text).await?;

        let mut data = Vec::new();
        receiver.receive_data(&mut data).await?;
        let message: Message = serde_json::from_slice(&data)?;

        match message {
          Message::Error(error) =>  {
            println!("{}", error.text);
            return Ok(());
          },
          Message::ApproveReq(req) => {
            println!("Accept {} ({})? (y/n)", req.filename, req.size);
            let mut input = read_input(); 
            while input != "y" && input != "n" {
              println!("Please submit only 'y' or 'n'.");
              input = read_input();
            }
            let approved = input == "y";
            let res_message = Message::new_approve_res(approved);
            sender.send_text(res_message).await?;
          },
          _ => unreachable!(),
        }
        let content_message: ContentMessage = serde_json::from_slice(&data)?;

        sender.flush().await?;
        fs::write(content_message.filename, content_message.content)?;
        Ok(())
    }
}

async fn start_ws_conn() -> Result<(SenderSk<Compat<TcpStream>>, ReceiverSk<Compat<TcpStream>>), Box<dyn Error>> {
    let socket = TcpStream::connect("138.68.103.243:8004").await?;

    let mut client = Client::new(socket.compat(), "138.68.103.243:8004", "/");

    let (sender, receiver) = match client.handshake().await? {
        ServerResponse::Accepted { .. } => client.into_builder().finish(),
        ServerResponse::Redirect { .. } => unimplemented!("f"),
        ServerResponse::Rejected { .. } => unimplemented!("f"),
    };

    Ok((sender, receiver))
}

fn read_input() -> String {
  use std::io::{stdin, stdout, Write};
  let mut s = String::new();
  let _ = stdout().flush();
  stdin().read_line(&mut s).expect("Did not enter a correct string");
  s
}
