use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;

pub const OFFICIAL_ITUNES_URL: &str = "https://secure-appldnld.apple.com/itunes12/002-38600-20240508-6E4381C4-0C5F-4E90-B8E5-B4E612B17618/iTunes64Setup.exe";

pub fn get_airlift_home_dir() -> PathBuf {
    let mut path = env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/tmp"));
    path.push(".airlift");
    path
}

pub fn get_wine_prefix_dir() -> PathBuf {
    let mut path = get_airlift_home_dir();
    path.push("wine_prefix");
    path
}

pub fn get_deps_dir() -> PathBuf {
    let mut path = get_airlift_home_dir();
    path.push("deps");
    path
}

pub async fn ensure_socat_bridge() -> bool {
    let check = Command::new("pgrep")
        .arg("-f")
        .arg("socat.*27015")
        .stdout(Stdio::piped())
        .output()
        .await;

    if let Ok(out) = check {
        if !out.stdout.is_empty() {
            return true;
        }
    }

    let spawn = Command::new("socat")
        .arg("TCP-LISTEN:27015,fork,reuseaddr")
        .arg("UNIX-CONNECT:/var/run/usbmuxd")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();

    spawn.is_ok()
}

pub async fn run_setup() -> Result<(), Box<dyn std::error::Error>> {
    let airlift_home = get_airlift_home_dir();
    let deps_dir = get_deps_dir();
    let wine_prefix = get_wine_prefix_dir();

    fs::create_dir_all(&deps_dir)?;
    fs::create_dir_all(&wine_prefix)?;

    println!("Checking dependencies...");
    let socat_ok = ensure_socat_bridge().await;
    if !socat_ok {
        eprintln!("Warning: socat bridge to /var/run/usbmuxd could not be started automatically");
    }

    let installer_path = deps_dir.join("iTunes64Setup.exe");
    if !installer_path.exists() {
        let existing_installer = PathBuf::from("/home/vcvk/airlift/build/iTunes64Setup.exe");
        if existing_installer.exists() {
            fs::copy(&existing_installer, &installer_path)?;
        } else {
            println!("Downloading iTunes setup from Apple servers...");
            let status = Command::new("curl")
                .arg("-L")
                .arg("-o")
                .arg(&installer_path)
                .arg(OFFICIAL_ITUNES_URL)
                .status()
                .await?;

            if !status.success() {
                return Err("Failed to download iTunes setup".into());
            }
        }
    }

    let wine_exe = deps_dir.join("airtraffic_host_win.exe");
    let existing_exe = PathBuf::from("/home/vcvk/airlift/build/airtraffic_host_win.exe");
    if existing_exe.exists() {
        fs::copy(&existing_exe, &wine_exe)?;
    }

    let existing_prefix = PathBuf::from("/home/vcvk/wine_airtraffic");
    if existing_prefix.exists() && !wine_prefix.exists() {
        let _ = Command::new("cp")
            .arg("-r")
            .arg(&existing_prefix)
            .arg(&wine_prefix)
            .status()
            .await;
    }

    println!("Setup completed successfully.");
    println!("Environment path: {:?}", airlift_home);
    Ok(())
}
