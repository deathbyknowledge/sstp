use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum Message {
    Send(SendMessage),
    Get(GetMessage),
    Ready,
    Content(ContentMessage),
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

    pub fn new_content(filename: String, content: Vec<u8>) -> ContentMessage {
        ContentMessage { filename, content }
    }
}

