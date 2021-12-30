use crate::messages::{ContentMessage, Message};
use crate::utils::*;
use bytesize::to_string;
use std::error::Error;
use std::fs;
use std::net::SocketAddr;
use std::str;

pub struct Client {}

pub struct SendParams {
  filepath: String,
  filename: String,
  relay_addr: SocketAddr,
}

impl SendParams {
  pub fn new(filepath: Option<&str>, relay_addr: Option<&str>) -> Self {
    let filepath = filepath.expect("Filepath must be provided").to_string();
    let filename = validate_filepath(&filepath);
    let relay_addr = parse_relay_addr(relay_addr);
    SendParams {
      filepath,
      filename,
      relay_addr,
    }
  }
}

pub struct GetParams {
  code: String,
  relay_addr: SocketAddr,
}

impl GetParams {
  pub fn new(code: Option<&str>, relay_addr: Option<&str>) -> Self {
    let code = code.expect("Room code must be provided").to_string();
    let relay_addr = parse_relay_addr(relay_addr);
    GetParams { code, relay_addr }
  }
}

impl Client {
  pub fn new() -> Self {
    Client {}
  }

  pub async fn send(&self, params: SendParams) -> Result<(), Box<dyn Error>> {
    let file = fs::read(params.filepath)?;
    let filename = params.filename;
    let (mut sender, mut receiver) = start_ws_conn(params.relay_addr).await?;

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

  pub async fn get(&self, params: GetParams) -> Result<(), Box<dyn Error>> {
    let (mut sender, mut receiver) = start_ws_conn(params.relay_addr).await?;

    let message = Message::new_get(params.code);
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
