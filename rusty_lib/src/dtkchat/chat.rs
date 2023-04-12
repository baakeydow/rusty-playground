/// RUSTY chat by baakeydow
use std::io::{Read, Write};

use base64::{engine::general_purpose, Engine};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use futures::StreamExt;
use mongodb::{
    bson::{self, doc, to_document, Binary, Bson, Document},
    options::FindOptions,
};

use crate::{
    dtkchat::chat_utils::{get_chat_collection_name, get_chat_db_name},
    dtkmongo::dtk_connect::{get_dtkmongo_client, get_mongodb_main_db, get_mongodb_uri},
};

use super::chat_model::{DtkChat, DtkChatMessage, DtkChatUser};

fn gzip_text(text: &str) -> Vec<u8> {
    let mut gz = GzEncoder::new(Vec::new(), Compression::default());
    gz.write_all(text.as_bytes()).unwrap();
    gz.finish().unwrap()
}

fn get_str_from_binary_string(binary_string: &str) -> Result<String, &'static str> {
    let binary_prefix = "Binary(0x2, ";
    if !binary_string.starts_with(binary_prefix) {
        return Err("Input string is not a valid binary string");
    }
    Ok(binary_string[binary_prefix.len()..binary_string.len() - 1].to_owned())
}

pub async fn get_all_chat_users() -> Vec<DtkChatUser> {
    let client = get_dtkmongo_client(get_mongodb_uri().as_str()).await;
    let coll = client.database(&get_mongodb_main_db()).collection::<Document>("users");
    let mut cursor = coll.find(doc! {}, None).await.unwrap();
    let mut res: Vec<DtkChatUser> = vec![];
    while let Some(doc) = cursor.next().await {
        let doc = doc.unwrap();
        let user = DtkChatUser {
            id: doc.get("_id").unwrap().as_object_id().unwrap().to_string(),
            name: doc.get("name").unwrap().as_str().unwrap().to_string(),
            email: doc.get("email").unwrap().as_str().unwrap().to_string(),
        };
        res.extend([user]);
    }
    res
}

pub async fn get_all_dtk_chat_for_user(user: DtkChatUser) -> Vec<DtkChat> {
    let user_id = user.id.clone();
    println!("User id: {}", user_id);
    let client = get_dtkmongo_client(get_mongodb_uri().as_str()).await;
    let coll = client
        .database(&get_chat_db_name())
        .collection::<Document>(get_chat_collection_name().as_str());
    let filter = doc! { "users.id":  user_id.clone()  };
    let options = FindOptions::builder()
        .sort(mongodb::bson::doc! {"last_update": -1})
        .build();
    let cursor = coll.find(filter, options).await.unwrap();
    let mut dtk_chat_data: Vec<DtkChat> = cursor
        .map(|res| {
            let chat_doc = res.clone().unwrap();
            let doc_messages = chat_doc.get("messages").unwrap().as_array().unwrap().to_vec();
            let uncompressed_messages = doc_messages
                .into_iter()
                .map(|msg| {
                    // println!("msg: {:#?}", msg);
                    let test_message = msg.as_document().unwrap().get("message").unwrap();
                    let compressed_data = general_purpose::STANDARD
                        .decode(&get_str_from_binary_string(test_message.as_str().unwrap()).unwrap())
                        .map_err(|e| format!("{}", e))
                        .unwrap();
                    let mut decoder = GzDecoder::new(&compressed_data[..]);
                    let mut decompressed_data = vec![];
                    decoder.read_to_end(&mut decompressed_data).unwrap();

                    let decompressed_text = String::from_utf8(decompressed_data).unwrap();
                    let sender_id = msg.as_document().unwrap().get("sender_id").unwrap().as_str().unwrap();
                    let date = msg.as_document().unwrap().get("date").unwrap().as_str().unwrap();
                    DtkChatMessage {
                        message: decompressed_text,
                        sender_id: sender_id.to_string(),
                        date: date.to_string(),
                    }
                })
                .collect::<Vec<DtkChatMessage>>();
            let mut data: DtkChat = bson::from_bson(bson::Bson::Document(res.unwrap())).unwrap();
            data.messages = uncompressed_messages;
            data
        })
        .collect::<Vec<DtkChat>>()
        .await;
    if dtk_chat_data.is_empty() {
        log::info!("No Chat not found, creating new chat");
        let mut new_chat = DtkChat::new(user_id);
        new_chat.add_user(user);
        return [new_chat].to_vec();
    } else if dtk_chat_data
        .clone()
        .into_iter()
        .filter(|chat| chat.clone().users.into_iter().filter(|u| u.id != user_id).count() == 0)
        .count()
        == 0
    {
        log::info!("User Chat not found, creating new chat");
        let mut new_chat = DtkChat::new(user_id);
        new_chat.add_user(user);
        dtk_chat_data.push(new_chat);
    }
    println!("Chats found {}", dtk_chat_data.len());
    dtk_chat_data
}

pub async fn create_dtk_chat_message(dtk_chat: DtkChat) {
    let client = get_dtkmongo_client(get_mongodb_uri().as_str()).await;
    let coll = client
        .database(&get_chat_db_name())
        .collection::<Document>(get_chat_collection_name().as_str());
    let filter = doc! { "channel_id": dtk_chat.channel_id.clone() };
    let chat_doc = to_document(&dtk_chat).unwrap();
    let messages = chat_doc.get("messages").unwrap().as_array().unwrap().to_vec();
    let users = chat_doc.get("users").unwrap().as_array().unwrap().to_vec();
    let compressed_messages = messages
        .into_iter()
        .map(|m| {
            let sender_id = m.as_document().unwrap().get("sender_id").unwrap().as_str().unwrap();
            let date = m.as_document().unwrap().get("date").unwrap().as_str().unwrap();
            let message = m.as_document().unwrap().get("message").unwrap().as_str().unwrap();
            let compressed_text = gzip_text(message);
            let bson = Bson::Binary(Binary {
                bytes: compressed_text,
                subtype: mongodb::bson::spec::BinarySubtype::BinaryOld,
            });
            mongodb::bson::to_bson(&DtkChatMessage {
                sender_id: sender_id.to_string(),
                date: date.to_string(),
                message: bson.to_string(),
            })
            .unwrap()
        })
        .collect::<Vec<Bson>>();
    let update = doc! { "$set": {"last_update": dtk_chat.last_update}, "$addToSet": { "users": { "$each": users }, "messages": { "$each": compressed_messages } } };
    let options = mongodb::options::UpdateOptions::builder().upsert(true).build();
    coll.update_one(filter, update, Some(options))
        .await
        .expect("Failed to insert chat data");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let text = "Binary(0x2, H4sIAAAAAAAA/0tMTUkrSwUAABkbnwYAAAA=)";
        let compressed_data = general_purpose::STANDARD
            .decode(get_str_from_binary_string(text).unwrap())
            .map_err(|e| format!("{}", e))
            .unwrap();

        let mut decoder = GzDecoder::new(&compressed_data[..]);
        let mut decompressed_data = vec![];
        decoder.read_to_end(&mut decompressed_data).unwrap();

        let decompressed_text = String::from_utf8(decompressed_data).unwrap();
        assert_eq!(decompressed_text, "aedfve");
    }

    #[test]
    fn test_chat() {
        let mut chat = DtkChat::new("id1-id2".to_string());
        chat.add_user(DtkChatUser {
            id: "1".to_string(),
            email: "bl@".to_string(),
            name: "User 1".to_string(),
        });
        chat.add_user(DtkChatUser {
            id: "2".to_string(),
            email: "qw@".to_string(),
            name: "User 2".to_string(),
        });
        chat.add_message(DtkChatMessage {
            sender_id: "User 1".to_string(),
            date: chrono::Utc::now().to_string(),
            message: "How are you doing?".to_string(),
        });
        chat.add_message(DtkChatMessage {
            sender_id: "User 2".to_string(),
            date: chrono::Utc::now().to_string(),
            message: "I'm doing well, thanks. How about you?".to_string(),
        });
        println!("Chat: {:#?}", chat);
        assert_eq!(chat.users.len(), 2);
        assert_eq!(chat.messages.len(), 2);
    }
}
