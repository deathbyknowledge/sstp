use parity_wordlist::random_phrase;
use soketto::connection::{Receiver as ReceiverSk, Sender as SenderSk};
use soketto::handshake::server::Response;
use soketto::handshake::{Client, ServerResponse};
use std::error::Error;
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio_util::compat::{Compat, TokioAsyncReadCompatExt};

const PUBLIC_RELAY: &str = "138.68.103.243:8004";

// CONNECTIONS
pub async fn start_ws_conn(
  relay_addr: SocketAddr,
) -> Result<(SenderSk<Compat<TcpStream>>, ReceiverSk<Compat<TcpStream>>), Box<dyn Error>> {
  let socket = TcpStream::connect(relay_addr).await?;
  let str_addr = relay_addr.clone().to_string();
  let mut client = Client::new(socket.compat(), &str_addr, "/");

  let (sender, receiver) = match client.handshake().await? {
    ServerResponse::Accepted { .. } => client.into_builder().finish(),
    ServerResponse::Redirect { .. } => unimplemented!("f"),
    ServerResponse::Rejected { .. } => unimplemented!("f"),
  };

  Ok((sender, receiver))
}

pub async fn start_ws_handshake(
  stream: TcpStream,
) -> Result<(SenderSk<Compat<TcpStream>>, ReceiverSk<Compat<TcpStream>>), Box<dyn Error>> {
  let mut server = soketto::handshake::Server::new(stream.compat());
  let websocket_key = {
    let req = server.receive_request().await?;
    req.key()
  };
  let accept = Response::Accept {
    key: websocket_key,
    protocol: None,
  };
  server.send_response(&accept).await?;

  let (sender, receiver) = server.into_builder().finish();
  Ok((sender, receiver))
}

// Rand
pub fn gen_room_key() -> String {
  let phrase = random_phrase(3);
  str::replace(&phrase, " ", "-")
}

// Format
pub fn calc_chunks(size: u64) -> u64 {
  ((size as f32 + 1_000_000.0 - 1.0) / 1_000_000.0) as u64
}

// Validation
pub fn parse_relay_addr(addr: Option<&str>) -> SocketAddr {
  if let Some(addr) = addr {
    let addr = addr.parse().expect("Couldn't parse the provided address");
    addr
  } else {
    PUBLIC_RELAY.parse().unwrap()
  }
}
