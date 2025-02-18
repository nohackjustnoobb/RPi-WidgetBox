mod handler;
mod logger;
mod plugin;

use std::{fs, path::Path};

use log::{error, info};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use ws::{
    listen, CloseCode, Handler, Handshake, Message as WSMessage, Request, Response, Result, Sender,
};

use logger::setup_logger;

fn _data_default() -> Value {
    Value::Null
}

#[derive(Serialize, Deserialize)]
struct Message {
    #[serde(rename = "type")]
    type_: MessageType,
    #[serde(default = "_data_default")]
    data: Value,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum MessageType {
    AddPlugin,
    RemovePlugin,
    ConfigPlugin,
    ListPlugins,
    Error,
    Broadcast,
    #[serde(untagged)]
    Unknown(String),
}

impl Message {
    pub fn error(msg: &str) -> Self {
        Message {
            type_: MessageType::Error,
            data: Value::String(msg.to_string()),
        }
    }
}

struct Server {
    out: Sender,
    ip_addr: Option<String>,
}

impl Server {
    fn new(out: Sender) -> Self {
        Server { out, ip_addr: None }
    }

    fn send(&self, msg: Message) -> Result<()> {
        self.out
            .send(WSMessage::Text(serde_json::to_string(&msg).unwrap()))
    }

    fn broadcast(&self, msg: Message) -> Result<()> {
        self.out
            .broadcast(WSMessage::Text(serde_json::to_string(&msg).unwrap()))
    }
}

fn try_find_plugin(path: &str) -> Result<Response> {
    let re = Regex::new("/script/(.*)\\.js").unwrap();
    let not_found: std::result::Result<Response, _> =
        Ok(Response::new(404, "Not Found", b"404 - Not Found".to_vec()));

    if let Some(caps) = re.captures(path) {
        let file_path = format!("plugins/{}/script.js", &caps[1]);

        if Path::new(&file_path).exists() {
            if let Ok(content) = fs::read_to_string(file_path) {
                let mut resp = Response::new(200, "OK", content.as_bytes().to_vec());
                resp.headers_mut()
                    .push(("Content-Type".into(), b"text/javascript".to_vec()));

                return Ok(resp);
            }
        }
    }

    not_found
}

impl Handler for Server {
    fn on_request(&mut self, req: &Request) -> Result<Response> {
        match req.resource() {
            "/" => {
                if req.header("upgrade").is_some() {
                    Response::from_request(req)
                } else {
                    Ok(Response::new(200, "OK", b"TODO: client".to_vec()))
                }
            }
            "/edit" => Ok(Response::new(200, "OK", b"TODO: edit".to_vec())),
            _ => try_find_plugin(req.resource()),
        }
    }

    fn on_open(&mut self, shake: Handshake) -> Result<()> {
        if let Some(ip_addr) = shake.remote_addr()? {
            self.ip_addr = Some(ip_addr.clone());
            info!("Connection opened from {}.", ip_addr)
        } else {
            error!("Unable to obtain client's IP address.")
        }

        Ok(())
    }

    fn on_close(&mut self, _: CloseCode, _: &str) {
        info!("Connection closed from {}.", self.ip_addr.clone().unwrap());
    }

    fn on_message(&mut self, mesg: WSMessage) -> Result<()> {
        let text = match mesg {
            WSMessage::Text(t) => t,
            _ => return self.send(Message::error("Binary format is not supported.")),
        };

        let json = match serde_json::from_str::<Message>(&text) {
            Ok(j) => j,
            Err(_) => return self.send(Message::error("Failed to parse message.")),
        };

        self.handler(json.type_, json.data)
    }
}

fn main() {
    setup_logger().expect("Failed to initialize logger");

    if let Err(error) = listen("0.0.0.0:3012", |out| Server::new(out)) {
        error!("Failed to create WebSocket due to {:?}", error);
    }
}
