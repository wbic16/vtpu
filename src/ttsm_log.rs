//! Versioned, append-only payload snapshots for TTSM.
//!
//! A single file is the commit index: every complete, checksummed, synced
//! record advances the recovered head. There is deliberately no separate
//! manifest/head pointer that could disagree with the append log.

use super::{simple_hash, TTSMError, TTSMResult, TemporalBlock, TemporalBlockId};
use crate::PhextCoord;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

const MAGIC: &[u8; 8] = b"VTTSM001";
const MAX_BODY: usize = 2 * 1024 * 1024;
const HEADER_LEN: usize = 12; // u32 body size + u64 FNV-1a checksum

#[derive(Clone)]
pub(super) enum LogEvent {
    Commit {
        block: TemporalBlock,
        payload: Vec<u8>,
    },
    Fork {
        source_head: TemporalBlockId,
        fork_head: TemporalBlockId,
    },
}

pub(super) struct SnapshotLog {
    file: File,
    poisoned: bool,
}

fn io_error(context: &str, err: std::io::Error) -> TTSMError {
    TTSMError::Storage {
        message: format!("{context}: {err}"),
    }
}

fn corruption(message: impl Into<String>) -> TTSMError {
    TTSMError::CorruptLog {
        message: message.into(),
    }
}

impl SnapshotLog {
    pub(super) fn open(path: &Path) -> TTSMResult<(Self, Vec<LogEvent>)> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)
            .map_err(|e| io_error("open snapshot log", e))?;

        let file_len = file
            .metadata()
            .map_err(|e| io_error("stat snapshot log", e))?
            .len();
        if file_len == 0 {
            file.write_all(MAGIC)
                .map_err(|e| io_error("write magic", e))?;
            file.sync_all()
                .map_err(|e| io_error("sync new snapshot log", e))?;
            sync_parent(path)?;
            return Ok((
                Self {
                    file,
                    poisoned: false,
                },
                Vec::new(),
            ));
        }
        if file_len < MAGIC.len() as u64 {
            let mut partial = vec![0u8; file_len as usize];
            file.read_exact(&mut partial)
                .map_err(|e| io_error("read partial magic", e))?;
            if partial != MAGIC[..file_len as usize] {
                return Err(corruption("snapshot log header is invalid"));
            }
            // Initialization was interrupted before the first record.
            file.set_len(0)
                .map_err(|e| io_error("recover partial magic", e))?;
            file.seek(SeekFrom::Start(0))
                .map_err(|e| io_error("rewind partial magic", e))?;
            file.write_all(MAGIC)
                .map_err(|e| io_error("rewrite magic", e))?;
            file.sync_all()
                .map_err(|e| io_error("sync recovered magic", e))?;
            sync_parent(path)?;
            return Ok((
                Self {
                    file,
                    poisoned: false,
                },
                Vec::new(),
            ));
        }
        let mut magic = [0u8; 8];
        file.read_exact(&mut magic)
            .map_err(|e| io_error("read magic", e))?;
        if &magic != MAGIC {
            return Err(corruption("snapshot log magic/version mismatch"));
        }

        let mut events = Vec::new();
        let mut offset = MAGIC.len() as u64;
        while offset < file_len {
            let remaining = file_len - offset;
            if remaining < HEADER_LEN as u64 {
                // Incomplete append: no acknowledgement could have followed
                // sync_data. Retain the preceding valid prefix.
                truncate_tail(&mut file, offset)?;
                break;
            }
            let mut header = [0u8; HEADER_LEN];
            file.read_exact(&mut header)
                .map_err(|e| io_error("read record header", e))?;
            let body_len = u32::from_le_bytes(header[0..4].try_into().unwrap()) as usize;
            let checksum = u64::from_le_bytes(header[4..12].try_into().unwrap());
            if body_len == 0 || body_len > MAX_BODY {
                return Err(corruption(format!(
                    "record at {offset} has invalid size {body_len}"
                )));
            }
            if remaining - (HEADER_LEN as u64) < body_len as u64 {
                truncate_tail(&mut file, offset)?;
                break;
            }
            let mut body = vec![0u8; body_len];
            file.read_exact(&mut body)
                .map_err(|e| io_error("read record body", e))?;
            if checksum != simple_hash(&body) {
                return Err(corruption(format!("record at {offset} failed checksum")));
            }
            events.push(decode(&body)?);
            offset += HEADER_LEN as u64 + body_len as u64;
        }
        file.seek(SeekFrom::End(0))
            .map_err(|e| io_error("seek append position", e))?;
        Ok((
            Self {
                file,
                poisoned: false,
            },
            events,
        ))
    }

    pub(super) fn append(&mut self, event: &LogEvent) -> TTSMResult<()> {
        if self.poisoned {
            return Err(TTSMError::Storage {
                message: "prior append failed; reopen and recover before writing again".into(),
            });
        }
        let body = encode(event)?;
        let body_len =
            u32::try_from(body.len()).map_err(|_| corruption("record exceeds u32 size limit"))?;
        let mut record = Vec::with_capacity(HEADER_LEN + body.len());
        record.extend_from_slice(&body_len.to_le_bytes());
        record.extend_from_slice(&simple_hash(&body).to_le_bytes());
        record.extend_from_slice(&body);

        // A successful commit/fork only returns after these bytes are synced.
        // A failed write/sync poisons this writer so later appends cannot be
        // accidentally placed after a torn record without recovery.
        if let Err(error) = self
            .file
            .write_all(&record)
            .and_then(|_| self.file.sync_data())
        {
            self.poisoned = true;
            return Err(io_error("append/sync snapshot", error));
        }
        Ok(())
    }
}

fn truncate_tail(file: &mut File, offset: u64) -> TTSMResult<()> {
    file.set_len(offset)
        .map_err(|e| io_error("discard torn record", e))?;
    file.sync_data()
        .map_err(|e| io_error("sync torn-tail recovery", e))?;
    file.seek(SeekFrom::Start(offset))
        .map_err(|e| io_error("seek recovered end", e))?;
    Ok(())
}

#[cfg(unix)]
fn sync_parent(path: &Path) -> TTSMResult<()> {
    // A new file's directory entry must be durable, not just its data.
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    File::open(parent)
        .and_then(|dir| dir.sync_all())
        .map_err(|e| io_error("sync snapshot directory", e))
}

#[cfg(not(unix))]
fn sync_parent(_path: &Path) -> TTSMResult<()> {
    // Windows does not expose a portable directory-fsync equivalent via std.
    Ok(())
}

fn write_id(out: &mut Vec<u8>, id: TemporalBlockId) {
    out.extend_from_slice(&id.sequence.to_le_bytes());
    out.extend_from_slice(&id.fork_id.to_le_bytes());
    out.extend_from_slice(&id.timestamp_ns.to_le_bytes());
}

fn write_text(out: &mut Vec<u8>, text: &str) -> TTSMResult<()> {
    let bytes = text.as_bytes();
    let len = u16::try_from(bytes.len()).map_err(|_| corruption("owner/label too long"))?;
    out.extend_from_slice(&len.to_le_bytes());
    out.extend_from_slice(bytes);
    Ok(())
}

fn encode(event: &LogEvent) -> TTSMResult<Vec<u8>> {
    let mut out = Vec::new();
    match event {
        LogEvent::Commit { block, payload } => {
            out.push(1);
            write_id(&mut out, block.id);
            write_id(
                &mut out,
                block
                    .parent
                    .ok_or_else(|| corruption("commit missing parent"))?,
            );
            for dim in block.state_coord.dims() {
                out.extend_from_slice(&dim.to_le_bytes());
            }
            out.push(block.state_coord.flags());
            write_text(&mut out, &block.owner)?;
            write_text(&mut out, &block.label)?;
            out.extend_from_slice(
                &u32::try_from(payload.len())
                    .map_err(|_| corruption("payload exceeds u32"))?
                    .to_le_bytes(),
            );
            out.extend_from_slice(payload);
        }
        LogEvent::Fork {
            source_head,
            fork_head,
        } => {
            out.push(2);
            write_id(&mut out, *source_head);
            write_id(&mut out, *fork_head);
        }
    }
    if out.len() > MAX_BODY {
        return Err(corruption("snapshot record exceeds 2 MiB safety limit"));
    }
    Ok(out)
}

struct Cursor<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl<'a> Cursor<'a> {
    fn take(&mut self, n: usize) -> TTSMResult<&'a [u8]> {
        let end = self
            .at
            .checked_add(n)
            .ok_or_else(|| corruption("record offset overflow"))?;
        let part = self
            .bytes
            .get(self.at..end)
            .ok_or_else(|| corruption("truncated framed record"))?;
        self.at = end;
        Ok(part)
    }
    fn u8(&mut self) -> TTSMResult<u8> {
        Ok(self.take(1)?[0])
    }
    fn u16(&mut self) -> TTSMResult<u16> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }
    fn u32(&mut self) -> TTSMResult<u32> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn u64(&mut self) -> TTSMResult<u64> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }
    fn u128(&mut self) -> TTSMResult<u128> {
        Ok(u128::from_le_bytes(self.take(16)?.try_into().unwrap()))
    }
    fn id(&mut self) -> TTSMResult<TemporalBlockId> {
        Ok(TemporalBlockId {
            sequence: self.u64()?,
            fork_id: self.u64()?,
            timestamp_ns: self.u128()?,
        })
    }
    fn text(&mut self) -> TTSMResult<String> {
        let len = self.u16()? as usize;
        String::from_utf8(self.take(len)?.to_vec())
            .map_err(|_| corruption("invalid UTF-8 in owner/label"))
    }
    fn finish(&self) -> TTSMResult<()> {
        if self.at == self.bytes.len() {
            Ok(())
        } else {
            Err(corruption("trailing data in framed record"))
        }
    }
}

fn decode(body: &[u8]) -> TTSMResult<LogEvent> {
    let mut c = Cursor { bytes: body, at: 0 };
    let event = match c.u8()? {
        1 => {
            let id = c.id()?;
            let parent = c.id()?;
            let mut dims = [0u16; 11];
            for dim in &mut dims {
                *dim = c.u16()?;
            }
            if dims.iter().any(|&d| d > PhextCoord::MAX_DIM) {
                return Err(corruption("dimension exceeds 11-bit range"));
            }
            let flags = c.u8()?;
            if flags > 127 {
                return Err(corruption("coordinate flags exceed 7 bits"));
            }
            let mut coord = PhextCoord::new(dims);
            coord.set_flags(flags);
            let owner = c.text()?;
            let label = c.text()?;
            let len = c.u32()? as usize;
            let payload = c.take(len)?.to_vec();
            let block = TemporalBlock {
                id,
                parent: Some(parent),
                state_coord: coord,
                state_hash: simple_hash(&payload),
                state_size: payload.len(),
                owner,
                label,
            };
            LogEvent::Commit { block, payload }
        }
        2 => LogEvent::Fork {
            source_head: c.id()?,
            fork_head: c.id()?,
        },
        kind => return Err(corruption(format!("unknown snapshot event type {kind}"))),
    };
    c.finish()?;
    Ok(event)
}
