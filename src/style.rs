use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

use crate::{Message, MessageType, Server};

use reqwest::blocking::get;
use serde::{Deserialize, Serialize};
use serde_json::{from_value, json, Value};
use ws::Result;

#[derive(Clone, Serialize, Deserialize)]
pub struct Style {
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    inline: Option<String>,
}

impl Server {
    pub fn remove_style(&self) -> Result<()> {
        if Path::new("data/style.css").exists() {
            if fs::remove_file("data/style.css").is_ok() {
                return self.broadcast(Message {
                    type_: MessageType::RemoveStyle,
                    data: Value::Null,
                });
            }
        }

        self.send(Message::error("Failed to remove style."))
    }

    pub fn get_style(&self) -> Result<()> {
        let file_path = "data/style.css";

        self.send(Message {
            type_: MessageType::GetStyle,
            data: json!(Style {
                inline: None,
                url: if Path::new(file_path).exists() {
                    Some("/custom/style.css".to_string())
                } else {
                    None
                }
            }),
        })
    }

    pub fn set_style(&self, data: Value) -> Result<()> {
        let parsed = match from_value::<Style>(data) {
            Ok(p) => p,
            Err(_) => return self.send(Message::error("Failed to parse data.")),
        };

        if parsed.inline.is_none() && parsed.url.is_none() {
            return self.send(Message::error("Failed to get style."));
        }

        let style = match parsed.inline {
            Some(s) => s,
            None => {
                let url = parsed.url.unwrap();
                let resp = match get(url) {
                    Ok(r) => r,
                    Err(_) => return self.send(Message::error("Failed to get style.")),
                };
                match resp.text() {
                    Ok(t) => t,
                    Err(_) => return self.send(Message::error("Failed to get style.")),
                }
            }
        };

        let mut file = match File::create("data/style.css") {
            Ok(f) => f,
            Err(_) => return self.send(Message::error("Failed to open style file.")),
        };

        if file.write_all(style.as_bytes()).is_err() {
            return self.send(Message::error("Failed to write style."));
        }

        return self.broadcast(Message {
            type_: MessageType::SetStyle,
            data: json!(Style {
                url: Some("/custom/style.css".to_string()),
                inline: None,
            }),
        });
    }
}
