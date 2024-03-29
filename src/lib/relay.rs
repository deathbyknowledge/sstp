use std::collections::HashMap;
use std::error::Error;
use std::net::SocketAddr;
use std::str;
use std::sync::Arc;
use std::time::Instant;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use tokio_stream::{wrappers::TcpListenerStream, StreamExt};
use tokio_util::compat::Compat;

use crate::messages::{Message, SendMessage};
use crate::utils::{calc_chunks, start_ws_handshake};

pub struct Relay {}

struct ClientInfo {
  tx: soketto::Sender<Compat<TcpStream>>,
  rx: soketto::Receiver<Compat<TcpStream>>,
  addr: SocketAddr,
}

struct RoomInfo {
  sender: ClientInfo,
  filename: String,
  opened: Instant,
  size: u64,
}

impl RoomInfo {
  async fn get_approval(&mut self, client: &mut ClientInfo) -> Result<bool, Box<dyn Error>> {
    let approve_req =
      Message::new_approve_req(self.filename.to_string(), self.size, self.sender.addr);
    client.tx.send_text(approve_req).await?;
    let mut data = Vec::new();
    client.rx.receive_data(&mut data).await?;
    let res = serde_json::from_slice(&data)?;
    if let Message::ApproveRes(res) = res {
      Ok(res.approved)
    } else {
      unreachable!();
    }
  }
}

impl Relay {
  pub async fn start() -> Result<(), Box<dyn Error>> {
    println!("Starting Relay Server...");
    let listener = TcpListener::bind("0.0.0.0:8004").await?;
    let mut incoming = TcpListenerStream::new(listener);
    let rooms = Arc::new(Mutex::new(HashMap::new()));

    let cleanup = rooms.clone();
    tokio::spawn(async move {
      Relay::start_cleanup(cleanup).await;
    });

    while let Some(socket) = incoming.next().await {
      let rooms = rooms.clone();
      tokio::spawn(async move {
        Relay::process_req(socket.unwrap(), rooms)
          .await
          .expect("Error when processing request");
      });
    }
    Ok(())
  }

  async fn start_cleanup(rooms: Arc<Mutex<HashMap<String, Arc<Mutex<RoomInfo>>>>>) {
    loop {
      sleep(Duration::new(3600, 0)).await;
      let mut rooms = rooms.lock().await;
      rooms.retain(move |_, v| {
        let room = v.blocking_lock();
        room.opened.elapsed() < Duration::new(3600, 0)
      });
    }
  }

  async fn create_room(
    client: ClientInfo,
    message: SendMessage,
    rooms: Arc<Mutex<HashMap<String, Arc<Mutex<RoomInfo>>>>>,
  ) -> Result<(), Box<dyn Error>> {
    let code = message.code;
    let filename = message.filename;
    let size = message.size;
    let room = Arc::new(Mutex::new(RoomInfo {
      sender: client,
      filename,
      size,
      opened: Instant::now(),
    }));
    let mut rooms = rooms.lock().await;
    rooms.insert(code.to_string(), room);
    Ok(())
  }

  async fn process_req(
    socket: TcpStream,
    rooms: Arc<Mutex<HashMap<String, Arc<Mutex<RoomInfo>>>>>,
  ) -> Result<(), Box<dyn Error>> {
    let addr = socket.peer_addr()?;
    let (sender, receiver) = start_ws_handshake(socket).await?;
    let mut client = ClientInfo {
      tx: sender,
      rx: receiver,
      addr,
    };
    let mut data = Vec::new();
    client.rx.receive_data(&mut data).await?;
    let message: Message = serde_json::from_str(str::from_utf8(&data).unwrap())?;

    match message {
      Message::Send(message) => Relay::create_room(client, message, rooms).await?,
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
            let approved = room.get_approval(&mut client).await?;
            if !approved {
              return Ok(());
            }

            // Send Ready message to start the file transfer from the sending client
            let ready_message = Message::new_ready(client.addr);
            room.sender.tx.send_text(ready_message).await?;
            let mut data = Vec::with_capacity(1_000_000);
            for _ in 0..calc_chunks(room.size) {
              room.sender.rx.receive_data(&mut data).await?;
              client.tx.send_binary_mut(&mut data).await?;
              data.clear();
            }
            let mut rooms = rooms.lock().await;
            rooms.remove(&code).unwrap();
          }

          None => {
            let error = Message::new_error(String::from("Room code does not exist"));
            client.tx.send_text(error).await?;
          }
        };
      }
      _ => unreachable!(),
    };
    Ok(())
  }
}