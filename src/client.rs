use crate::messages::{ContentMessage, Message};
use crate::utils::*;
use bytesize::to_string;
use std::error::Error;
use std::fs;
use std::str;

pub struct Client {}

impl Client {
  pub fn new() -> Self {
    Client {}
  }

  pub async fn send(&self, filepath: &str) -> Result<(), Box<dyn Error>> {
    let filename = validate_filepath(filepath);
    let file = fs::read(filepath)?;
    let (mut sender, mut receiver) = start_ws_conn().await?;

    let code = gen_room_key();
    println!(
      "Sending '{}' ({})",
      filename,
      to_string(file.len().try_into()?, false)
    );
    println!("Code is: {}", code);
    println!("In the other computer run");
    println!("\nsstp get {}\n", code);
    let message = Message::new_send(filename.to_string(), file.len(), code);

    sender.send_text(message).await?;

    let mut data = Vec::new();
    receiver.receive_data(&mut data).await?;

    let message: Message = serde_json::from_slice(&data)?;
    if let Message::Ready(message) = message {
      println!("Sending (->{})", message.addr);
      let pb = create_pb(file.len());
      for chunk in file.chunks(1_000_000) {
        let content = Message::new_content(chunk.to_vec());
        sender.send_binary(content).await?;
        pb.inc(chunk.len().try_into()?);
      }
      pb.finish_and_clear();
      println!("Succesfully sent! ✅");
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
        filename = req.filename.clone();
        size = req.size;
        let approved = req_keyboard_approval(req.filename, req.size);
        let res_message = Message::new_approve_res(approved);
        sender.send_text(res_message).await?;
        if !approved {
          return Ok(());
        }
        println!("Receiving (<-{})", req.addr);
      }
      _ => unreachable!(),
    }

    let mut buffer = Vec::new();
    let chunks = calc_chunks(size);
    let pb = create_pb(size);
    for _ in 0..chunks {
      let mut data = Vec::new();
      receiver.receive_data(&mut data).await?;
      let mut msg: ContentMessage = serde_json::from_slice(&data)?;
      pb.inc(msg.content.len().try_into()?);
      buffer.append(&mut msg.content);
    }
    pb.finish_and_clear();
    println!("Downloaded ✅");

    sender.flush().await?;
    fs::write(filename, buffer)?;
    Ok(())
  }
}
