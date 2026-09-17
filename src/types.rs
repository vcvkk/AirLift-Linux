use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct StageResult {
    pub ok: bool,
    pub zip_response: String,
    pub source_objects_present: bool,
    pub books_written: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AirTrafficResult {
    pub ok: bool,
    pub sync_allowed: Option<bool>,
    pub ready_for_sync: Option<bool>,
    pub file_complete_messages: Option<u32>,
    pub exit_code: Option<i32>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FinishResult {
    pub ok: bool,
    pub recovered_present: bool,
    pub recovered_bytes_match: bool,
    pub cleanup_complete: bool,
    pub failures: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PrimaryResult {
    pub stage_succeeded: bool,
    pub air_traffic_succeeded: bool,
    pub exact_bytes_recovered: bool,
    pub cleanup_complete: bool,
    pub stage: StageResult,
    pub atc: AirTrafficResult,
    pub finish: FinishResult,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RunReport {
    pub ok: bool,
    pub target_directory: String,
    pub generated_leaf: String,
    pub payload_sha256: String,
    pub primary: PrimaryResult,
}
