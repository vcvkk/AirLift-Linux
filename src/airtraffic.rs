use std::env;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;
use crate::setup::{ensure_socat_bridge, get_deps_dir, get_wine_prefix_dir};
use crate::types::AirTrafficResult;

pub async fn run_native_airtraffic_host(
    udid: &str,
    identifiers: &[String],
    destinations: &[String],
) -> AirTrafficResult {
    let _ = ensure_socat_bridge().await;

    let mut exe_path = get_deps_dir().join("airtraffic_host_win.exe");
    if !exe_path.exists() {
        let mut alt_path = PathBuf::from(env::current_dir().unwrap_or_default());
        alt_path.push("build");
        alt_path.push("airtraffic_host_win.exe");
        if alt_path.exists() {
            exe_path = alt_path;
        } else {
            let root_path = PathBuf::from("/home/vcvk/airlift/build/airtraffic_host_win.exe");
            if root_path.exists() {
                exe_path = root_path;
            } else {
                return AirTrafficResult {
                    ok: false,
                    sync_allowed: None,
                    ready_for_sync: None,
                    file_complete_messages: None,
                    exit_code: Some(1),
                    error: Some(format!("airtraffic_host_win.exe not found at {:?}", exe_path)),
                };
            }
        }
    }

    let wine_prefix = env::var("WINEPREFIX").unwrap_or_else(|_| {
        let existing = PathBuf::from("/home/vcvk/wine_airtraffic");
        if existing.exists() {
            existing.to_string_lossy().to_string()
        } else {
            get_wine_prefix_dir().to_string_lossy().to_string()
        }
    });

    let mut command = Command::new("wine");
    command.arg(&exe_path);
    command.arg(udid);

    for (ident, dest) in identifiers.iter().zip(destinations.iter()) {
        command.arg(ident);
        command.arg(dest);
    }

    command.env("WINEPREFIX", wine_prefix);
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    match command.output().await {
        Ok(output) => {
            let stdout_str = String::from_utf8_lossy(&output.stdout);
            let exit_code = output.status.code().unwrap_or(-1);

            for line in stdout_str.lines().rev() {
                let trimmed = line.trim();
                if trimmed.starts_with('{') && trimmed.ends_with('}') {
                    if let Ok(mut parsed) = serde_json::from_str::<AirTrafficResult>(trimmed) {
                        parsed.exit_code = Some(exit_code);
                        return parsed;
                    }
                }
            }

            AirTrafficResult {
                ok: false,
                sync_allowed: None,
                ready_for_sync: None,
                file_complete_messages: None,
                exit_code: Some(exit_code),
                error: Some(format!("Wine process produced no valid JSON output: {}", stdout_str)),
            }
        }
        Err(err) => AirTrafficResult {
            ok: false,
            sync_allowed: None,
            ready_for_sync: None,
            file_complete_messages: None,
            exit_code: None,
            error: Some(format!("Failed to execute Wine process: {}", err)),
        },
    }
}
