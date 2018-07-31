use common::error::*;
use common::io::base_writer::*;
use common::io::magic::*;
use common::Ptr;
use common::StringBlock;

use memmap::MmapMut;
use memmap::MmapOptions;

use std::cell::RefCell;
use std::error::Error;
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

    pub fn jump(&mut self, len: usize) -> Result<FileWriter, SkillFail> {
        self.flush()?;

        let new_pos = match self.file.borrow_mut().seek(SeekFrom::Current(len as i64)) {
            Ok(p) => Ok(p),
            Err(e) => Err(SkillFail::internal(InternalFail::BadSeek {
                why: e.description().to_owned(),
            })),
        }? as usize;

        // NOTE this _actually_ extends the file size
        match self.file.borrow_mut().set_len(new_pos as u64) {
            Ok(()) => Ok(()),
            Err(e) => Err(SkillFail::internal(InternalFail::FailedToResizeFile {
                why: e.description().to_owned(),
            })),
        }?;

        let mmap = match unsafe {
            MmapOptions::new()
                .len(new_pos)
                // TODO use offset?
                .map_mut(&self.file.borrow())
        } {
            Ok(map) => Ok(map),
            Err(e) => Err(SkillFail::internal(InternalFail::FailedToCreateMMap {
                why: e.description().to_owned(),
            })),
        }?;

        let writer = FileWriter {
            file: self.file.clone(),
            buffer_position: new_pos - len,
            out: Out::MMap(mmap),
        };
        Ok(writer)
    }

    fn require_buffer(&mut self, space: usize) -> Result<(), SkillFail> {
        // double matching can't be prevented because borrowing rules
        match self.out {
            Out::Buffer(ref mut buf) => {
                if self.buffer_position + space >= BUFFER_SIZE {
                    match self
                        .file
                        .borrow_mut()
                        .write_all(&buf[..self.buffer_position])
                    {
                        Ok(_) => Ok(()),
                        Err(e) => Err(SkillFail::internal(InternalFail::BadWrite {
                            why: e.description().to_owned(),
                        })),
                    }?;
                    self.buffer_position = 0;
                }
            }
            Out::MMap(ref mut map) => {}
        }
        Ok(())
    }

    pub fn flush(&mut self) -> Result<usize, SkillFail> {
        match self.out {
            Out::Buffer(ref mut buf) => {
                match self
                    .file
                    .borrow_mut()
                    .write_all(&buf[..self.buffer_position])
                {
                    Ok(_) => Ok(()),
                    Err(e) => Err(SkillFail::internal(InternalFail::BadWrite {
                        why: e.description().to_owned(),
                    })),
                }?;
                match self.file.borrow_mut().flush() {
                    Ok(_) => Ok(()),
                    Err(e) => Err(SkillFail::internal(InternalFail::BadFlush {
                        why: e.description().to_owned(),
                    })),
                }?;
                let tmp = self.buffer_position;
                self.buffer_position = 0;

                match self.file.borrow_mut().sync_data() {
                    Ok(_) => Ok(()),
                    Err(e) => Err(SkillFail::internal(InternalFail::BadFlush {
                        why: e.description().to_owned(),
                    })),
                }?;
                Ok(tmp)
            }
            Out::MMap(ref mut map) => {
                match map.flush() {
                    Ok(_) => Ok(()),
                    Err(e) => Err(SkillFail::internal(InternalFail::BadFlush {
                        why: e.description().to_owned(),
                    })),
                }?;
                match self.file.borrow_mut().sync_data() {
                    Ok(_) => Ok(()),
                    Err(e) => Err(SkillFail::internal(InternalFail::BadFlush {
                        why: e.description().to_owned(),
                    })),
                }?;
                Ok(map.len())
            }
        }
    }

    // writeing
    // boolean
    pub fn write_bool(&mut self, what: bool) -> Result<(), SkillFail> {
        self.require_buffer(1)?;
        match &mut self.out {
            Out::Buffer(ref mut buf) => write_bool(&mut self.buffer_position, buf, what),
            Out::MMap(ref mut map) => write_bool(&mut self.buffer_position, &mut map[..], what),
        }
        Ok(())
    }

    // integer types
    pub fn write_i8(&mut self, what: i8) -> Result<(), SkillFail> {
        self.require_buffer(1)?;
        match self.out {
            Out::Buffer(ref mut buf) => write_i8(&mut self.buffer_position, buf, what),
            Out::MMap(ref mut map) => write_i8(&mut self.buffer_position, &mut map[..], what),
        }
        Ok(())
    }

    pub fn write_i16(&mut self, what: i16) -> Result<(), SkillFail> {
        self.require_buffer(2)?;
        match self.out {
            Out::Buffer(ref mut buf) => write_i16(&mut self.buffer_position, buf, what),
            Out::MMap(ref mut map) => write_i16(&mut self.buffer_position, &mut map[..], what),
        }
        Ok(())
    }

    pub fn write_i32(&mut self, what: i32) -> Result<(), SkillFail> {
        self.require_buffer(4)?;
        match self.out {
            Out::Buffer(ref mut buf) => write_i32(&mut self.buffer_position, buf, what),
            Out::MMap(ref mut map) => write_i32(&mut self.buffer_position, &mut map[..], what),
        }
        Ok(())
    }

    pub fn write_i64(&mut self, what: i64) -> Result<(), SkillFail> {
        self.require_buffer(8)?;
        match self.out {
            Out::Buffer(ref mut buf) => write_i64(&mut self.buffer_position, buf, what),
            Out::MMap(ref mut map) => write_i64(&mut self.buffer_position, &mut map[..], what),
        }
        Ok(())
    }

    pub fn write_v64(&mut self, what: i64) -> Result<(), SkillFail> {
        self.require_buffer(9)?;
        match self.out {
            Out::Buffer(ref mut buf) => write_v64(&mut self.buffer_position, buf, what),
            Out::MMap(ref mut map) => write_v64(&mut self.buffer_position, &mut map[..], what),
        }
        Ok(())
    }

    // float types
    pub fn write_f32(&mut self, what: f32) -> Result<(), SkillFail> {
        self.require_buffer(4)?;
        match self.out {
            Out::Buffer(ref mut buf) => write_f32(&mut self.buffer_position, buf, what),
            Out::MMap(ref mut map) => write_f32(&mut self.buffer_position, &mut map[..], what),
        }
        Ok(())
    }

    pub fn write_f64(&mut self, what: f64) -> Result<(), SkillFail> {
        self.require_buffer(8)?;
        match self.out {
            Out::Buffer(ref mut buf) => write_f64(&mut self.buffer_position, buf, what),
            Out::MMap(ref mut map) => write_f64(&mut self.buffer_position, &mut map[..], what),
        }
        Ok(())
    }

    // string
    pub fn write_raw_string(&mut self, what: &str) -> Result<(), SkillFail> {
        self.require_buffer(what.len())?;
        match self.out {
            Out::Buffer(ref mut buf) => {
                if what.len() > BUFFER_SIZE {
                    let pos = match self.file.borrow_mut().seek(SeekFrom::Current(0)) {
                        Ok(pos) => Ok(pos),
                        Err(e) => Err(SkillFail::internal(InternalFail::BadSeek {
                            why: e.description().to_owned(),
                        })),
                    }?;
                    trace!(
                        target: "SkillBaseTypewriting",
                        "#W# str:|{:?}| position:{:?} out:file",
                        what,
                        pos
                    );
                    match self.file.borrow_mut().write_all(what.as_bytes()) {
                        Ok(_) => Ok(()),
                        Err(e) => Err(SkillFail::internal(InternalFail::BadWrite {
                            why: e.description().to_owned(),
                        })),
                    }?
                } else {
                    write_string(&mut self.buffer_position, buf, what)?;
                }
            }
            Out::MMap(ref mut map) => write_string(&mut self.buffer_position, &mut map[..], what)?,
        }
        Ok(())
    }

    pub fn write_field_type(&mut self, field_type: &FieldType) -> Result<(), SkillFail> {
        match field_type {
            FieldType::BuildIn(BuildInType::ConstTi8) => {
                info!(target: "SkillWriting", "~~~~FieldType = const i8");
                self.write_i8(0x0)?;
            }
            FieldType::BuildIn(BuildInType::ConstTi16) => {
                info!(target: "SkillWriting", "~~~~FieldType = const i16");
                self.write_i8(0x1)?;
            }
            FieldType::BuildIn(BuildInType::ConstTi32) => {
                info!(target: "SkillWriting", "~~~~FieldType = const i32");
                self.write_i8(0x2)?;
            }
            FieldType::BuildIn(BuildInType::ConstTi64) => {
                info!(target: "SkillWriting", "~~~~FieldType = const i64");
                self.write_i8(0x3)?;
            }
            FieldType::BuildIn(BuildInType::ConstTv64) => {
                info!(target: "SkillWriting", "~~~~FieldType = const v64");
                self.write_i8(0x4)?;
            }
            FieldType::BuildIn(BuildInType::Tannotation) => {
                info!(target: "SkillWriting", "~~~~FieldType = annotation");
                self.write_i8(0x5)?;
            }
            FieldType::BuildIn(BuildInType::Tbool) => {
                info!(target: "SkillWriting", "~~~~FieldType = bool");
                self.write_i8(0x6)?;
            }
            FieldType::BuildIn(BuildInType::Ti8) => {
                info!(target: "SkillWriting", "~~~~FieldType = i8");
                self.write_i8(0x7)?;
            }
            FieldType::BuildIn(BuildInType::Ti16) => {
                info!(target: "SkillWriting", "~~~~FieldType = i16");
                self.write_i8(0x8)?;
            }
            FieldType::BuildIn(BuildInType::Ti32) => {
                info!(target: "SkillWriting", "~~~~FieldType = i32");
                self.write_i8(0x9)?;
            }
            FieldType::BuildIn(BuildInType::Ti64) => {
                info!(target: "SkillWriting", "~~~~FieldType = i64");
                self.write_i8(0xA)?;
            }
            FieldType::BuildIn(BuildInType::Tv64) => {
                info!(target: "SkillWriting", "~~~~FieldType = v64");
                self.write_i8(0xB)?;
            }
            FieldType::BuildIn(BuildInType::Tf32) => {
                info!(target: "SkillWriting", "~~~~FieldType = f32");
                self.write_i8(0xC)?;
            }
            FieldType::BuildIn(BuildInType::Tf64) => {
                info!(target: "SkillWriting", "~~~~FieldType = f64");
                self.write_i8(0xD)?;
            }
            FieldType::BuildIn(BuildInType::Tstring) => {
                info!(target: "SkillWriting", "~~~~FieldType = string");
                self.write_i8(0xE)?;
            }
            FieldType::BuildIn(BuildInType::ConstTarray(length, ref boxed)) => {
                info!(
                    target: "SkillWriting",
                    "~~~~FieldType = const array length: {:?}",
                    length
                );
                self.write_i8(0xF)?;
                self.write_v64(*length as i64)?;
                self.write_field_type(boxed)?;
            }
            FieldType::BuildIn(BuildInType::Tarray(ref boxed)) => {
                info!(target: "SkillWriting", "~~~~FieldType = varray");
                self.write_i8(0x11)?;
                self.write_field_type(boxed)?;
            }
            FieldType::BuildIn(BuildInType::Tlist(ref boxed)) => {
                info!(target: "SkillWriting", "~~~~FieldType = list");
                self.write_i8(0x12)?;
                self.write_field_type(boxed)?;
            }
            FieldType::BuildIn(BuildInType::Tset(ref boxed)) => {
                info!(target: "SkillWriting", "~~~~FieldType = set");
                self.write_i8(0x13)?;
                self.write_field_type(boxed)?;
            }
            FieldType::BuildIn(BuildInType::Tmap(ref key_boxed, ref val_boxed)) => {
                info!(target: "SkillWriting", "~~~~FieldType = map");
                self.write_i8(0x14)?;
                self.write_field_type(key_boxed)?;
                self.write_field_type(val_boxed)?;
            }
            FieldType::User(ref user) => {
                info!(
                    target: "SkillWriting",
                    "~~~~FieldType = User{} ID:{}",
                    user.borrow().name().as_str(),
                    user.borrow().get_type_id()
                );
                self.write_v64(user.borrow().get_type_id() as i64)?;
            }
        }
        Ok(())
    }
}

impl Drop for FileWriter {
    fn drop(&mut self) {
        match self.flush() {
            Ok(_) => {}
            Err(e) => error!(
                target: "SkillWriting",
                "{}",
                e,
            ),
        };
    }
}
