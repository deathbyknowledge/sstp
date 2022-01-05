use super::ws_conn::WSConn;
use crate::messages::Message;
use crate::utils::*;
use std::error::Error;
use std::io::Read;
use std::net::SocketAddr;

pub struct Sender {
  pub filename: String,
  pub code: Option<String>,
  pub peer_addr: Option<SocketAddr>,
  pub size: u64,
  relay_addr: SocketAddr,
  ws_conn: WSConn,
}

impl Sender {
  pub fn new(filename: String, size: u64, relay_addr: Option<&str>) -> Self {
    let relay_addr = parse_relay_addr(relay_addr);
    Sender {
      filename,
      relay_addr,
      size,
      ws_conn: WSConn::new_empty(),
      code: None,
      peer_addr: None,
    }
  }

  pub async fn connect(&mut self) -> Result<(), Box<dyn Error>> {
    let (tx, rx) = start_ws_conn(self.relay_addr).await?;
    self.ws_conn.init(tx, rx);
    Ok(())
  }

  pub async fn create_room(&mut self) -> Result<(), Box<dyn Error>> {
    let code = gen_room_key();
    let mut message = Message::new_send(self.filename.to_string(), self.size, code.clone());
    self.code = Some(code);
    self.ws_conn.send(&mut message).await?;
    Ok(())
  }

  pub async fn wait_for_receiver(&mut self) -> Result<(), Box<dyn Error>> {
    let mut data = Vec::new();
    self.ws_conn.recv(&mut data).await?;

    let message: Message = serde_json::from_slice(&data)?;
    if let Message::Ready(message) = message {
      self.peer_addr = Some(message.addr);
      Ok(())
    } else {
      panic!("Expected ReadyMessage. Got something else.");
    }
  }

  pub async fn close_conn(&mut self) -> Result<(), Box<dyn Error>> {
    self.ws_conn.close().await?;
    Ok(())
  }

  pub async fn start_transfer(
    &mut self,
    rdr: &mut impl Read,
    f: impl Fn(u64) -> (),
  ) -> Result<(), Box<dyn Error>> {
    let mut buff = vec![0; 1_000_000];
    while let n @ 1.. = rdr.read(&mut buff)? {
      self.send_chunk(&buff[0..n]).await?;
    }
    Ok(())
  }

  async fn send_chunk(&mut self, chunk: &[u8]) -> Result<(), Box<dyn Error>> {
    if chunk.len() > 1_000_000 {
      panic!("Chunks should not be bigger than 1MB");
    }
    let mut content = Message::new_content(chunk.to_vec());
    self.ws_conn.send(&mut content).await?;
    Ok(())
  }

  pub async fn finish(&mut self) -> Result<(), Box<dyn Error>> {
    self.ws_conn.close().await?;
    Ok(())
  }
}
