import "./style.css";

import { Message, WebSocketClient } from "./webSocket";

function updateTheme() {
  const useDarkMode = window.matchMedia("(prefers-color-scheme: dark)").matches;

  window.document.documentElement.setAttribute(
    "data-theme",
    useDarkMode ? "dark" : "light"
  );
}

window
  .matchMedia("(prefers-color-scheme: dark)")
  .addEventListener("change", () => updateTheme());
updateTheme();

interface Config<T> {
  name: string;
  type: string;
  value: T;
  default: T;
}

interface Plugin {
  name: string;
  version: string;
  enabled: boolean;
  description?: string;
  url?: string;
  configs: Array<Config<any>>;
  script: {
    url?: string;
    inline?: string;
  };
}

class Display {
  ws: WebSocketClient;
  _plugins: { [name: string]: Plugin } = {};
  host: string;
  selected?: Plugin;

  get plugins() {
    return Object.values(this._plugins);
  }

  set plugins(plugins: Array<Plugin>) {
    this._plugins = {};
    plugins
      .map((p) => {
        p.enabled = p.configs.find((c) => c.name == "enabled")!.value;
        p.script.url = "http://" + this.host + p.script.url;
        return p;
      })
      .forEach((p) => {
        if (p.enabled) this._plugins[p.name] = p;
      });

    this.updateDOM();
  }

  constructor() {
    const searchParams = new URLSearchParams(window.location.search);
    this.host = searchParams.get("url") || window.location.host;
    this.ws = new WebSocketClient(
      `ws://${this.host}`,
      this.handler.bind(this),
      () =>
        this.ws.send({
          type: "listPlugins",
        })
    );
  }

  updateDOM() {
    if (!this.plugins.length) {
      // TODO: Display no plugins
      return;
    }

    if (!this.selected) this.selected = this.plugins[0];
    // Update selected
    this.selected = this._plugins[this.selected.name];

    // Change changes
    const info = document.querySelector(".info");
    if (info) {
      // No changes
      if (JSON.stringify(this.selected) === info.innerHTML) return;

      // Check if only the config changes
      const parsed = JSON.parse(info.innerHTML);
      if (parsed.name === this.selected.name) {
        const webComponent = document.querySelector(this.selected.name);
        if (webComponent) {
          for (const config of this.selected.configs) {
            if (config.name !== "enabled" && config.value !== null)
              webComponent.setAttribute(config.name, String(config.value));
          }
        }

        return;
      }
    }

    // Clear the body
    document.body.innerHTML = "";

    // Insert the plugin
    const infoElement = document.createElement("info");
    infoElement.innerHTML = JSON.stringify(this.selected);
    document.body.appendChild(infoElement);

    const scriptElement = document.createElement("script");
    scriptElement.src = this.selected.script.url!;
    document.body.appendChild(scriptElement);

    const webComponent = document.createElement(this.selected.name);
    for (const config of this.selected.configs) {
      if (config.name !== "enabled" && config.value !== null)
        webComponent.setAttribute(config.name, String(config.value));
    }
    document.body.appendChild(webComponent);
  }

  handler(mesg: Message) {
    switch (mesg.type) {
      case "listPlugins":
        this.plugins = mesg.data as Array<Plugin>;
        break;
      case "configPlugin":
      case "addPlugin":
        let plugin = mesg.data as Plugin;
        this.plugins = [...this.plugins, plugin];
        break;
      case "removePlugin":
        let name = mesg.data.name as string;
        this.plugins = this.plugins.filter((p) => p.name !== name);
        break;
      case "error":
        console.error("Error:", mesg.data);
        break;
      // IGNORE
      case "broadcast":
        break;
      default:
        console.warn("Unknown plugin type:", mesg.type);
        break;
    }
  }
}

new Display();
