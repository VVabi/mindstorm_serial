use std::error::Error;
extern crate serde_json;
extern crate serde;

pub struct MqttStrMessage {
    pub topic: std::string::String,
    pub payload: std::string::String
}

pub trait Messenger {
    fn publish_message(self: &Self, m: &dyn Message) -> Result<(), Box<dyn Error>>;
    fn receive_messages(self: &mut Self) -> Vec<MqttStrMessage>;
}

pub trait Message {
    fn to_json(&self) -> std::result::Result<std::string::String, serde_json::Error>;
    fn get_topic_dyn(&self) -> std::string::String;
}
   