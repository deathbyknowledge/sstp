use parity_wordlist::random_phrase;
use soketto::connection::{Receiver as ReceiverSk, Sender as SenderSk};
use soketto::handshake::server::Response;
use soketto::handshake::{Client, ServerResponse};
use std::error::Error;
use tokio::net::TcpStream;
use tokio_util::compat::{Compat, TokioAsyncReadCompatExt};

// CONNECTIONS
pub async fn start_ws_conn(
) -> Result<(SenderSk<Compat<TcpStream>>, ReceiverSk<Compat<TcpStream>>), Box<dyn Error>> {
  let socket = TcpStream::connect("138.68.103.243:8004").await?;

  let mut client = Client::new(socket.compat(), "138.68.103.243:8004", "/");

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

// STDOUT/STDIN
fn read_input() -> String {
  use std::io::{stdin, stdout, Write};
  let mut s = String::new();
  let _ = stdout().flush();
  stdin()
    .read_line(&mut s)
    .expect("Did not enter a correct string");
  if let Some('\n') = s.chars().next_back() {
    s.pop();
  }
  if let Some('\r') = s.chars().next_back() {
    s.pop();
  }
  s
}

pub fn req_keyboard_approval() -> bool {
  let mut input = read_input();
  while input != "y" && input != "n" {
    println!("Please submit only 'y' or 'n'.");
    input = read_input();
  }
  let approved = input.eq("y");
  approved
}

// Rand
pub fn gen_room_key() -> String {
  let phrase = random_phrase(4);
  str::replace(&phrase, " ", "-")
}

// Format
pub fn calc_chunks(size: usize) -> usize {
  ((size as f32 + 1_000_000.0 - 1.0) / 1_000_000.0) as usize
}

pub fn split_vec(mut vec: Vec<u8>) -> Vec<Vec<u8>> {
  let chunks = calc_chunks(vec.len());
  let mut buffer = Vec::new();
  for _ in 0..chunks {
    buffer.insert(0, vec.split_off(1_000_000));
  }
  buffer
}
