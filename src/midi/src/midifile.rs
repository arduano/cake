use std::{collections::VecDeque, rc::Rc};

use getset::Getters;
use to_vec::ToVec;

use crate::{
    data::{IntVector4, Leaf, TreeSerializer},
    errors::MIDILoadError,
    miditrack::{MIDITrack, MidiTrackOutput},
    readers::{DiskReader, MIDIReader, RAMReader},
};
struct TrackPos {
    pos: u64,
    len: u32,
}

#[derive(Getters)]
pub struct MIDIFile {
    reader: Box<dyn MIDIReader>,
    track_positions: Vec<TrackPos>,

    #[getset(get = "pub")]
    ppq: u16,
    #[getset(get = "pub")]
    track_count: u32,
}

impl MIDIFile {
    pub fn new(
        filename: &str,
        load_to_ram: bool,
        read_progress: Option<&dyn Fn(u32)>,
    ) -> Result<Self, MIDILoadError> {
        let mut reader = match load_to_ram {
            true => Box::new(RAMReader::new(filename)?) as Box<dyn MIDIReader>,
            false => Box::new(DiskReader::new(filename)?) as Box<dyn MIDIReader>,
        };

        reader.assert_header("MThd")?;

        let header_len = reader.read_value(4)?;

        if header_len != 6 {
            return Err(MIDILoadError::CorruptChunks);
        }

        let _format = reader.read_value(2)?;
        let _track_count_bad = reader.read_value(2)?;
        let ppq = reader.read_value(2)? as u16;

        let mut track_count = 0 as u32;
        let mut track_positions = Vec::<TrackPos>::new();
        while !reader.is_end()? {
            reader.assert_header("MTrk")?;
            track_count += 1;
            let len = reader.read_value(4)?;
            let pos = reader.get_position()?;
            track_positions.push(TrackPos { len, pos });
            reader.skip(len as u64)?;

            match read_progress {
                Some(progress) => progress(track_count),
                _ => {}
            };
        }

        Ok(MIDIFile {
            reader,
            ppq,
            track_count,
            track_positions,
        })
    }

    pub fn parse_all_tracks(&mut self, tps: u32) -> Result<Vec<IntVector4>, MIDILoadError> {
        let mut tracks = self
            .track_positions
            .iter()
            .enumerate()
            .map(|(i, pos)| {
                let r = self.reader.open_reader(pos.pos, pos.len as u64, true);
                MIDITrack::new(r, i as u32)
            })
            .to_vec();

        let mut time = 0.0;

        let mut output = MidiTrackOutput::new(self.ppq as u32);

        let mut trees = Vec::new();
        let mut vecs = Vec::new();

        for _ in 0..256 {
            trees.push(TreeSerializer::new(4));
            vecs.push(VecDeque::new());
        }

        let mut all_ended = false;
        while !all_ended {
            let time_int = (time * tps as f64) as i64;
            if time_int > i32::MAX as i64 {
                return Err(MIDILoadError::MIDITooLong);
            }
            let time_int = time_int as i32;

            all_ended = true;
            for track in tracks.iter_mut() {
                if track.ended() {
                    continue;
                }
                all_ended = false;
                track.read_tick(&mut output, time_int)?;
            }

            time += output.last_tempo_time_step();

            if *output.note_events_counted() > 10000000 {
                println!("Feeding notes, {}", output.note_count());

                for i in 0..256 {
                    let vec = &mut vecs[i];
                    let tree = &mut trees[i];
                    output.flush_notes(i as i32, vec);
                    loop {
                        match vec.pop_back() {
                            None => break,
                            Some(note) => {
                                tree.feed_note(Rc::new(note));
                            }
                        }
                    }
                }

                output.reset_note_event_counted();
            }
        }
                
        for i in 0..256 {
            let vec = &mut vecs[i];
            let tree = &mut trees[i];
            output.flush_notes(i as i32, vec);
            loop {
                match vec.pop_back() {
                    None => break,
                    Some(note) => {
                        tree.feed_note(Rc::new(note));
                    }
                }
            }
        }
        output.assert_empty();

        let trees = trees.into_iter().map(|t| t.complete()).to_vec();

        // for queue in &mut output.queues {
        //     let mut tree = TreeSerializer::new(4);
        //     loop {
        //         match queue.pop_back() {
        //             None => break,
        //             Some(note) => {
        //                 let note = Rc::try_unwrap(note).expect("Not all notes were ended!");
        //                 let note = note.into_inner();
        //                 tree.feed_note(Rc::new(note));
        //             }
        //         }
        //     }
        //     trees.push(tree.complete());
        // }

        let sum: u64 = trees.iter().map(|l| l.count()).sum();

        println!("Nodes: {}", sum);

        let mut serialized = (0..256)
            .map(|v| IntVector4::default())
            .to_vec();
        for (i, t) in trees.iter().enumerate() {
            serialized[i].val1 = t.serialize_to_vec(&mut serialized);
        }

        Ok(serialized)
    }
}
