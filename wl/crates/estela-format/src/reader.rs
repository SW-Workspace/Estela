use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::Path,
};

use crate::{
    consts::{BRAND, CLOSURE_UUID, SLOT_A_UUID, SLOT_B_UUID},
    tracks::Recording,
    writer::MAX_METADATA,
    Error, Result,
};

pub fn open(path: &Path) -> Result<Recording> {
    let mut file = File::open(path)?;
    let length = file.metadata()?.len();
    let mut header = [0; 12];
    file.read_exact(&mut header)?;
    if &header[4..8] != b"ftyp" || header[8..12] != BRAND {
        return Err(Error::Invalid("Not an Estela recording".into()));
    }
    let mut offset = 0;
    let mut indexes = Vec::new();
    while offset + 8 <= length {
        file.seek(SeekFrom::Start(offset))?;
        let mut header = [0; 8];
        file.read_exact(&mut header)?;
        let short = u32::from_be_bytes(
            header[..4]
                .try_into()
                .map_err(|_| Error::Invalid("Invalid box".into()))?,
        );
        let (size, header_size) = match short {
            0 => (length - offset, 8),
            1 => {
                let mut large = [0; 8];
                file.read_exact(&mut large)?;
                (u64::from_be_bytes(large), 16)
            }
            size => (u64::from(size), 8),
        };
        if size < header_size || size > length - offset {
            break;
        }
        if &header[4..] == b"uuid" && size >= header_size + 32 {
            let mut uuid = [0; 16];
            file.read_exact(&mut uuid)?;
            if uuid == SLOT_A_UUID || uuid == SLOT_B_UUID {
                let mut index = [0; 16];
                file.read_exact(&mut index)?;
                indexes.push(index);
            }
        }
        offset += size;
    }
    for index in indexes.iter().rev() {
        if let Ok(recording) = read_index(&mut file, index, length) {
            return Ok(recording);
        }
    }
    Err(Error::Invalid(
        "No valid recording metadata slot; original media is retained".into(),
    ))
}

fn read_index(file: &mut File, index: &[u8; 16], file_length: u64) -> Result<Recording> {
    let offset = u64::from_be_bytes(
        index[..8]
            .try_into()
            .map_err(|_| Error::Invalid("Invalid index".into()))?,
    );
    let length = u32::from_be_bytes(
        index[8..12]
            .try_into()
            .map_err(|_| Error::Invalid("Invalid index".into()))?,
    ) as usize;
    let crc = u32::from_be_bytes(
        index[12..]
            .try_into()
            .map_err(|_| Error::Invalid("Invalid index".into()))?,
    );
    if length > MAX_METADATA || offset > file_length || length as u64 + 24 > file_length - offset {
        return Err(Error::Invalid("Metadata bounds are invalid".into()));
    }
    file.seek(SeekFrom::Start(offset))?;
    let mut header = [0; 24];
    file.read_exact(&mut header)?;
    if u32::from_be_bytes(
        header[..4]
            .try_into()
            .map_err(|_| Error::Invalid("Invalid metadata box".into()))?,
    ) as usize
        != length + 24
        || &header[4..8] != b"uuid"
        || header[8..] != CLOSURE_UUID
    {
        return Err(Error::Invalid("Missing recording metadata".into()));
    }
    let mut payload = vec![0; length];
    file.read_exact(&mut payload)?;
    if crc32fast::hash(&payload) != crc {
        return Err(Error::Invalid("Recording metadata checksum failed".into()));
    }
    let recording: Recording = serde_json::from_slice(&payload)?;
    recording.validate()?;
    Ok(recording)
}
