#[derive(Debug)]
pub enum MIDILoadError {
    NotFound,
    CorruptChunks,
    Format2MIDI,
    UnknownFilesystemError,
    OutOfBoundsError,
    MIDITooLong,
}
