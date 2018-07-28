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

        let new_pos = self
            .file
            .borrow_mut()
            .seek(SeekFrom::Current(len as i64))
            // TODO better error
            .or(Err(SkillError::UnexpectedEndOfInput))? as usize;

        // NOTE this _actually_ extends the file size
        self.file
            .borrow_mut()
            .set_len(new_pos as u64)
            // TODO better error
            .or(Err(SkillError::UnexpectedEndOfInput))?;

        let mmap = match unsafe {
            MmapOptions::new()
                    .len(new_pos)
                    // TODO use offset?
                    .map_mut(&self.file.borrow())
        } {
            Ok(map) => map,
            Err(e) => panic!("{}", e),
        };

        let writer = FileWriter {
            file: self.file.clone(),
            buffer_position: new_pos - len,
            out: Out::MMap(mmap),
        };
        Ok(writer)
    }

    fn require_buffer(&mut self, space: usize) -> Result<(), SkillError> {
        // double matching can't be prevented because borrowing rules
        match self.out {
            Out::Buffer(ref mut buf) => {
                if self.buffer_position + space >= BUFFER_SIZE {
                    self.file
                    .borrow_mut()
                    .write_all(&buf[..self.buffer_position])
                    // TODO better error
                    .or(Err(SkillError::UnexpectedEndOfInput))?;
                    self.buffer_position = 0;
                }
            }
            Out::MMap(ref mut map) => {}
        }
        Ok(())
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
                    .or(Err(SkillError::UnexpectedEndOfInput))?;
                let tmp = self.buffer_position;
                self.buffer_position = 0;
                self.file
                    .borrow_mut()
                    .sync_data()
                // TODO better error
                    .or(Err(SkillError::UnexpectedEndOfInput))?;
                Ok(tmp)
            }
            Out::MMap(ref mut map) => {
                // TODO better error
                map.flush().or(Err(SkillError::UnexpectedEndOfInput))?;
                self.file
                    .borrow_mut()
                    .sync_data()
                // TODO better error
                    .or(Err(SkillError::UnexpectedEndOfInput))?;
                Ok(map.len())
            }
        }
    }

    // writeing
    // boolean
    pub fn write_bool(&mut self, what: bool) {
        self.require_buffer(1);
        match &mut self.out {
            Out::Buffer(ref mut buf) => write_bool(&mut self.buffer_position, buf, what),
            Out::MMap(ref mut map) => write_bool(&mut self.buffer_position, &mut map[..], what),
        }
    }

    // integer types
    pub fn write_i8(&mut self, what: i8) {
        self.require_buffer(1);
        match self.out {
            Out::Buffer(ref mut buf) => write_i8(&mut self.buffer_position, buf, what),
            Out::MMap(ref mut map) => write_i8(&mut self.buffer_position, &mut map[..], what),
        }
    }

    pub fn write_i16(&mut self, what: i16) {
        self.require_buffer(2);
        match self.out {
            Out::Buffer(ref mut buf) => write_i16(&mut self.buffer_position, buf, what),
            Out::MMap(ref mut map) => write_i16(&mut self.buffer_position, &mut map[..], what),
        }
    }

    pub fn write_i32(&mut self, what: i32) {
        self.require_buffer(4);
        match self.out {
            Out::Buffer(ref mut buf) => write_i32(&mut self.buffer_position, buf, what),
            Out::MMap(ref mut map) => write_i32(&mut self.buffer_position, &mut map[..], what),
        }
    }

    pub fn write_i64(&mut self, what: i64) {
        self.require_buffer(8);
        match self.out {
            Out::Buffer(ref mut buf) => write_i64(&mut self.buffer_position, buf, what),
            Out::MMap(ref mut map) => write_i64(&mut self.buffer_position, &mut map[..], what),
        }
    }

    pub fn write_v64(&mut self, what: i64) {
        self.require_buffer(9);
        match self.out {
            Out::Buffer(ref mut buf) => write_v64(&mut self.buffer_position, buf, what),
            Out::MMap(ref mut map) => write_v64(&mut self.buffer_position, &mut map[..], what),
        }
    }

    // float types
    pub fn write_f32(&mut self, what: f32) {
        self.require_buffer(4);
        match self.out {
            Out::Buffer(ref mut buf) => write_f32(&mut self.buffer_position, buf, what),
            Out::MMap(ref mut map) => write_f32(&mut self.buffer_position, &mut map[..], what),
        }
    }

    pub fn write_f64(&mut self, what: f64) {
        self.require_buffer(8);
        match self.out {
            Out::Buffer(ref mut buf) => write_f64(&mut self.buffer_position, buf, what),
            Out::MMap(ref mut map) => write_f64(&mut self.buffer_position, &mut map[..], what),
        }
    }

    // string
    pub fn write_raw_string(&mut self, what: &str) {
        self.require_buffer(what.len());
        match self.out {
            Out::Buffer(ref mut buf) => {
                if what.len() > BUFFER_SIZE {
                    let pos = match self.file.borrow_mut().seek(SeekFrom::Current(0)) {
                        Ok(pos) => pos,
                        Err(e) => panic!("{}", e),
                    };
                    trace!(
                        target: "SkillBaseTypewriting",
                        "#W# str:|{:?}| position:{:?} out:file",
                        what,
                        pos
                    );
                    match self.file.borrow_mut().write_all(what.as_bytes()) {
                        Ok(_) => {}
                        Err(_) => panic!("Couldn't write the complete string!"),
                    }
                } else {
                    write_string(&mut self.buffer_position, buf, what)
                }
            }
            Out::MMap(ref mut map) => write_string(&mut self.buffer_position, &mut map[..], what),
        }
    }

    // compound types
    // are generated
}

impl Drop for FileWriter {
    fn drop(&mut self) {
        self.flush();
    }
}
