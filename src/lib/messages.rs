use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;
use std::net::SocketAddr;

/*
 * STEPS OF TRANSMITION (I'm not even mentioning encryption yet)
 * 1. Sender --> SendMessage (Starts process sending the Filename they want to send)
 * 2. Relay <-- SendMessage (Creates RoomInfo and waits for GetMessage)
 * 3. Receiver --> GetMessage (Sends RoomInfo code it wants to read from)
 * 4. Relay <-- GetMessage (Looks up code in the RoomInfo map)
 * 5. ** LOOK UP MISS ** Relay --> ErrorMessage (RoomInfo NotFound)
 * 5. ** LOOK UP HIT ** Relay --> ApprovalReqMessage (Sends File metadata for approval)
 * 6. Receiver <-- ErrorMessage (Reports error and exits program)
 * 6. Receiver <-- ApprovalReqMessage (Displays file metadata and asks for transfer approval)
 * 7. Receiver --> ApprovalResMessage (yes/no. If no, exit program)
 * ** DENIED ** Relay <-- ApprovalResMessage (Ignores)
 * ** APPROVED ** Relay <-- ApprovalResMessage (Proceed with process)
 * Relay --> ReadyMessage (Signal the Sender that it can start sending data)
 * Sender <-- ReadyMessage
 * Sender --> ContentMessage (Several messages sent with Chunking)
 * Relay <-- ContentMessage (Relays all ContentMessages to Receiver)
 */

#[derive(Serialize, Deserialize)]
pub enum Message {
  ApproveRes(ApproveResMessage),
  ApproveReq(ApproveReqMessage),
  Content(ContentMessage),
  Error(ErrorMessage),
  Get(GetMessage),
  Ready(ReadyMessage),
  Send(SendMessage),
}

#[derive(Serialize, Deserialize)]
pub struct SendMessage {
  pub filename: String,
  pub size: u64,
  pub code: String,
}

#[derive(Serialize, Deserialize)]
pub struct GetMessage {
  pub code: String,
}

#[derive(Serialize, Deserialize)]
pub struct ContentMessage {
  pub content: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
pub struct ReadyMessage {
  pub addr: SocketAddr,
}

#[derive(Serialize, Deserialize)]
pub struct ApproveReqMessage {
  pub filename: String,
  pub size: u64,
  pub addr: SocketAddr,
}

#[derive(Serialize, Deserialize)]
pub struct ApproveResMessage {
  pub approved: bool,
}

#[derive(Serialize, Deserialize)]
pub struct ErrorMessage {
  pub text: String,
}

impl Error for ErrorMessage {}

impl fmt::Display for ErrorMessage {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", self.text)
  }
}

impl fmt::Debug for ErrorMessage {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", self.text)
  }
}

impl Message {
  pub fn new_send(filename: String, size: u64, code: String) -> String {
    let msg = Message::Send(SendMessage {
      filename,
      size,
      code,
    });
    serde_json::to_string(&msg).expect("Couldn't parse message.")
  }

  pub fn new_approve_req(filename: String, size: u64, addr: SocketAddr) -> String {
    let msg = Message::ApproveReq(ApproveReqMessage {
      filename,
      size,
      addr,
    });
    serde_json::to_string(&msg).expect("Couldn't parse message.")
  }

  pub fn new_approve_res(approved: bool) -> String {
    let msg = Message::ApproveRes(ApproveResMessage { approved });
    serde_json::to_string(&msg).expect("Couldn't parse message.")
  }

  pub fn new_get(code: &str) -> String {
    let msg = Message::Get(GetMessage {
      code: code.to_string(),
    });
    serde_json::to_string(&msg).expect("Couldn't parse message.")
  }

  pub fn new_ready(addr: SocketAddr) -> String {
    let msg = Message::Ready(ReadyMessage { addr });
    serde_json::to_string(&msg).expect("Couldn't parse message.")
  }

  pub fn new_content(content: Vec<u8>) -> String {
    let msg = ContentMessage { content };
    serde_json::to_string(&msg).expect("Couldn't parse message.")
  }

  pub fn new_error(text: String) -> String {
    let msg = Message::Error(ErrorMessage { text });
    serde_json::to_string(&msg).expect("Couldn't parse message.")
  }
}
