use std::collections::HashMap;
use std::error::Error;
use std::str;
use std::time::Instant;
use tokio::net::{TcpListener, TcpStream};
use tokio_stream::{wrappers::TcpListenerStream, StreamExt};
use tokio_util::compat::{Compat};

use crate::messages::Message;
use crate::utils::{start_ws_handshake};

pub struct Relay {
  rooms: HashMap<String, RoomInfo>,
}

struct RoomInfo {
  sender: ClientInfo,
  receiver: Option<ClientInfo>,
  filename: String,
  opened: Instant,
  size: usize,
}

struct ClientInfo {
  tx: soketto::Sender<Compat<TcpStream>>,
  rx: soketto::Receiver<Compat<TcpStream>>,
}

impl Relay {
  pub fn new() -> Self {
    Relay {
      rooms: HashMap::new(),
    }
  }

  pub async fn start(&mut self) -> Result<(), Box<dyn Error>> {
    println!("Starting Relay Server...");
    let listener = TcpListener::bind("0.0.0.0:8004").await?;
    let mut incoming = TcpListenerStream::new(listener);

    while let Some(socket) = incoming.next().await {
      let (mut sender, mut receiver) = start_ws_handshake(socket?).await?;

      let mut data = Vec::new();
      let _data_type = receiver.receive_data(&mut data).await?;
      let message: Message = serde_json::from_str(str::from_utf8(&data).unwrap())?;

      match message {
        Message::Send(message) => {
          let code = "my-room";
          let filename = message.filename;
          let size = message.size;
          let room = RoomInfo {
            sender: ClientInfo {
              tx: sender,
              rx: receiver,
            },
            receiver: None,
            filename,
            size,
            opened: Instant::now(),
          };
          self.rooms.insert(code.to_string(), room);
        }
        Message::Get(message) => {
          let code = message.code;
          match self.rooms.get_mut(&code) {
            Some(room) => {
              // Send Approval request for this file
              let approve_req = Message::new_approve_req(room.filename.to_string(), room.size);
              sender.send_text(approve_req).await?;
              let mut data = Vec::new();
              room.sender.rx.receive_data(&mut data).await?;
              let res_message: Message = serde_json::from_slice(&data)?;
              if let Message::ApproveRes(res) = res_message {
                if !res.approved {
                  continue;
                }
              }

              // Send Ready message to start the file transfer from the sending client
              let ready_message = Message::new_ready();
              room
                .sender
                .tx
                .send_text(serde_json::to_string(&ready_message)?)
                .await?;
              let mut data = Vec::new();
              room.sender.rx.receive_data(&mut data).await?;
              sender.send_binary(&data).await?;
              self.rooms.remove(&code).unwrap();
            }
            None => {
              let error_message = Message::new_error(String::from("Room code does not exist"));
              sender
                .send_text(serde_json::to_string(&error_message)?)
                .await?;
            }
          };
        }
        _ => {}
      };
    }
    Ok(())
  }
}
