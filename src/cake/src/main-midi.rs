use midi::midifile::MIDIFile;

pub fn main() {
    let midi = MIDIFile::new(
        "D:\\Midis\\Clubstep.mid",
        true,
        Some(&|read| {
            println!("{}", read);
        }),
    );
    match midi {
        Err(_) => {
            println!("Error loading midi")
        }
        Ok(mut file) => {
            println!("Success! {} tracks, {} ppq", file.track_count(), file.ppq());
            file.parse_all_tracks(16384).expect("MIDI parse failed");
        }
    }
}
