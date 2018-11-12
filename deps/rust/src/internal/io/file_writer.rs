use common::error::*;
use common::internal::io::base_writer::*;
use common::internal::io::magic::*;
use common::internal::StringBlock;
use common::Ptr;

use memmap::MmapMut;
use memmap::MmapOptions;

use std::cell::RefCell;
use std::error::Error;
use std::io::{Seek, SeekFrom, Write};
use std::ops::{Deref, DerefMut};
use std::rc::{Rc, Weak};

const BUFFER_SIZE: usize = 4096;

/// enum that contains the different types of outputs
#[derive(Debug)]
enum Out<'v> {
    // Buffer for small writes
    Buffer(Box<[u8]>),
    // mmap directly into the file
    MMap(MmapMut),
    // view to jump over bits
    View(&'v mut [u8]),
}

// The kind of output doesn't matter, all provide a slice / array to write into
impl<'v> Deref for Out<'v> {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        match self {
            Out::Buffer(buf) => buf.as_ref(),
            Out::MMap(map) => &map[..],
            Out::View(view) => view,
        }
    }
}

impl<'v> DerefMut for Out<'v> {
    fn deref_mut(&mut self) -> &mut [u8] {
        match self {
            Out::Buffer(buf) => buf.as_mut(),
            Out::MMap(map) => &mut map[..],
            Out::View(view) => view,
        }
    }
}

/// struct that provides the means to write to a file
#[derive(Debug)]
pub(crate) struct FileWriter<'v> {
    file: Rc<RefCell<std::fs::File>>,
    buffer_position: usize,
    out: Out<'v>,
}

impl<'v> FileWriter<'v> {
    pub(crate) fn new(file: Rc<RefCell<std::fs::File>>) -> FileWriter<'v> {
        FileWriter {
            file,
            buffer_position: 0,
            out: Out::Buffer(Box::new([0; BUFFER_SIZE])),
        }
    }

    /// Jumps over the given number of bytes
    ///
    /// Returns a FileWriter object that writes from the current position a
    /// maximum of the given number of bytes.
    pub(crate) fn jump<'vv>(&'vv mut self, len: usize) -> Result<FileWriter<'v>, SkillFail> {
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
                // offset could be used but that would make the position in
                // the log even more useless
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

    /// Returns a FileWriter object that writes from the given offset until
    /// the second given offset from the current position.
    ///
    /// In case the FileWriter uses the buffer this method fails
    pub(crate) fn rel_view(&mut self, from: usize, to: usize) -> Result<FileWriter, SkillFail> {
        match self.out {
            Out::Buffer(_) => Err(SkillFail::internal(InternalFail::ViewOnBuffer)),
            Out::MMap(ref mut map) => Ok(FileWriter {
                file: self.file.clone(),
                buffer_position: 0,
                out: Out::View(&mut map[self.buffer_position + from..self.buffer_position + to]),
            }),
            Out::View(ref mut view) => Ok(FileWriter {
                file: self.file.clone(),
                buffer_position: 0,
                out: Out::View(&mut view[self.buffer_position + from..self.buffer_position + to]),
            }),
        }
    }

    /// Makes sure that at least the given number of bytes are free to write in the buffer
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
            Out::View(ref mut view) => {}
        }
        Ok(())
    }

    /// Flushes the backend
    pub(crate) fn flush(&mut self) -> Result<usize, SkillFail> {
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
            Out::View(ref view) => {
                // We are "flushed" automatically to the mmap we come from
                Ok(view.len())
            }
        }
    }

    // writeing
    // boolean
    pub(crate) fn write_bool(&mut self, what: bool) -> Result<(), SkillFail> {
        self.require_buffer(1)?;
        write_bool(&mut self.buffer_position, &mut self.out, what);
        Ok(())
    }

    // integer types
    pub(crate) fn write_i8(&mut self, what: i8) -> Result<(), SkillFail> {
        self.require_buffer(1)?;
        write_i8(&mut self.buffer_position, &mut self.out, what);
        Ok(())
    }

    pub(crate) fn write_i16(&mut self, what: i16) -> Result<(), SkillFail> {
        self.require_buffer(2)?;
        write_i16(&mut self.buffer_position, &mut self.out, what);
        Ok(())
    }

    pub(crate) fn write_i32(&mut self, what: i32) -> Result<(), SkillFail> {
        self.require_buffer(4)?;
        write_i32(&mut self.buffer_position, &mut self.out, what);
        Ok(())
    }

    pub(crate) fn write_i64(&mut self, what: i64) -> Result<(), SkillFail> {
        self.require_buffer(8)?;
        write_i64(&mut self.buffer_position, &mut self.out, what);
        Ok(())
    }

    pub(crate) fn write_v64(&mut self, what: i64) -> Result<(), SkillFail> {
        self.require_buffer(9)?;
        write_v64(&mut self.buffer_position, &mut self.out, what);
        Ok(())
    }

    // float types
    pub(crate) fn write_f32(&mut self, what: f32) -> Result<(), SkillFail> {
        self.require_buffer(4)?;
        write_f32(&mut self.buffer_position, &mut self.out, what);
        Ok(())
    }

    pub(crate) fn write_f64(&mut self, what: f64) -> Result<(), SkillFail> {
        self.require_buffer(8)?;
        write_f64(&mut self.buffer_position, &mut self.out, what);
        Ok(())
    }

    // string
    pub(crate) fn write_raw_string(&mut self, what: &str) -> Result<(), SkillFail> {
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
            Out::View(ref mut view) => {
                write_string(&mut self.buffer_position, view.as_mut(), what)?
            }
        }
        Ok(())
    }

    pub(crate) fn write_field_type(&mut self, field_type: &FieldType) -> Result<(), SkillFail> {
        match field_type {
            FieldType::BuildIn(BuildInType::ConstTi8(val)) => {
                trace!(target: "SkillWriting", "~~~~FieldType = const i8");
                self.write_i8(0x0)?;
                self.write_i8(*val)?;
            }
            FieldType::BuildIn(BuildInType::ConstTi16(val)) => {
                trace!(target: "SkillWriting", "~~~~FieldType = const i16");
                self.write_i8(0x1)?;
                self.write_i16(*val)?;
            }
            FieldType::BuildIn(BuildInType::ConstTi32(val)) => {
                trace!(target: "SkillWriting", "~~~~FieldType = const i32");
                self.write_i8(0x2)?;
                self.write_i32(*val)?;
            }
            FieldType::BuildIn(BuildInType::ConstTi64(val)) => {
                trace!(target: "SkillWriting", "~~~~FieldType = const i64");
                self.write_i8(0x3)?;
                self.write_i64(*val)?;
            }
            FieldType::BuildIn(BuildInType::ConstTv64(val)) => {
                trace!(target: "SkillWriting", "~~~~FieldType = const v64");
                self.write_i8(0x4)?;
                self.write_v64(*val)?;
            }
            FieldType::BuildIn(BuildInType::Tannotation) => {
                trace!(target: "SkillWriting", "~~~~FieldType = annotation");
                self.write_i8(0x5)?;
            }
            FieldType::BuildIn(BuildInType::Tbool) => {
                trace!(target: "SkillWriting", "~~~~FieldType = bool");
                self.write_i8(0x6)?;
            }
            FieldType::BuildIn(BuildInType::Ti8) => {
                trace!(target: "SkillWriting", "~~~~FieldType = i8");
                self.write_i8(0x7)?;
            }
            FieldType::BuildIn(BuildInType::Ti16) => {
                trace!(target: "SkillWriting", "~~~~FieldType = i16");
                self.write_i8(0x8)?;
            }
            FieldType::BuildIn(BuildInType::Ti32) => {
                trace!(target: "SkillWriting", "~~~~FieldType = i32");
                self.write_i8(0x9)?;
            }
            FieldType::BuildIn(BuildInType::Ti64) => {
                trace!(target: "SkillWriting", "~~~~FieldType = i64");
                self.write_i8(0xA)?;
            }
            FieldType::BuildIn(BuildInType::Tv64) => {
                trace!(target: "SkillWriting", "~~~~FieldType = v64");
                self.write_i8(0xB)?;
            }
            FieldType::BuildIn(BuildInType::Tf32) => {
                trace!(target: "SkillWriting", "~~~~FieldType = f32");
                self.write_i8(0xC)?;
            }
            FieldType::BuildIn(BuildInType::Tf64) => {
                trace!(target: "SkillWriting", "~~~~FieldType = f64");
                self.write_i8(0xD)?;
            }
            FieldType::BuildIn(BuildInType::Tstring) => {
                trace!(target: "SkillWriting", "~~~~FieldType = string");
                self.write_i8(0xE)?;
            }
            FieldType::BuildIn(BuildInType::ConstTarray(length, ref boxed)) => {
                trace!(
                    target: "SkillWriting",
                    "~~~~FieldType = const array length: {:?}",
                    length
                );
                self.write_i8(0xF)?;
                self.write_v64(*length as i64)?;
                self.write_field_type(boxed)?;
            }
            FieldType::BuildIn(BuildInType::Tarray(ref boxed)) => {
                trace!(target: "SkillWriting", "~~~~FieldType = varray");
                self.write_i8(0x11)?;
                self.write_field_type(boxed)?;
            }
            FieldType::BuildIn(BuildInType::Tlist(ref boxed)) => {
                trace!(target: "SkillWriting", "~~~~FieldType = list");
                self.write_i8(0x12)?;
                self.write_field_type(boxed)?;
            }
            FieldType::BuildIn(BuildInType::Tset(ref boxed)) => {
                trace!(target: "SkillWriting", "~~~~FieldType = set");
                self.write_i8(0x13)?;
                self.write_field_type(boxed)?;
            }
            FieldType::BuildIn(BuildInType::Tmap(ref key_boxed, ref val_boxed)) => {
                trace!(target: "SkillWriting", "~~~~FieldType = map");
                self.write_i8(0x14)?;
                self.write_field_type(key_boxed)?;
                self.write_field_type(val_boxed)?;
            }
            FieldType::User(ref user) => {
                let user = user.upgrade().unwrap();
                trace!(
                    target: "SkillWriting",
                    "~~~~FieldType = User {} ID:{}",
                    user.borrow().pool().name().as_str(),
                    user.borrow().pool().get_type_id()
                );
                self.write_v64(user.borrow().pool().get_type_id() as i64)?;
            }
        }
        Ok(())
    }
}

impl<'v> Drop for FileWriter<'v> {
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
