use super::ws_conn::WSConn;
use crate::messages::{ContentMessage, Message};
use crate::utils::*;
use std::error::Error;
use std::io::Write;
use std::net::SocketAddr;
use std::str;

pub struct Getter {
  pub code: String,
  pub relay_addr: SocketAddr,
  ws_conn: WSConn,
  pub filename: Option<String>,
  pub size: Option<u64>,
  pub peer_addr: Option<SocketAddr>,
}

impl Getter {
  pub fn new(code: Option<&str>, relay_addr: Option<&str>) -> Self {
    let code = code.expect("Room code must be provided").to_string();
    let relay_addr = parse_relay_addr(relay_addr);
    Getter {
      code,
      relay_addr,
      ws_conn: WSConn::new_empty(),
      filename: None,
      size: None,
      peer_addr: None,
    }
  }

  pub async fn connect(&mut self) -> Result<(), Box<dyn Error>> {
    let (tx, rx) = start_ws_conn(self.relay_addr).await?;
    self.ws_conn.init(tx, rx);
    Ok(())
  }

  pub async fn get_room(&mut self) -> Result<(), Box<dyn Error>> {
    let mut msg = Message::new_get(&self.code);
    self.ws_conn.send(&mut msg).await?;

    let mut data = Vec::new();
    self.ws_conn.recv(&mut data).await?;
    let message: Message = serde_json::from_slice(&data)?;
    match message {
      Message::Error(error) => Err(Box::new(error)),
      Message::ApproveReq(req) => {
        self.filename = Some(req.filename.clone());
        self.size = Some(req.size);
        self.peer_addr = Some(req.addr);
        Ok(())
      }
      _ => unreachable!(),
    }
  }

  pub async fn send_approval(&mut self, approved: bool) -> Result<(), Box<dyn Error>> {
    let mut res_message = Message::new_approve_res(approved);
    self.ws_conn.send(&mut res_message).await?;
    Ok(())
  }

  pub async fn start_transfer(
    &mut self,
    wtr: &mut impl Write,
    f: impl Fn(u64) -> (),
  ) -> Result<(), Box<dyn Error>> {
    let chunks = calc_chunks(self.size.unwrap());
    let mut data = Vec::with_capacity(1_000_000);
    for _ in 0..chunks {
      self.ws_conn.recv(&mut data).await?;
      let mut msg: ContentMessage = serde_json::from_slice(&data)?;
      f(msg.content.len().try_into()?);
      wtr.write(&mut msg.content)?;
      data.clear();
    }
    self.ws_conn.close().await?;
    Ok(())
  }
}
