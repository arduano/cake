use midi::midifile::MIDIFile;

pub fn main() {
    let midi = MIDIFile::new("D:\\Midis\\tau2.5.9.mid");
    match midi {
        Err(error) => {
            println!("Error loading midi")
        }
        Ok(file) => {
            println!("Success!")
        }
    }
}
