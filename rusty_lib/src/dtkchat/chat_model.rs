use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct DtkUser {
    pub id: String,
    pub email: String,
    pub name: String,
    pub lvl: Vec<String>,
    pub token: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChatForUsers {
    pub count: u32,
    pub id: String,
    pub chat: Vec<DtkChat>,
    pub users: Vec<DtkChatUser>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DtkChatUser {
    pub id: String,
    pub name: String,
    pub email: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DtkChatMessage {
    pub sender_id: String,
    pub date: String,
    pub message: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DtkChat {
    pub channel_id: String,
    pub last_update: String,
    pub users: Vec<DtkChatUser>,
    pub messages: Vec<DtkChatMessage>,
}

impl DtkChat {
    pub fn new(id: String) -> DtkChat {
        DtkChat {
            channel_id: id,
            last_update: chrono::Utc::now().to_string(),
            users: Vec::new(),
            messages: Vec::new(),
        }
    }

    pub fn add_user(&mut self, user: DtkChatUser) {
        self.users.push(user);
    }

    pub fn add_message(&mut self, message: DtkChatMessage) {
        self.messages.push(message);
    }
}