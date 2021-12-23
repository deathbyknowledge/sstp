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
 *
 */

#[derive(Serialize, Deserialize)]
pub enum Message {
    Send(SendMessage),
    Get(GetMessage),
    Ready,
    Content(ContentMessage),
    Error(ErrorMessage),
}

#[derive(Serialize, Deserialize)]
pub struct SendMessage {
    pub filename: String,
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
pub struct ApprovalMessage {
    pub filename: String,
    pub size: f32,
}

#[derive(Serialize, Deserialize)]
pub struct ErrorMessage {
    pub text: String,
}


impl Message {
    
    pub fn new_send(filename: String) -> Self {
        Message::Send(SendMessage { filename })
    }

    pub fn new_get(code: String) -> Self {
        Message::Get(GetMessage { code })
    }

    pub fn new_ready() -> Self {
        Message::Ready
    }

    pub fn new_error(text: String) -> Self {
        Message::Error(ErrorMessage { text } )
    }

    pub fn new_content(filename: String, content: Vec<u8>) -> ContentMessage {
        ContentMessage { filename, content }
    }
}

