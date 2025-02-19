# RPi WidgetBox

RPi WidgetBox is a simple desktop widget application designed primarily for Raspberry Pi and other small-screen devices. However, it can run on any device that supports a modern web browser.

Features

- Lightweight & flexible—runs on low-power devices
- Plugin-based system—comes with no built-in widgets, allowing full customization
- Cross-platform—works on Raspberry Pi, PCs, and more

To use the application, you'll need to add plugins. Several common plugins are available, including a fullscreen clock, weather widget. Check out the [RPi-WidgetBox-Plugins](https://github.com/nohackjustnoobb/RPi-WidgetBox-Plugins) repository for more details.

## Quick Start

TODO

## Development Setup

1. Clone the Repository

First, clone the repository and navigate into the project directory:

```bash
git clone https://github.com/nohackjustnoobb/RPi-WidgetBox && cd RPi-WidgetBox
```

2. Build the Static Files

Ensure you have `node` and `yarn` installed before running the build script:

```bash
chmod +x build_static.sh && ./build_static.sh
```

3. Run the Application

Finally, start the application using Cargo:

```bash
cargo run
```

## Roadmap to v1.0.0

- [x] Display modules
- [x] Build script
- [ ] Documentation
- [ ] Dockerize
