use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum MessageType {
    Send,
    Get,
    Ready,
}

#[derive(Serialize, Deserialize)]
pub struct Message {
    pub message_type: MessageType,
    pub code: Option<String>,
    pub filename: Option<String>,
}

impl Message {
    
    pub fn new_send(filename: &str) -> Self {
        Message { message_type: MessageType::Send, code: None, filename: Some(filename.to_string()) }
    }

    pub fn new_get(code: &str) -> Self {
        Message { message_type: MessageType::Get, code: Some(code.to_string()), filename: None }
    }

    pub fn new_ready() -> Self {
        Message { message_type: MessageType::Ready, code: None, filename: None }
    }

    pub fn is_ready(&self) -> bool {
        let a = match self.message_type {
            MessageType::Ready => true,
            _ => false,
        };
        a
    }
}

