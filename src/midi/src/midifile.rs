use std::{fs::File, io::Read};

struct TrackPos {
    pos: i64,
    len: u32,
}

pub enum MIDILoadError {
    NotFound,
    CorruptChunks,
}

pub struct MIDIFile {
    reader: File,
    track_positions: Vec<TrackPos>,
    ppq: u16,
    track_count: i32,
    format: u32,
}

impl MIDIFile {
    pub fn new(filename: &str) -> Result<Self, MIDILoadError> {
        let reader_maybe = File::open(filename);
        if reader_maybe.is_err() {
            return Err(MIDILoadError::NotFound);
        }

        let mut reader = reader_maybe.unwrap();

        let reader_borrow = &mut reader;

        let mut check_header = |text: &str| -> bool {
            let chars = text.as_bytes();
            let mut bytes = vec![0 as u8; chars.len()];
            let read = reader_borrow.read_exact(&mut bytes);

            if read.is_err() {
                return false;
            }

            for i in 0..chars.len() {
                if chars[i] != bytes[i] {
                    return false;
                }
            }
            return true;
        };

        if !check_header("MThd") {
            return Err(MIDILoadError::CorruptChunks);
        }

        Ok(MIDIFile {
            reader,
            format: 0,
            ppq: 0,
            track_count: 0,
            track_positions: vec![],
        })
    }
}
