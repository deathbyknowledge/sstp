use std::collections::HashMap;
use std::error::Error;
use std::str;
use std::sync::Arc;
use std::time::Instant;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio_stream::{wrappers::TcpListenerStream, StreamExt};
use tokio_util::compat::Compat;

use crate::messages::Message;
use crate::utils::start_ws_handshake;

pub struct Relay {}

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
    Relay {}
  }

  pub async fn start(&mut self) -> Result<(), Box<dyn Error>> {
    println!("Starting Relay Server...");
    let listener = TcpListener::bind("0.0.0.0:8004").await?;
    let mut incoming = TcpListenerStream::new(listener);
    let rooms = Arc::new(Mutex::new(HashMap::new()));

    while let Some(socket) = incoming.next().await {
      let rooms = rooms.clone();
      tokio::spawn(async move {
        process_req(socket.unwrap(), rooms)
          .await
          .expect("Error when processing request");
      });
    }
    Ok(())
  }
}

async fn process_req(
  socket: TcpStream,
  rooms: Arc<Mutex<HashMap<String, Arc<Mutex<RoomInfo>>>>>,
) -> Result<(), Box<dyn Error>> {
  let (mut sender, mut receiver) = start_ws_handshake(socket).await?;

  let mut data = Vec::new();
  let _data_type = receiver.receive_data(&mut data).await?;
  let message: Message = serde_json::from_str(str::from_utf8(&data).unwrap())?;

  match message {
    Message::Send(message) => {
      let code = message.code;
      let filename = message.filename;
      let size = message.size;
      let room = Arc::new(Mutex::new(RoomInfo {
        sender: ClientInfo {
          tx: sender,
          rx: receiver,
        },
        receiver: None,
        filename,
        size,
        opened: Instant::now(),
      }));
      let mut rooms = rooms.lock().await;
      rooms.insert(code.to_string(), room);
    }

    Message::Get(message) => {
      let code = message.code;
      let room = {
        let mut rooms = rooms.lock().await;
        let room_res = rooms.get_mut(&code);
        if let Some(room) = room_res {
          Some(room.clone())
        } else {
          None
        }
      };
      match room {
        Some(room) => {
          // Send Approval request for this file
          let mut room = room.lock().await;
          let approve_req = Message::new_approve_req(room.filename.to_string(), room.size);
          sender.send_text(approve_req).await?;
          let mut data = Vec::new();
          receiver.receive_data(&mut data).await?;
          let res_message: Message = serde_json::from_slice(&data)?;
          if let Message::ApproveRes(res) = res_message {
            if !res.approved {
              return Ok(());
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
          let mut rooms = rooms.lock().await;
          rooms.remove(&code).unwrap();
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
  Ok(())
}
