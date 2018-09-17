/*
 * @author Roland Jaeger
 */

use common::error::*;
use common::internal::io::base_reader::*;
use common::internal::io::magic::*;
use common::internal::*;
use common::Ptr;

use memmap::Mmap;

use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
pub(crate) struct FileReader {
    position: usize,
    end: usize,
    mmap: Rc<Mmap>,
}

impl From<Rc<Mmap>> for FileReader {
    fn from(mmap: Rc<Mmap>) -> Self {
        let len = mmap.len();
        FileReader {
            position: 0,
            end: len,
            mmap,
        }
    }
}

impl FileReader {
    pub(crate) fn jump(&mut self, len: usize) -> FileReader {
        let reader = FileReader {
            position: self.position,
            end: self.position + len,
            mmap: self.mmap.clone(),
        };
        self.position += len;
        reader
    }

    pub(crate) fn rel_view(&self, from: usize, to: usize) -> FileReader {
        FileReader {
            position: self.position + from,
            end: self.position + to,
            mmap: self.mmap.clone(),
        }
    }

    pub(crate) fn pos(&self) -> usize {
        self.position
    }

    pub(crate) fn len(&self) -> usize {
        self.end - self.position
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.position >= self.end
    }

    // Reading
    // boolean
    pub(crate) fn read_bool(&mut self) -> Result<bool, SkillFail> {
        read_bool(&mut self.position, self.end, &*self.mmap)
    }

    // integer types
    pub(crate) fn read_i8(&mut self) -> Result<i8, SkillFail> {
        read_i8(&mut self.position, self.end, &*self.mmap)
    }

    pub(crate) fn read_i16(&mut self) -> Result<i16, SkillFail> {
        read_i16(&mut self.position, self.end, &*self.mmap)
    }

    pub(crate) fn read_i32(&mut self) -> Result<i32, SkillFail> {
        read_i32(&mut self.position, self.end, &*self.mmap)
    }

    pub(crate) fn read_i64(&mut self) -> Result<i64, SkillFail> {
        read_i64(&mut self.position, self.end, &*self.mmap)
    }

    pub(crate) fn read_v64(&mut self) -> Result<i64, SkillFail> {
        read_v64(&mut self.position, self.end, &*self.mmap)
    }

    // float types
    pub(crate) fn read_f32(&mut self) -> Result<f32, SkillFail> {
        read_f32(&mut self.position, self.end, &*self.mmap)
    }

    pub(crate) fn read_f64(&mut self) -> Result<f64, SkillFail> {
        read_f64(&mut self.position, self.end, &*self.mmap)
    }

    // string
    // TODO replace String with lazy loading
    pub(crate) fn read_raw_string(&mut self, length: u32) -> Result<String, SkillFail> {
        read_string(&mut self.position, self.end, &*self.mmap, length)
    }

    pub(crate) fn read_field_type(
        &mut self,
        pools: &Vec<Rc<RefCell<PoolProxy>>>,
    ) -> Result<FieldType, SkillFail> {
        let field_type = self.read_v64()?; // type of field

        //TODO add from for the enum and use that to match and throw an error?
        match field_type {
            0x0 => {
                self.read_i8()?;
                trace!(target: "SkillParsing", "~~~~FieldType = const i8");
                Ok(FieldType::BuildIn(BuildInType::ConstTi8))
            }
            0x1 => {
                self.read_i16()?;
                trace!(target: "SkillParsing", "~~~~FieldType = const i16");
                Ok(FieldType::BuildIn(BuildInType::ConstTi16))
            }
            0x2 => {
                self.read_i32()?;
                trace!(target: "SkillParsing", "~~~~FieldType = const i32");
                Ok(FieldType::BuildIn(BuildInType::ConstTi32))
            }
            0x3 => {
                self.read_i64()?;
                trace!(target: "SkillParsing", "~~~~FieldType = const i64");
                Ok(FieldType::BuildIn(BuildInType::ConstTi64))
            }
            0x4 => {
                self.read_v64()?;
                trace!(target: "SkillParsing", "~~~~FieldType = const v64");
                Ok(FieldType::BuildIn(BuildInType::ConstTv64))
            }
            0x5 => {
                trace!(target: "SkillParsing", "~~~~FieldType = annotation");
                Ok(FieldType::BuildIn(BuildInType::Tannotation))
            }
            0x6 => {
                trace!(target: "SkillParsing", "~~~~FieldType = bool");
                Ok(FieldType::BuildIn(BuildInType::Tbool))
            }
            0x7 => {
                trace!(target: "SkillParsing", "~~~~FieldType = i8");
                Ok(FieldType::BuildIn(BuildInType::Ti8))
            }
            0x8 => {
                trace!(target: "SkillParsing", "~~~~FieldType = i16");
                Ok(FieldType::BuildIn(BuildInType::Ti16))
            }
            0x9 => {
                trace!(target: "SkillParsing", "~~~~FieldType = i32");
                Ok(FieldType::BuildIn(BuildInType::Ti32))
            }
            0xA => {
                trace!(target: "SkillParsing", "~~~~FieldType = i64");
                Ok(FieldType::BuildIn(BuildInType::Ti64))
            }
            0xB => {
                trace!(target: "SkillParsing", "~~~~FieldType = v64");
                Ok(FieldType::BuildIn(BuildInType::Tv64))
            }
            0xC => {
                trace!(target: "SkillParsing", "~~~~FieldType = f32");
                Ok(FieldType::BuildIn(BuildInType::Tf32))
            }
            0xD => {
                trace!(target: "SkillParsing", "~~~~FieldType = f64");
                Ok(FieldType::BuildIn(BuildInType::Tf64))
            }
            0xE => {
                trace!(target: "SkillParsing", "~~~~FieldType = string");
                Ok(FieldType::BuildIn(BuildInType::Tstring))
            }
            0xF => {
                let length = self.read_v64()? as u64;
                trace!(target: "SkillParsing", "~~~~FieldType = const array length: {:?}", length);
                Ok(FieldType::BuildIn(BuildInType::ConstTarray(
                    length,
                    Box::new(self.read_field_type(pools)?),
                )))
            }
            0x11 => {
                trace!(target: "SkillParsing", "~~~~FieldType = varray");
                Ok(FieldType::BuildIn(BuildInType::Tarray(Box::new(
                    self.read_field_type(pools)?,
                ))))
            }
            0x12 => {
                trace!(target: "SkillParsing", "~~~~FieldType = list");
                Ok(FieldType::BuildIn(BuildInType::Tlist(Box::new(
                    self.read_field_type(pools)?,
                ))))
            }
            0x13 => {
                trace!(target: "SkillParsing", "~~~~FieldType = set");
                Ok(FieldType::BuildIn(BuildInType::Tset(Box::new(
                    self.read_field_type(pools)?,
                ))))
            }
            0x14 => {
                trace!(target: "SkillParsing", "~~~~FieldType = map");
                Ok(FieldType::BuildIn(BuildInType::Tmap(
                    Box::new(self.read_field_type(pools)?),
                    Box::new(self.read_field_type(pools)?),
                )))
            }
            user => {
                if user < 32 {
                    return Err(SkillFail::internal(InternalFail::ReservedID {
                        id: user as usize,
                    }));
                }
                trace!(target: "SkillParsing", "~~~~FieldType = User ID {:?}", user);
                Ok(FieldType::User(Rc::downgrade(&pools[user as usize - 32])))
            }
        }
    }
}
