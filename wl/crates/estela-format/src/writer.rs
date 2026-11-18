use std::{
    fs::OpenOptions,
    io::{Read, Seek, SeekFrom, Write},
    path::Path,
};

use crate::{
    consts::{BRAND, CLOSURE_UUID, SLOT_A_UUID, SLOT_B_UUID, SLOT_SIZE},
    tracks::Recording,
    Error, Result,
};

pub const MAX_METADATA: usize = 64 * 1024 * 1024;

pub fn finalize(path: &Path, recording: &Recording) -> Result<()> {
    recording.validate()?;
    let payload = serde_json::to_vec(recording)?;
    if payload.len() > MAX_METADATA {
        return Err(Error::Invalid(
            "Recording metadata exceeds the size limit".into(),
        ));
    }
    let mut file = OpenOptions::new().read(true).write(true).open(path)?;
    let mut header = [0; 12];
    file.read_exact(&mut header)?;
    if &header[4..8] != b"ftyp" {
        return Err(Error::Invalid("Missing media file type box".into()));
    }
    file.seek(SeekFrom::Start(8))?;
    file.write_all(&BRAND)?;
    let offset = file.seek(SeekFrom::End(0))?;
    write_box(&mut file, CLOSURE_UUID, &payload)?;
    let mut index = Vec::with_capacity(SLOT_SIZE - 24);
    index.extend_from_slice(&offset.to_be_bytes());
    index.extend_from_slice(&(payload.len() as u32).to_be_bytes());
    index.extend_from_slice(&crc32fast::hash(&payload).to_be_bytes());
    index.resize(SLOT_SIZE - 24, 0);
    write_box(&mut file, SLOT_A_UUID, &index)?;
    file.sync_all()?;
    write_box(&mut file, SLOT_B_UUID, &index)?;
    file.sync_all()?;
    Ok(())
}

fn write_box(writer: &mut impl Write, uuid: [u8; 16], payload: &[u8]) -> Result<()> {
    let length = u32::try_from(payload.len() + 24)
        .map_err(|_| Error::Invalid("Metadata box is too large".into()))?;
    writer.write_all(&length.to_be_bytes())?;
    writer.write_all(b"uuid")?;
    writer.write_all(&uuid)?;
    writer.write_all(payload)?;
    Ok(())
}
