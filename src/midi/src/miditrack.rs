use getset::Getters;
use std::{
    borrow::{Borrow, BorrowMut},
    cell::{Cell, RefCell, UnsafeCell},
    collections::VecDeque,
    rc::Rc,
    sync::Arc,
};

use crate::{data::Note, errors::MIDILoadError, readers::TrackReader};

#[derive(Getters)]
pub struct NoteQueues {
    pub queues: Vec<VecDeque<Rc<UnsafeCell<Note>>>>,
}

impl NoteQueues {}

#[derive(Getters)]
pub struct MidiTrackOutput {
    pub queues: Vec<VecDeque<Rc<UnsafeCell<Note>>>>,

    #[getset(get = "pub")]
    note_events_counted: u64,

    #[getset(get = "pub")]
    last_tempo_time_step: f64,

    ppq: u32,
}

impl MidiTrackOutput {
    pub fn new(ppq: u32) -> Self {
        let mut queues = Vec::<VecDeque<Rc<UnsafeCell<Note>>>>::new();

        for _ in 0..(256) {
            queues.push(VecDeque::new());
        }

        let mut output = MidiTrackOutput {
            queues,
            last_tempo_time_step: 0 as f64,
            note_events_counted: 0,
            ppq,
        };
        output.update_tempo(500000);
        output
    }

    pub fn add_note(&mut self, key: u8, note: Rc<UnsafeCell<Note>>) {
        self.queues[key as usize].push_front(note);
    }

    pub fn note_count(&self) -> u64 {
        self.queues.iter().map(|q| q.len() as u64).sum()
    }

    pub fn flush_notes(&mut self, key: i32, queue: &mut VecDeque<Note>) {
        let source = &mut self.queues[key as usize];

        loop {
            match source.back() {
                None => break,
                Some(note) => unsafe {
                    if (*note.get()).end == -1 {
                        break;
                    }
                    let note = source.pop_back().unwrap();
                    let note = Rc::try_unwrap(note).expect("Not all notes were ended!");
                    let note = note.into_inner();
                    queue.push_front(note);
                },
            }
        }
    }

    pub fn assert_empty(&self) {
        debug_assert!(self.queues.iter().map(|q| q.len()).sum::<usize>() == 0);
    }

    pub fn count_note_event(&mut self) {
        self.note_events_counted += 1;
    }

    pub fn reset_note_event_counted(&mut self) {
        self.note_events_counted = 0;
    }

    pub fn update_tempo(&mut self, tempo: u32) {
        self.last_tempo_time_step = (tempo as f64 / self.ppq as f64) / 1000000.0;
    }
}

pub struct MIDITrack {
    track_id: u32,

    ended: bool,
    reader: Box<dyn TrackReader>,
    has_read_delta: bool,
    next_event_pos: u64,
    pos: u64,
    pushback: i32,
    prev_command: u8,

    unended_notes: Option<Vec<VecDeque<Rc<UnsafeCell<Note>>>>>,
}

impl MIDITrack {
    pub fn new(reader: Box<dyn TrackReader>, track_id: u32) -> MIDITrack {
        MIDITrack {
            track_id,

            reader,
            ended: false,
            has_read_delta: false,
            next_event_pos: 0,
            pos: 0,
            pushback: -1,
            prev_command: 0,
            unended_notes: None,
        }
    }

    fn init_unended_queues() -> Vec<VecDeque<Rc<UnsafeCell<Note>>>> {
        let mut unended_notes = Vec::new();

        for _ in 0..(256 * 16) {
            unended_notes.push(VecDeque::new());
        }

        unended_notes
    }

    fn get_unended_queue_mut(&mut self, key: u8, chan: u8) -> &mut VecDeque<Rc<UnsafeCell<Note>>> {
        if self.unended_notes.is_none() {
            self.unended_notes.replace(MIDITrack::init_unended_queues());
        }

        let unended_notes = self.unended_notes.as_mut().unwrap();
        &mut unended_notes[(key as u32 * 16 + chan as u32) as usize]
    }

    #[inline]
    fn read(&mut self) -> Result<u8, MIDILoadError> {
        if self.pushback == -1 {
            self.read_fast()
        } else {
            let b = self.pushback as u8;
            self.pushback = -1;
            Ok(b)
        }
    }

    #[inline]
    fn read_fast(&mut self) -> Result<u8, MIDILoadError> {
        self.reader.read()
    }

    #[inline]
    fn read_variable_len(&mut self) -> Result<u32, MIDILoadError> {
        let mut val = 0 as u32;
        for _ in 0..4 {
            let c = self.read_fast()?;

            if c > 0x7F {
                val = (val << 7) | (c as u32 & 0x7F);
            } else {
                val = val << 7 | c as u32;
                break;
            }
        }
        return Ok(val);
    }

    fn end_note(note: Rc<UnsafeCell<Note>>, time: i32) {
        unsafe {
            let note = note.as_ref();
            (*note.get()).end = time;
        }
    }

    fn end_track(&mut self, time_int: i32) {
        self.ended = true;
        match &mut self.unended_notes {
            Some(unended_notes) => {
                for k in unended_notes {
                    loop {
                        let n = k.pop_back();
                        match n {
                            Some(n) => MIDITrack::end_note(n, time_int),
                            None => break,
                        }
                    }
                }
            }
            None => {}
        }
        self.unended_notes.take();
    }

    pub fn ended(&self) -> bool {
        self.ended
    }

    pub fn read_tick(
        &mut self,
        output: &mut MidiTrackOutput,
        time_int: i32,
    ) -> Result<(), MIDILoadError> {
        debug_assert!(self.ended == false);

        let mut read = || -> Result<(), MIDILoadError> {
            if !self.has_read_delta {
                self.read_delta()?;
            }

            while self.next_event_pos < self.pos {
                self.read_event(output, time_int)?;
                self.read_delta()?;
            }
            self.pos += 1;

            Ok(())
        };

        match read() {
            Err(e) => match e {
                MIDILoadError::OutOfBoundsError => {
                    self.end_track(time_int);
                    Ok(())
                }
                e => Err(e),
            },
            Ok(_) => Ok(()),
        }
    }

    fn read_delta(&mut self) -> Result<(), MIDILoadError> {
        debug_assert!(self.has_read_delta == false);

        self.next_event_pos += self.read_variable_len()? as u64;
        self.has_read_delta = true;

        Ok(())
    }

    fn read_event(
        &mut self,
        output: &mut MidiTrackOutput,
        time_int: i32,
    ) -> Result<(), MIDILoadError> {
        debug_assert!(self.has_read_delta == true);
        self.has_read_delta = false;

        let mut command = self.read()?;
        if command < 0x80 {
            self.pushback = command as i32;
            command = self.prev_command;
        }
        self.prev_command = command;

        let comm = command & 0xF0;

        match comm {
            0x90 | 0x80 => {
                let channel = command & 0x0F;
                let key = self.read()?;
                let vel = self.read_fast()?;

                output.count_note_event();

                if comm == 0x80 || vel == 0 {
                    let l = self.get_unended_queue_mut(key, channel);
                    let note = l.pop_back();
                    match note {
                        None => {}
                        Some(note) => MIDITrack::end_note(note, time_int),
                    }
                } else {
                    let n = Note::new_unended(time_int, self.track_id, channel);
                    let n = Rc::new(UnsafeCell::new(n));
                    output.add_note(key, n.clone());
                    let queue = self.get_unended_queue_mut(key, channel);
                    queue.push_front(n);
                }
            }

            0xA0 => {
                self.read()?;
                self.read_fast()?;
            }
            0xB0 => {
                self.read()?;
                self.read_fast()?;
            }
            0xC0 => {
                self.read()?;
            }
            0xD0 => {
                self.read()?;
            }
            0xE0 => {
                self.read()?;
                self.read_fast()?;
            }
            _ => match command {
                0xF0 => while self.read()? != 0b11110111 {},
                0b11110010 => {
                    self.read()?;
                    self.read_fast()?;
                }
                0b11110011 => {
                    self.read()?;
                }
                0xFF => {
                    let command = self.read()?;
                    let size = self.read_variable_len()?;
                    match command {
                        0x2F => {
                            self.end_track(time_int);
                        }
                        0x51 => {
                            if size != 3 {
                                self.end_track(time_int);
                            }

                            let mut btempo = 0 as u32;
                            for _ in 0..3 {
                                btempo = (btempo << 8) | self.read_fast()? as u32;
                            }

                            output.update_tempo(btempo);
                        }
                        _ => {
                            for _ in 0..size {
                                self.read_fast()?;
                            }
                        }
                    }
                }
                _ => {
                    // undefined event
                }
            },
        }

        Ok(())
    }
}
