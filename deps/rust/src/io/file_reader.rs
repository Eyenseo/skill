use common::io::base_reader::*;
use common::Ptr;
use common::SkillError;
use common::StringBlock;

use memmap::Mmap;

use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct FileReader {
    position: Offset,
    end: Offset,
    mmap: Rc<Mmap>,
}

impl From<Rc<Mmap>> for FileReader {
    fn from(mmap: Rc<Mmap>) -> Self {
        let len = Offset::from(mmap.len());
        FileReader {
            position: Offset::from(0),
            end: len,
            mmap,
        }
    }
}

impl FileReader {
    pub fn jump(&mut self, len: Offset) -> FileReader {
        let reader = FileReader {
            position: self.position,
            end: self.position + len,
            mmap: self.mmap.clone(),
        };
        self.position += len;
        reader
    }

    pub fn pos(&self) -> Offset {
        self.position
    }

    pub fn len(&self) -> Offset {
        self.end - self.position
    }

    pub fn is_empty(&self) -> bool {
        self.position >= self.end
    }

    // TODO fastpath for bigedian?
    // Reading
    fn read_byte(&mut self) -> Result<u8, SkillError> {
        if self.position < self.end {
            Err(SkillError::UnexpectedEndOfInput)
        } else {
            Ok(read_byte_unchecked(&mut self.position, &*self.mmap))
        }
    }

    // boolean
    pub fn read_bool(&mut self) -> Result<bool, SkillError> {
        read_bool(&mut self.position, self.end, &*self.mmap)
    }

    // integer types
    pub fn read_i8(&mut self) -> Result<i8, SkillError> {
        read_i8(&mut self.position, self.end, &*self.mmap)
    }

    pub fn read_i16(&mut self) -> Result<i16, SkillError> {
        read_i16(&mut self.position, self.end, &*self.mmap)
    }

    pub fn read_i32(&mut self) -> Result<i32, SkillError> {
        read_i32(&mut self.position, self.end, &*self.mmap)
    }

    pub fn read_i64(&mut self) -> Result<i64, SkillError> {
        read_i64(&mut self.position, self.end, &*self.mmap)
    }

    pub fn read_v64(&mut self) -> Result<i64, SkillError> {
        read_v64(&mut self.position, self.end, &*self.mmap)
    }

    // float types
    pub fn read_f32(&mut self) -> Result<f32, SkillError> {
        read_f32(&mut self.position, self.end, &*self.mmap)
    }

    pub fn read_f64(&mut self) -> Result<f64, SkillError> {
        read_f64(&mut self.position, self.end, &*self.mmap)
    }

    // string
    // TODO replace String with lazy loading
    pub fn read_raw_string(&mut self, length: u32) -> Result<String, SkillError> {
        read_string(&mut self.position, self.end, &*self.mmap, length)
    }

    // compound types
    // are generated
}
