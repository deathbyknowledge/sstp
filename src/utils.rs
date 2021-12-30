use bytesize::to_string;
use dialoguer::{console, Confirm};
use indicatif::{ProgressBar, ProgressStyle};
use parity_wordlist::random_phrase;
use soketto::connection::{Receiver as ReceiverSk, Sender as SenderSk};
use soketto::handshake::server::Response;
use soketto::handshake::{Client, ServerResponse};
use std::error::Error;
use std::net::SocketAddr;
use std::path::Path;
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

// STDOUT/STDIN
pub fn req_keyboard_approval(filename: String, size: usize) -> bool {
  let output = format!(
    "Accept {} ({})?",
    filename,
    to_string(
      size.try_into().expect("Error when parsing usize to u64"),
      false
    )
  );
  let approved = Confirm::new()
    .with_prompt(output)
    .interact()
    .expect("Error when requesting input");
  let term = console::Term::stdout();
  term
    .clear_last_lines(1)
    .expect("Could not clear terminal line");
  approved
}

pub fn create_pb(size: usize) -> ProgressBar {
  let bar = ProgressBar::new(size.try_into().expect("Error when parsing usize to u64"));
  bar.set_style(ProgressStyle::default_bar()
    .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
    .progress_chars("#>-"));
  bar
}

// Rand
pub fn gen_room_key() -> String {
  let phrase = random_phrase(3);
  str::replace(&phrase, " ", "-")
}

// Format
pub fn calc_chunks(size: usize) -> usize {
  ((size as f32 + 1_000_000.0 - 1.0) / 1_000_000.0) as usize
}

// Validation
pub fn validate_filepath(filepath: &String) -> String {
  let path = Path::new(&filepath);
  if !path.is_file() {
    panic!("File does not exist.");
  }
  path
    .file_name()
    .expect("Coudln't get filename")
    .to_str()
    .expect("Errored when parsing OsStr")
    .to_string()
}

pub fn parse_relay_addr(addr: Option<&str>) -> SocketAddr {
  if let Some(addr) = addr {
    let addr = addr.parse().expect("Couldn't parse the provided address");
    addr
  } else {
    PUBLIC_RELAY.parse().unwrap()
  }
}
