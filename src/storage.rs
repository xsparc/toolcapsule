//! Bounded local I/O; callers choose paths, never guests or manifests.
use crate::{
    Error,
    audit::{AuditCheckpoint, AuditEntry, Receipt},
};
use serde::de::DeserializeOwned;
use std::{
    fs::{File, OpenOptions},
    io::{BufRead, Read, Write},
    path::Path,
};

pub const MAX_AUDIT_BYTES: usize = 8 * 1024 * 1024;

pub fn read_bounded(mut reader: impl Read, max: usize) -> Result<Vec<u8>, Error> {
    let mut bytes = Vec::new();
    reader
        .by_ref()
        .take(max.checked_add(1).ok_or(Error::Policy)? as u64)
        .read_to_end(&mut bytes)
        .map_err(|_| Error::Io)?;
    if bytes.len() > max {
        return Err(Error::InputLimit);
    }
    Ok(bytes)
}

pub fn read_file(path: &Path, max: usize) -> Result<Vec<u8>, Error> {
    read_bounded(File::open(path).map_err(|_| Error::Io)?, max)
}

pub fn read_json<T: DeserializeOwned>(path: &Path, max: usize) -> Result<T, Error> {
    serde_json::from_slice(&read_file(path, max)?).map_err(|_| Error::Format)
}

/// Never clobbers an existing key, package, or receipt. New Unix files are owner-only.
pub fn write_new(path: &Path, bytes: &[u8]) -> Result<(), Error> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path).map_err(|_| Error::Io)?;
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(|_| Error::Io)
}

/// Locks the file, checks its existing chain, and durably appends one entry.
pub fn append_audit(path: &Path, receipt: Receipt) -> Result<AuditEntry, Error> {
    let mut options = OpenOptions::new();
    options.read(true).append(true).create(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path).map_err(|_| Error::Io)?;
    file.try_lock().map_err(|_| Error::Io)?;
    let bytes = read_bounded(&mut file, MAX_AUDIT_BYTES)?;
    let (sequence, previous) = verify_log(&bytes)?;
    let entry = AuditEntry::new(sequence, previous, receipt)?;
    let mut line = serde_json::to_vec(&entry).map_err(|_| Error::Audit)?;
    line.push(b'\n');
    if bytes.len() + line.len() > MAX_AUDIT_BYTES {
        return Err(Error::Audit);
    }
    file.write_all(&line)
        .and_then(|()| file.sync_all())
        .map_err(|_| Error::Io)?;
    Ok(entry)
}

pub fn verify_log(bytes: &[u8]) -> Result<(u64, String), Error> {
    verify_chain(bytes, None)
}

/// Verify every entry and require the trusted anchor to match an observed prefix.
/// Exact mode additionally rejects valid suffix entries after that anchor.
pub fn verify_log_against(
    bytes: &[u8],
    anchor: &AuditCheckpoint,
    exact: bool,
) -> Result<(u64, String), Error> {
    anchor.validate()?;
    let head = verify_chain(bytes, Some(anchor))?;
    if exact && head.0 != anchor.entries {
        return Err(Error::Audit);
    }
    Ok(head)
}

pub fn checkpoint_log(bytes: &[u8]) -> Result<AuditCheckpoint, Error> {
    let (entries, checkpoint) = verify_log(bytes)?;
    Ok(AuditCheckpoint {
        schema_version: 1,
        entries,
        checkpoint,
    })
}

fn verify_chain(bytes: &[u8], anchor: Option<&AuditCheckpoint>) -> Result<(u64, String), Error> {
    if bytes.len() > MAX_AUDIT_BYTES || (!bytes.is_empty() && !bytes.ends_with(b"\n")) {
        return Err(Error::Audit);
    }
    let mut sequence = 0;
    let mut previous = String::new();
    for line in bytes.split(|&b| b == b'\n').filter(|l| !l.is_empty()) {
        let entry: AuditEntry = serde_json::from_slice(line).map_err(|_| Error::Audit)?;
        entry.verify(sequence, &previous)?;
        previous = entry.hash;
        sequence += 1;
        if anchor.is_some_and(|a| a.entries == sequence && a.checkpoint != previous) {
            return Err(Error::Audit);
        }
    }
    if anchor.is_some_and(|a| a.entries > sequence) {
        return Err(Error::Audit);
    }
    Ok((sequence, previous))
}

/// Bounded newline-delimited transport; oversize frames terminate the connection.
pub fn read_frame(reader: &mut impl BufRead, max: usize) -> Result<Option<Vec<u8>>, Error> {
    let mut frame = Vec::new();
    reader
        .take(max.checked_add(1).ok_or(Error::Policy)? as u64)
        .read_until(b'\n', &mut frame)
        .map_err(|_| Error::Io)?;
    if frame.len() > max {
        return Err(Error::InputLimit);
    }
    if frame.is_empty() {
        return Ok(None);
    }
    if !frame.ends_with(b"\n") {
        return Err(Error::Format);
    }
    Ok(Some(frame))
}
