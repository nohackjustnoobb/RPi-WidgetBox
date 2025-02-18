use serde_json::Value;
use ws::Result;

use crate::{Message, MessageType, Server};

impl Server {
    pub fn handler(&self, type_: MessageType, data: Value) -> Result<()> {
        match type_ {
            MessageType::ListPlugins => self.list_plugins(),
            MessageType::AddPlugin => self.add_plugin(data),
            MessageType::RemovePlugin => self.remove_plugin(data),
            MessageType::ConfigPlugin => self.config_plugin(data),
            MessageType::Broadcast => {
                let mesg = match data.as_str() {
                    Some(s) => s,
                    None => {
                        return self.send(Message::error("Failed to parse the broadcast message."))
                    }
                };

                self.out.broadcast(mesg)
            }
            _ => self.send(Message::error("Unsupported type.")),
        }
    }
}
