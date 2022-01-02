use soketto::connection::{Receiver as ReceiverSk, Sender as SenderSk};
use std::error::Error;
use tokio::net::TcpStream;
use tokio_util::compat::Compat;

pub struct WSConn {
  tx: Option<SenderSk<Compat<TcpStream>>>,
  rx: Option<ReceiverSk<Compat<TcpStream>>>,
}

impl WSConn {
  pub fn new_empty() -> Self {
    WSConn { tx: None, rx: None }
  }

  pub fn init(&mut self, tx: SenderSk<Compat<TcpStream>>, rx: ReceiverSk<Compat<TcpStream>>) {
    self.tx = Some(tx);
    self.rx = Some(rx);
  }

  pub async fn send(&mut self, msg: &str) -> Result<(), Box<dyn Error>> {
    self
      .tx
      .as_mut()
      .expect("Can't send messages because the connection was not initialised")
      .send_binary(msg)
      .await?;
    Ok(())
  }

  pub async fn recv(&mut self, buffer: &mut Vec<u8>) -> Result<(), Box<dyn Error>> {
    self
      .rx
      .as_mut()
      .expect("Can't receive messages because the connection was not initialised")
      .receive_data(buffer)
      .await?;
    Ok(())
  }

  pub async fn close(&mut self) -> Result<(), Box<dyn Error>> {
    self
      .tx
      .as_mut()
      .expect("Can't close connection because it wasn't initialised")
      .close()
      .await?;
    Ok(())
  }
}
