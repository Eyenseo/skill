use common::io::base_writer::*;
use common::Ptr;
use common::SkillError;
use common::StringBlock;

use memmap::MmapMut;
use memmap::MmapOptions;

use std::cell::RefCell;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
use std::rc::Rc;

const BUFFER_SIZE: usize = 4096;

#[derive(Debug)]
enum Out {
    Buffer(Box<[u8]>),
    MMap(MmapMut),
}

#[derive(Debug)]
pub struct FileWriter {
    file: Rc<RefCell<std::fs::File>>,
    buffer_position: usize,
    out: Out,
}

impl FileWriter {
    pub fn new(file: Rc<RefCell<std::fs::File>>) -> FileWriter {
        FileWriter {
            file,
            buffer_position: 0,
            out: Out::Buffer(Box::new([0; BUFFER_SIZE])),
        }
    }

    pub fn jump(&mut self, len: usize) -> Result<FileWriter, SkillError> {
        self.flush();

        // NOTE might have to resize the file after seek
        let new_pos = self
            .file
            .borrow_mut()
            .seek(SeekFrom::Current(len as i64))
            // TODO better error
            .or(Err(SkillError::UnexpectedEndOfInput))? as usize;

        let writer = FileWriter {
            file: self.file.clone(),
            buffer_position: new_pos - len,
            out: Out::MMap(unsafe {
                MmapOptions::new()
                    .len(new_pos)
                    .map_mut(&self.file.borrow())
                    // TODO use offset?
                    // TODO better error
                    .or(Err(SkillError::UnexpectedEndOfInput))?
            }),
        };
        Ok(writer)
    }

    pub fn flush(&mut self) -> Result<usize, SkillError> {
        match self.out {
            Out::Buffer(ref mut buf) => {
                self.file
                    .borrow_mut()
                    .write_all(&buf[..self.buffer_position])
                    // TODO better error
                    .or(Err(SkillError::UnexpectedEndOfInput))?;
                self.file
                    .borrow_mut()
                    .flush()
                    // TODO better error
                    .or(Err(SkillError::UnexpectedEndOfInput));
                let tmp = self.buffer_position;
                self.buffer_position = 0;
                Ok(tmp)
            }
            Out::MMap(ref mut map) => {
                // TODO better error
                map.flush().or(Err(SkillError::UnexpectedEndOfInput))?;
                Ok(map.len())
            }
        }
    }

    // writeing
    // boolean
    pub fn write_bool(&mut self, what: bool) {
        match self.out {
            Out::Buffer(ref mut buf) => write_bool(&mut self.buffer_position, buf, what),
            Out::MMap(ref mut map) => write_bool(&mut self.buffer_position, &mut map[..], what),
        }
    }

    // integer types
    pub fn write_i8(&mut self, what: i8) {
        match self.out {
            Out::Buffer(ref mut buf) => write_i8(&mut self.buffer_position, buf, what),
            Out::MMap(ref mut map) => write_i8(&mut self.buffer_position, &mut map[..], what),
        }
    }

    pub fn write_i16(&mut self, what: i16) {
        match self.out {
            Out::Buffer(ref mut buf) => write_i16(&mut self.buffer_position, buf, what),
            Out::MMap(ref mut map) => write_i16(&mut self.buffer_position, &mut map[..], what),
        }
    }

    pub fn write_i32(&mut self, what: i32) {
        match self.out {
            Out::Buffer(ref mut buf) => write_i32(&mut self.buffer_position, buf, what),
            Out::MMap(ref mut map) => write_i32(&mut self.buffer_position, &mut map[..], what),
        }
    }

    pub fn write_i64(&mut self, what: i64) {
        match self.out {
            Out::Buffer(ref mut buf) => write_i64(&mut self.buffer_position, buf, what),
            Out::MMap(ref mut map) => write_i64(&mut self.buffer_position, &mut map[..], what),
        }
    }

    pub fn write_v64(&mut self, what: i64) {
        match self.out {
            Out::Buffer(ref mut buf) => write_v64(&mut self.buffer_position, buf, what),
            Out::MMap(ref mut map) => write_v64(&mut self.buffer_position, &mut map[..], what),
        }
    }

    // float types
    pub fn write_f32(&mut self, what: f32) {
        match self.out {
            Out::Buffer(ref mut buf) => write_f32(&mut self.buffer_position, buf, what),
            Out::MMap(ref mut map) => write_f32(&mut self.buffer_position, &mut map[..], what),
        }
    }

    pub fn write_f64(&mut self, what: f64) {
        match self.out {
            Out::Buffer(ref mut buf) => write_f64(&mut self.buffer_position, buf, what),
            Out::MMap(ref mut map) => write_f64(&mut self.buffer_position, &mut map[..], what),
        }
    }

    // string
    pub fn write_raw_string(&mut self, what: &str) {
        match self.out {
            Out::Buffer(ref mut buf) => write_string(&mut self.buffer_position, buf, what),
            Out::MMap(ref mut map) => write_string(&mut self.buffer_position, &mut map[..], what),
        }
    }

    // compound types
    // are generated
}
