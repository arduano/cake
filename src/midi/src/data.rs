use std::{
    borrow::Borrow,
    cmp::{max, min},
    collections::{LinkedList, VecDeque},
    rc::Rc,
};
use bytemuck::{Pod, Zeroable};

pub enum Leaf {
    Note(Option<Rc<Note>>),
    Node(Node),
}

impl Leaf {
    pub fn count(&self) -> u64 {
        match &self {
            &Leaf::Node(node) => node.upper.count() + node.lower.count() + 1,
            &Leaf::Note(_) => 1 as u64,
        }
    }

    pub fn serialize_to_vec(&self, vec: &mut Vec<IntVector4>) -> i32 {
        match &self {
            &Leaf::Node(node) => {
                let lower = node.lower.serialize_to_vec(vec);
                let upper = node.upper.serialize_to_vec(vec);

                vec.push(IntVector4 {
                    val1: node.cutoff,
                    val2: lower,
                    val3: upper,
                    val4: 0,
                });

                vec.len() as i32 - 1
            }
            &Self::Note(note) => {
                vec.push(match note {
                    None => IntVector4 {
                        val1: 0,
                        val2: 0,
                        val3: -1,
                        val4: 0,
                    },
                    Some(note) => IntVector4 {
                        val1: note.start,
                        val2: note.end,
                        val3: note.color,
                        val4: note.note_num,
                    },
                });

                -(vec.len() as i32 - 1)
            }
        }
    }
}

#[repr(C)]
#[derive(Pod, Copy, Clone, Zeroable)]
pub struct IntVector4 {
    pub val1: i32,
    pub val2: i32,
    pub val3: i32,
    pub val4: i32,
}

pub struct Note {
    pub start: i32,
    pub end: i32,
    pub color: i32,
    pub note_num: i32,
}

impl Note {
    const UNENDED: i32 = -1;

    fn encode_color(track: u32, channel: u8) -> i32 {
        track as i32 * 16 + channel as i32
    }

    pub fn new(start: i32, end: i32, track: u32, channel: u8) -> Self {
        Note {
            start,
            end,
            color: Note::encode_color(track, channel),
            note_num: 0,
        }
    }

    pub fn new_unended(start: i32, track: u32, channel: u8) -> Self {
        Note::new(start, Note::UNENDED, track, channel)
    }

    pub fn unended(&self) -> bool {
        self.end == Note::UNENDED
    }

    pub fn equals(&self, other: &Note) -> bool {
        self.start == other.start && self.end == other.end && self.color == other.color
    }
}

pub struct Node {
    cutoff: i32,
    upper: Box<Leaf>,
    lower: Box<Leaf>,
}

struct FetchingFirst {
    start: i32,
    half: i32,
    end: i32,
}

struct FetchingSecond {
    first: Leaf,
    half: i32,
    start: i32,
    end: i32,
}

enum SerializerFrame {
    FetchingFirst(FetchingFirst),
    FetchingSecond(FetchingSecond),
    FetchingNote(i32),
}

enum SerializerInput {
    Init,
    Note(Rc<Note>),
    End,
}

impl FetchingFirst {
    pub fn to_second(&self, first: Leaf, next_event: i32) -> SerializerFrame {
        debug_assert!(next_event < self.end);

        SerializerFrame::FetchingSecond(FetchingSecond {
            end: self.end,
            first,
            start: self.start,
            half: max(self.half, next_event),
        })
    }
}

impl SerializerFrame {
    pub fn new(start: i32, end: i32) -> SerializerFrame {
        if end - start == 1 {
            SerializerFrame::FetchingNote(start)
        } else {
            SerializerFrame::FetchingFirst(FetchingFirst {
                start,
                end,
                half: (start + end) / 2,
            })
        }
    }
}

pub struct TreeSerializer {
    time: i32,
    note_stack: LinkedList<Rc<Note>>,
    next_note: Option<Rc<Note>>,
    ended: bool,
    stack_frames: VecDeque<SerializerFrame>,
    fed_up_to: i32,
    parsed_up_to: i32,

    final_leaf: Option<Leaf>,
}

impl TreeSerializer {
    pub fn new(initial_end: i32) -> Self {
        let mut stack_frames = VecDeque::<SerializerFrame>::new();
        stack_frames.push_front(SerializerFrame::new(0, initial_end));
        let mut serializer = TreeSerializer {
            ended: false,
            time: 0,
            note_stack: LinkedList::new(),
            next_note: None,
            stack_frames,
            fed_up_to: 0,
            parsed_up_to: 0,

            final_leaf: None,
        };
        serializer.run_state_machine(SerializerInput::Init);

        serializer
    }

    fn clean_note_stack_fast(&mut self, time: i32) {
        loop {
            match self.note_stack.front() {
                None => break,
                Some(note) => {
                    if note.end > time {
                        break;
                    }
                }
            }
            self.note_stack.pop_front();
        }
    }

    fn max_parse_dist(&self) -> i32 {
        match &self.next_note {
            None => {
                if self.ended {
                    i32::MAX
                } else {
                    0
                }
            }
            Some(n) => n.start,
        }
    }

    pub fn feed_note(&mut self, note: Rc<Note>) {
        self.run_state_machine(SerializerInput::Note(note));
    }

    pub fn complete(mut self) -> Leaf {
        self.run_state_machine(SerializerInput::End);
        return self.final_leaf.expect("Final leaf not received");
    }

    fn next_event(&self) -> i32 {
        let next_start = match &self.next_note {
            None => i32::MAX,
            Some(n) => n.start,
        };
        let next_end = match self.note_stack.front() {
            None => i32::MAX,
            Some(n) => n.end,
        };

        min(next_start, next_end)
    }

    fn run_state_machine(&mut self, new_event: SerializerInput) {
        if let Some(n) = self.next_note.take() {
            self.fed_up_to = n.start;
            self.clean_note_stack_fast(n.end);
            self.note_stack.push_front(n);
        }

        let mut skip_returns = match new_event {
            SerializerInput::End => {
                self.ended = true;
                false
            }
            SerializerInput::Note(note) => {
                self.next_note = Some(note);
                false
            }
            SerializerInput::Init => true,
        };

        // Maximum position it can
        let max_parse_dist = self.max_parse_dist();

        loop {
            if !skip_returns {
                // Check if last stack frame is useable
                let pos = match self.stack_frames.front() {
                    Some(frame) => match frame {
                        SerializerFrame::FetchingNote(pos) => {
                            if *pos >= max_parse_dist {
                                break;
                            }
                            pos
                        }
                        _ => panic!("Invalid last frame in the stack"),
                    },

                    None => panic!("Empty stack frames in tree builder"),
                };

                // because FetchingNote's end is `pos + 1`
                self.parsed_up_to = pos + 1;

                let top_note = match self.note_stack.front() {
                    None => None,
                    Some(n) => Some(n.clone()),
                };

                self.stack_frames.pop_front();

                // Pop stack frames
                let mut ret = Leaf::Note(top_note);
                let next_event = self.next_event();
                loop {
                    if self.stack_frames.len() == 0 {
                        if self.ended && self.note_stack.len() == 0 {
                            self.final_leaf = Some(ret);
                            return;
                        }
                        let frame = SerializerFrame::new(0, max(next_event, self.parsed_up_to) * 2);
                        self.stack_frames.push_front(frame);
                    }

                    let frame = self.stack_frames.pop_front().unwrap();
                    match frame {
                        SerializerFrame::FetchingFirst(frame) => {
                            if next_event >= frame.end {
                                continue;
                            }
                            self.stack_frames
                                .push_front(frame.to_second(ret, next_event));
                            break;
                        }
                        SerializerFrame::FetchingSecond(frame) => {
                            let first = frame.first;

                            if let Leaf::Note(f) = &first {
                                if let Leaf::Note(s) = &ret {
                                    if let Some(f) = f {
                                        if let Some(s) = s {
                                            if s.equals(f.borrow()) {
                                                ret = first;
                                                continue;
                                            }
                                        }
                                    } else {
                                        if let None = s {
                                            ret = first;
                                            continue;
                                        }
                                    }
                                }
                            }

                            ret = Leaf::Node(Node {
                                cutoff: frame.half,
                                lower: Box::new(first),
                                upper: Box::new(ret),
                            });
                            continue;
                        }
                        SerializerFrame::FetchingNote(_) => {
                            panic!("Multiple fetch note frames in stack")
                        }
                    }
                }
            }
            skip_returns = false;

            // Push stack frames
            let mut final_start = 0;
            loop {
                let (start, end) = match &self.stack_frames.front().unwrap() {
                    &SerializerFrame::FetchingFirst(frame) => (frame.start, frame.half),
                    &SerializerFrame::FetchingSecond(frame) => (frame.half, frame.end),
                    &SerializerFrame::FetchingNote(_) => break,
                };
                final_start = start;

                self.stack_frames
                    .push_front(SerializerFrame::new(start, end));
            }
            self.clean_note_stack_fast(final_start);
        }
    }
}

// pub fn serialize_key_notes(queue: &mut VecDeque<Rc<Note>>) {
//     let lastPos = 0;

//     let mut stack = VecDeque::<Rc<Note>>::new();

//     let mut next_event = 0;

//     let mut step_to = |pos: i32| {
//         loop {
//             match stack.front() {
//                 Some(note) => {
//                     if note.end < pos {
//                         stack.pop_front();
//                     }
//                 }
//                 None => break,
//             }
//         }

//         loop {
//             match queue.back() {
//                 None => break,
//                 Some(note) => {
//                     if note.start > pos {
//                         break;
//                     }

//                     let note = queue.pop_back().unwrap();
//                     if note.end > pos {
//                         stack.push_front(note);
//                     }
//                 }
//             }
//         }

//         next_event = i32::MAX;

//         match stack.front() {
//             Some(n) => next_event = n.end,
//             None => {}
//         }

//         match queue.back() {
//             Some(n) => next_event = min(n.start, next_event),
//             None => {}
//         }
//     };

//     let mut recursive_build = |start: i32, end: i32| -> Leaf {
//         if end - start == 1 {
//             step_to(start);
//             let note = stack.front();
//             let note = match note {
//                 None => None,
//                 Some(n) => Some(n.clone()),
//             };
//             return Leaf::Note(note);
//         }

//         let half = (start + end) / 2;

//         let first = recursive_build(start, half);

//         if let Leaf::Note(note) = first {
//             if next_event >= end {
//                 return first;
//             }
//         }

//         if next_event > half && next_event < end {
//             half = next_event;
//         }

//         let second = recursive_build(half, end);

//         if let Leaf::Note(f) = first {
//             if let Leaf::Note(s) = second {
//                 if let Some(f) = f {
//                     if let Some(s) = s {
//                         if s.equals(f.borrow()) {
//                             return first;
//                         }
//                     }
//                 } else {
//                     if let None = s {
//                         return first;
//                     }
//                 }
//             }
//         }

//         Leaf::Node(Node {
//             cutoff: half,
//             lower: Box::new(first),
//             upper: Box::new(second),
//         })
//     };

//     // var root = RecursiveBuild(startTime, endTime);

//     // uniqueNoteCount[key] = uniqueNotes.Count;

//     // return root;
// }

// pub fn serialize_full_notes() {}
