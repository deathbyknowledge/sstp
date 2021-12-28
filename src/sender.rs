use crate::messages::{ContentMessage, Message};
use crate::utils::*;
use std::error::Error;
use std::fs;
use std::str;

pub struct Sender {}

impl Sender {
  pub fn new() -> Self {
    Sender {}
  }

  pub async fn send(&self, filename: &str) -> Result<(), Box<dyn Error>> {
    let (mut sender, mut receiver) = start_ws_conn().await?;
    let file = fs::read(filename)?;

    let key = gen_room_key();
    println!("You're key is {}", key);
    let message = Message::new_send(filename.to_string(), file.len(), key);

    sender.send_text(message).await?;

    let mut data = Vec::new();
    receiver.receive_data(&mut data).await?;

    let message: Message = serde_json::from_slice(&data)?;
    if let Message::Ready = message {
      for chunk in file.chunks(1_000_000) {
        let msg = Message::new_content(chunk.to_vec());
        sender.send_binary(msg).await?;
      }
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
    let filename;
    let size;
    match message {
      Message::Error(error) => {
        println!("{}", error.text);
        return Ok(());
      }
      Message::ApproveReq(req) => {
        println!("Accept {} ({}b)? (y/n)", req.filename, req.size);
        filename = req.filename;
        size = req.size;
        let approved = req_keyboard_approval();
        let res_message = Message::new_approve_res(approved);
        sender.send_text(res_message).await?;
        if !approved {
          return Ok(());
        }
      }
      _ => unreachable!(),
    }

    let mut buffer = Vec::new();
    let chunks = calc_chunks(size);
    for _ in 0..chunks {
      let mut data = Vec::new();
      receiver.receive_data(&mut data).await?;
      let mut msg: ContentMessage = serde_json::from_slice(&data)?;
      buffer.append(&mut msg.content);
    }

    sender.flush().await?;
    fs::write(filename, buffer)?;
    Ok(())
  }
}
