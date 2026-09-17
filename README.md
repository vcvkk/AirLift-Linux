# AirLift-Linux

AirTraffic and AFC device synchronization tool written in Rust for Linux.

## Requirements

- Linux (x86_64)
- `usbmuxd` daemon

## Quick Start

1. Run one-time setup (automatically prepares dependencies and local environment):
   ```bash
   airlift --setup
   ```

2. Run synchronization check against connected device:
   ```bash
   airlift --device <UDID> --target /var/mobile/Library/SpringBoard
   ```

## Build from Source

```bash
cargo build --release
```

The compiled binary is saved at `target/release/airlift`.

## Command Line Options

- `--setup`: Automatically fetch and configure required runtime environment in `~/.airlift/`.
- `--device <UDID>`: Specify target iOS device UDID (default: first available usbmux device).
- `--target <PATH>`: Target directory on the device (default: `/var/mobile/Library/SpringBoard`).
- `--check-dll`: Verify AirTraffic status.
