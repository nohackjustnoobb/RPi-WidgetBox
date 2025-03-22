mod handler;
mod logger;
mod plugin;
mod style;

use std::{
    fs,
    path::{Path, PathBuf},
};

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
    PluginMessage,
    SetStyle,
    RemoveStyle,
    GetStyle,
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

fn read_file_in_folder(folder: &str, filename: &str) -> Option<String> {
    let folder_path = Path::new(folder);

    if !folder_path.is_dir() {
        return None;
    }

    let file_path: PathBuf = folder_path.join(filename);

    if file_path.is_file() {
        match fs::read_to_string(&file_path) {
            Ok(contents) => Some(contents),
            Err(_) => None,
        }
    } else {
        None
    }
}

fn try_serve_static_file(folder: &str, filename: &str) -> Option<Response> {
    if let Some(content) = read_file_in_folder(folder, filename) {
        let extension = Path::new(filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        let content_type = match extension {
            "js" => b"text/javascript".to_vec(),
            "css" => b"text/css".to_vec(),
            _ => b"text/plain".to_vec(),
        };

        let mut response = Response::new(200, "OK", content.as_bytes().to_vec());
        response
            .headers_mut()
            .push(("Content-Type".into(), content_type));
        response
            .headers_mut()
            .push(("Access-Control-Allow-Origin".into(), "*".into()));

        return Some(response);
    }
    None
}

fn try_find_plugin_or_static(path: &str) -> Result<Response> {
    let not_found = Ok(Response::new(404, "Not Found", b"404 - Not Found".to_vec()));

    let style_re = Regex::new("custom/style.css").unwrap();
    if style_re.captures(path).is_some() {
        if let Some(response) = try_serve_static_file("data", "style.css") {
            return Ok(response);
        }
    }

    let plugin_re = Regex::new("/plugin/(.*)/(.*\\.js)").unwrap();
    if let Some(caps) = plugin_re.captures(path) {
        if let Some(response) =
            try_serve_static_file(format!("data/plugins/{}", &caps[1]).as_str(), &caps[2])
        {
            return Ok(response);
        }
    }

    let edit_re = Regex::new(r"/edit/(.*)").unwrap();
    if let Some(caps) = edit_re.captures(path) {
        if let Some(response) = try_serve_static_file("static/editor", &caps[1]) {
            return Ok(response);
        }
    }

    let display_re = Regex::new(r"/(.*)").unwrap();
    if let Some(caps) = display_re.captures(path) {
        if let Some(response) = try_serve_static_file("static/display", &caps[1]) {
            return Ok(response);
        }
    }

    not_found
}

fn read_html(folder: &str) -> Response {
    let mut resp = Response::new(
        200,
        "OK",
        read_file_in_folder(folder, "index.html")
            .unwrap()
            .as_bytes()
            .to_vec(),
    );

    resp.headers_mut()
        .push(("Content-Type".into(), b"text/html".to_vec()));

    return resp;
}

impl Handler for Server {
    fn on_request(&mut self, req: &Request) -> Result<Response> {
        match req.resource() {
            "/" => {
                if req.header("upgrade").is_some() {
                    Response::from_request(req)
                } else {
                    Ok(read_html("static/display"))
                }
            }
            "/edit" => Ok(read_html("static/editor")),
            _ => try_find_plugin_or_static(req.resource()),
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
