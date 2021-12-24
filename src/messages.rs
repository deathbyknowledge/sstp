use serde::{Deserialize, Serialize};

/*
 * STEPS OF TRANSMITION (I'm not even mentioning encryption yet)
 * 1. Sender --> SendMessage (Starts process sending the Filename they want to send.
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
  Ready,
  Send(SendMessage),
}

#[derive(Serialize, Deserialize)]
pub struct SendMessage {
  pub filename: String,
  pub size: usize,
}

#[derive(Serialize, Deserialize)]
pub struct GetMessage {
  pub code: String,
}

#[derive(Serialize, Deserialize)]
pub struct ContentMessage {
  pub filename: String,
  pub content: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
pub struct ApproveReqMessage {
  pub filename: String,
  pub size: usize,
}

#[derive(Serialize, Deserialize)]
pub struct ApproveResMessage {
  pub approved: bool,
}

#[derive(Serialize, Deserialize)]
pub struct ErrorMessage {
  pub text: String,
}

impl Message {
  pub fn new_send(filename: String, size: usize) -> Self {
    Message::Send(SendMessage { filename, size })
  }

  pub fn new_approve_req(filename: String, size: usize) -> String {
    let req = Message::ApproveReq(ApproveReqMessage { filename, size });
    serde_json::to_string(&req).expect("Couldn't parse message.")
  }

  pub fn new_approve_res(approved: bool) -> String {
    let req = Message::ApproveRes(ApproveResMessage { approved });
    serde_json::to_string(&req).expect("Couldn't parse message.")
  }

  pub fn new_get(code: String) -> Self {
    Message::Get(GetMessage { code })
  }

  pub fn new_ready() -> Self {
    Message::Ready
  }

  pub fn new_content(filename: String, content: Vec<u8>) -> ContentMessage {
    ContentMessage { filename, content }
  }

  pub fn new_error(text: String) -> Self {
    Message::Error(ErrorMessage { text })
  }
}
