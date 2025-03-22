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
            MessageType::PluginMessage => self.broadcast(Message {
                type_: MessageType::PluginMessage,
                data,
            }),
            MessageType::GetStyle => self.get_style(),
            MessageType::SetStyle => self.set_style(data),
            MessageType::RemoveStyle => self.remove_style(),
            _ => self.send(Message::error("Unsupported type.")),
        }
    }
}
