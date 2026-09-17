# AirLift-Linux

High-performance Rust implementation of the AirTraffic and AFC device synchronization utility for Linux environments.

## Overview

AirLift-Linux orchestrates device staging, AirTraffic protocol metadata synchronization, and file validation over USB connection (`usbmuxd`).

### Features

- Native Rust CLI executable (`airlift`).
- Interoperability with Apple Mobile Device Support (`AirTrafficHost.dll`) under Wine on Linux.
- Structured JSON reporting output.
- Automated cleanup and exact byte verification.

---

## Prerequisites

- Linux operating system (x86_64).
- `usbmuxd` daemon installed and running.
- `wine` 64-bit environment with Apple Mobile Device Support binaries (`AirTrafficHost.dll`, `CoreFP.dll`).
- Python 3 with `pymobiledevice3` installed.
- TCP bridge for `usbmuxd`:
  ```bash
  socat TCP-LISTEN:27015,fork,reuseaddr UNIX-CONNECT:/var/run/usbmuxd &
  ```

---

## Building from Source

### Prerequisites
Rust toolchain (Cargo 1.80+):

```bash
cargo build --release
```

The compiled binary will be placed at `target/release/airlift`.

---

## Usage

### Run Synchronization Test

```bash
airlift --device <UDID> --target /var/mobile/Library/SpringBoard
```

### Options

| Flag | Description | Default |
| --- | --- | --- |
| `--target <PATH>` | Target directory on the device | `/var/mobile/Library/SpringBoard` |
| `--device <UDID>` | UDID of the connected iOS device | First available usbmux device |
| `--check-dll` | Verify AirTrafficHost DLL status | `false` |

### Output Example

```json
{
  "ok": true,
  "target_directory": "/var/mobile/Library/SpringBoard",
  "generated_leaf": "airlift-canary-24A435-1f7c915c0f782f23987f3509d57e641c.bin",
  "payload_sha256": "052a912ddb9bd29268d756c45c13ffb95d54f28c1a8cd4a8d7928d5d03cae41e",
  "primary": {
    "stage_succeeded": true,
    "air_traffic_succeeded": true,
    "exact_bytes_recovered": true,
    "cleanup_complete": true
  }
}
```

---

## Continuous Integration & Workflows

The repository includes a GitHub Actions workflow (`.github/workflows/build.yml`) supporting manual triggers (`workflow_dispatch`) and release tagging.

The build workflow targets:
- `x86_64-unknown-linux-gnu` (Standard Linux GLIBC)
- `x86_64-unknown-linux-musl` (Static Linux MUSL)
- `x86_64-apple-darwin` / `aarch64-apple-darwin` (macOS)
- `x86_64-pc-windows-msvc` (Windows)
