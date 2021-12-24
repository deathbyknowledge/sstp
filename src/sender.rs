use crate::messages::{ContentMessage, Message};
use crate::utils::*;
use std::fs;
use std::str;
use std::error::Error;

pub struct Sender {}

impl Sender {
  pub fn new() -> Self {
    Sender {}
  }

  pub async fn send(&self, filename: &str) -> Result<(), Box<dyn Error>> {
    let (mut sender, mut receiver) = start_ws_conn().await?;
    let file = fs::read(filename)?;

    let message = Message::new_send(filename.to_string(), file.len());
    let message_text = serde_json::to_string(&message)?;

    sender.send_text(message_text).await?;

    let mut data = Vec::new();
    receiver.receive_data(&mut data).await?;

    let message: Message = serde_json::from_slice(&data)?;
    if let Message::Ready = message {
      let message = Message::new_content(filename.to_string(), file);
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
      Message::Error(error) => {
        println!("{}", error.text);
        return Ok(());
      }
      Message::ApproveReq(req) => {
        println!("Accept {} ({})? (y/n)", req.filename, req.size);
        let approved = req_keyboard_approval();
        let res_message = Message::new_approve_res(approved);
        sender.send_text(res_message).await?;
        if !approved {
          return Ok(());
        }
      }
      _ => unreachable!(),
    }
    data = Vec::new();
    receiver.receive_data(&mut data).await?;
    let content_message: ContentMessage = serde_json::from_slice(&data)?;

    sender.flush().await?;
    fs::write(content_message.filename, content_message.content)?;
    Ok(())
  }
}
