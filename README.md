# AirLift-Linux

AirTraffic and AFC device synchronization tool written in Rust for Linux.

## Requirements

- Linux (x86_64)
- `usbmuxd`
- `wine` 64-bit with Apple Mobile Device Support (`AirTrafficHost.dll`, `CoreFP.dll`)
- Python 3 with `pymobiledevice3`
- `socat` TCP bridge to usbmuxd:
  ```bash
  socat TCP-LISTEN:27015,fork,reuseaddr UNIX-CONNECT:/var/run/usbmuxd &
  ```

## Build

```bash
cargo build --release
```

The binary will be saved at `target/release/airlift`.

## Usage

```bash
# Run against connected device
./target/release/airlift --device <UDID> --target /var/mobile/Library/SpringBoard

# Check AirTrafficHost DLL status
./target/release/airlift --check-dll
```

### Command Line Options

- `--device <UDID>`: Specify target device UDID.
- `--target <PATH>`: Target directory on device (default: `/var/mobile/Library/SpringBoard`).
- `--check-dll`: Verify Wine AirTrafficHost setup.
