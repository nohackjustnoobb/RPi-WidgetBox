use crate::{Message, MessageType, Server};

use reqwest::blocking::get;
use serde::{Deserialize, Serialize};
use serde_json::{from_value, json, to_value, Value};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use ws::Result;

#[derive(Clone, Serialize, Deserialize)]
pub struct Config {
    name: String,
    #[serde(rename = "type")]
    type_: String,
    default: Value,
    value: Option<Value>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ConfigValue {
    name: String,
    value: Value,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Script {
    url: Option<String>,
    inline: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PluginMeta {
    name: String,
    version: String,
    url: Option<String>,
    description: Option<String>,
    configs: Option<Vec<Config>>,
    script: Script,
}

impl PluginMeta {
    pub fn update_script(&mut self) {
        self.script.url = Some(format!("/script/{}.js", self.name));
        self.script.inline = None;
    }
}

impl Server {
    /// Lists all available plugins by reading their metadata from the `plugins` directory.
    ///
    /// This function iterates through each subdirectory in the `plugins` directory,
    /// looking for a `meta.json` file. If found and successfully parsed, the plugin's
    /// metadata is added to a list. Finally, it sends a message containing the list
    /// of plugins.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Indicates success or failure of the operation.
    pub fn list_plugins(&self) -> Result<()> {
        let mut plugins = Vec::new();

        if let Ok(entries) = fs::read_dir("plugins") {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_dir() {
                        let file_path = format!("{}/meta.json", path.to_str().unwrap());
                        if let Ok(content) = fs::read_to_string(file_path) {
                            match serde_json::from_str::<PluginMeta>(&content) {
                                Ok(mut m) => {
                                    m.update_script();
                                    plugins.push(m)
                                }
                                Err(_) => continue,
                            };
                        }
                    }
                }
            }
        }

        self.send(Message {
            type_: MessageType::ListPlugins,
            data: json!(plugins),
        })
    }

    /// Adds a plugin by creating a new directory in `plugins` with the plugin's name,
    /// and creating a `meta.json` file in it with the plugin's metadata. The metadata is
    /// either obtained from the `url` field in the `data` parameter or from the `meta`
    /// field in the `data` parameter, if `url` is not provided. If `meta` is not provided
    /// or the metadata is not valid, the function returns an error message.
    ///
    /// If the plugin already exists, the function removes the old plugin and its
    /// associated files, and then creates a new one.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Indicates success or failure of the operation.
    pub fn add_plugin(&self, data: Value) -> Result<()> {
        let meta = match data["url"].as_str() {
            Some(url) => {
                let mut meta = None;

                if let Ok(response) = get(url) {
                    if let Ok(json) = response.json::<Value>() {
                        meta = Some(json);
                    }
                }

                meta
            }
            None => data.get("meta").cloned(),
        };

        if meta.is_none() {
            return self.send(Message::error("Failed to get meta."));
        }

        let mut parsed = match from_value::<PluginMeta>(meta.unwrap()) {
            Ok(p) => p,
            Err(_) => return self.send(Message::error("Failed to parse meta.")),
        };

        if parsed.script.inline.is_none() && parsed.script.url.is_none() {
            return self.send(Message::error("Failed to get script."));
        }

        let mut configs = parsed.configs.clone().unwrap_or(vec![]);

        configs.insert(
            0,
            Config {
                name: "enabled".to_string(),
                type_: "checkbox".to_string(),
                default: Value::Bool(false),
                value: None,
            },
        );

        for config in configs.iter_mut() {
            if config.value.is_none() {
                config.value = Some(config.default.clone());
            }
        }

        parsed.configs = Some(configs);

        let dir_path = format!("plugins/{}", parsed.name);
        if Path::new(dir_path.as_str()).exists() {
            if fs::remove_dir_all(&dir_path).is_err() {
                return self.send(Message::error("Failed to remove old plugin."));
            };
        }

        if fs::create_dir_all(dir_path.as_str()).is_err() {
            return self.send(Message::error("Failed to create plugin directory."));
        };

        let file_path = format!("{}/meta.json", dir_path);
        let mut file = match File::create(file_path) {
            Ok(f) => f,
            Err(_) => {
                fs::remove_dir_all(&dir_path).unwrap();
                return self.send(Message::error("Failed to create meta file."));
            }
        };

        let raw = match serde_json::to_string(&parsed) {
            Ok(r) => r,
            Err(_) => {
                fs::remove_dir_all(&dir_path).unwrap();
                return self.send(Message::error("Failed to serialize meta."));
            }
        };

        if file.write_all(raw.as_bytes()).is_err() {
            fs::remove_dir_all(&dir_path).unwrap();
            return self.send(Message::error("Failed to write to meta file."));
        };

        let script = match parsed.script.inline.as_ref() {
            Some(s) => s.clone(),
            None => {
                let url = parsed.script.url.as_ref().unwrap();
                let resp = match get(url) {
                    Ok(r) => r,
                    Err(_) => return self.send(Message::error("Failed to get the script file.")),
                };

                match resp.text() {
                    Ok(t) => t,
                    Err(_) => return self.send(Message::error("Failed to get the script file.")),
                }
            }
        };

        let file_path = format!("{}/script.js", dir_path);
        let mut file = match File::create(file_path) {
            Ok(f) => f,
            Err(_) => {
                fs::remove_dir_all(&dir_path).unwrap();
                return self.send(Message::error("Failed to create script file."));
            }
        };

        if file.write_all(script.as_bytes()).is_err() {
            fs::remove_dir_all(&dir_path).unwrap();
            return self.send(Message::error("Failed to write to script file."));
        };

        parsed.update_script();

        self.broadcast(Message {
            type_: MessageType::AddPlugin,
            data: to_value(parsed).unwrap(),
        })
    }

    /// Removes a plugin.
    ///
    /// # Parameters
    ///
    /// * `data` - A JSON object with a `name` property.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Indicates success or failure of the operation.
    pub fn remove_plugin(&self, data: Value) -> Result<()> {
        let name = match data["name"].as_str() {
            Some(n) => n,
            None => return self.send(Message::error("Failed to get plugin.")),
        };

        let dir_path = format!("plugins/{}", name);
        if !Path::new(dir_path.as_str()).exists() {
            return self.send(Message::error("Plugin not found."));
        }

        if fs::remove_dir_all(&dir_path).is_err() {
            return self.send(Message::error("Failed to remove plugin."));
        };

        self.broadcast(Message {
            type_: MessageType::RemovePlugin,
            data: json!({
                "name": name
            }),
        })
    }

    /// Configures a plugin.
    ///
    /// # Parameters
    ///
    /// * `data` - A JSON object with a `name` property and a `configs` property.
    ///   The `configs` property should be a JSON array of objects with `name` and `value` properties.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Indicates success or failure of the operation.
    pub fn config_plugin(&self, data: Value) -> Result<()> {
        let name = match data["name"].as_str() {
            Some(n) => n,
            None => return self.send(Message::error("Failed to get plugin.")),
        };

        let configs = match from_value::<Vec<ConfigValue>>(data["configs"].clone()) {
            Ok(c) => c,
            Err(_) => return self.send(Message::error("Failed to parse configs.")),
        };

        let file_path = format!("plugins/{}/meta.json", name);
        let raw = match fs::read_to_string(&file_path) {
            Ok(r) => r,
            Err(_) => return self.send(Message::error("Failed to read meta file.")),
        };

        let mut meta = match serde_json::from_str::<PluginMeta>(&raw) {
            Ok(v) => v,
            Err(_) => return self.send(Message::error("Failed to parse meta file.")),
        };

        let mut configs_map = HashMap::new();
        for cv in configs.iter() {
            configs_map.insert(cv.name.clone(), cv.value.clone());
        }

        let mut meta_configs = meta.configs.clone().unwrap();
        for config in meta_configs.iter_mut() {
            if let Some(value) = configs_map.get(&config.name) {
                config.value = Some(value.clone());
            }
        }
        meta.configs = Some(meta_configs);

        let raw = match serde_json::to_string(&meta) {
            Ok(r) => r,
            Err(_) => return self.send(Message::error("Failed to serialize meta.")),
        };

        match fs::write(file_path, raw) {
            Ok(_) => {
                meta.update_script();

                self.broadcast(Message {
                    type_: MessageType::ConfigPlugin,
                    data: json!(meta),
                })
            }
            Err(_) => self.send(Message::error("Failed to update meta file.")),
        }
    }
}
