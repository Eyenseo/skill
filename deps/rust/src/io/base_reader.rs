use common::error::*;

use failure::Fail;
use memmap::Mmap;

use std::error::Error;
use std::rc::Rc;

// TODO fastpath for bigedian?
// Reading
pub(crate) fn read_byte_unchecked(position: &mut usize, mmap: &Mmap) -> u8 {
    let ret = unsafe { *mmap.get_unchecked(*position) };
    *position += 1;
    ret
}

// boolean
pub fn read_bool(position: &mut usize, end: usize, mmap: &Mmap) -> Result<bool, SkillFail> {
    if *position < end {
        let val = read_byte_unchecked(position, mmap) == 0;
        trace!(
            target: "SkillBaseTypeReading",
            "#R# Bool:|{:?}| position:{:?} end:{:?} mmap:{:?}",
            val,
            position,
            end,
            mmap
        );
        Ok(val)
    } else {
        Err(SkillFail::internal(InternalFail::UnexpectedEndOfInput))
    }
}

// integer types
pub fn read_i8(position: &mut usize, end: usize, mmap: &Mmap) -> Result<i8, SkillFail> {
    if *position < end {
        let val = read_byte_unchecked(position, mmap) as i8;
        trace!(
            target: "SkillBaseTypeReading",
            "#R# i8:|{:?}| position:{:?} end:{:?} mmap:{:?}",
            val,
            position,
            end,
            mmap
        );
        Ok(val)
    } else {
        Err(SkillFail::internal(InternalFail::UnexpectedEndOfInput))
    }
}

pub fn read_i16(position: &mut usize, end: usize, mmap: &Mmap) -> Result<i16, SkillFail> {
    if *position + 1 < end {
        let mut val: i16 = (i16::from(read_byte_unchecked(position, mmap))) << 8;
        val |= i16::from(read_byte_unchecked(position, mmap));
        trace!(
            target: "SkillBaseTypeReading",
            "#R# i16:|{:?}| position:{:?} end:{:?} mmap:{:?}",
            val,
            position,
            end,
            mmap
        );
        Ok(val)
    } else {
        Err(SkillFail::internal(InternalFail::UnexpectedEndOfInput))
    }
}

pub fn read_i32(position: &mut usize, end: usize, mmap: &Mmap) -> Result<i32, SkillFail> {
    if *position + 3 < end {
        let mut val: i32 = (i32::from(read_byte_unchecked(position, mmap))) << 24;
        val |= (i32::from(read_byte_unchecked(position, mmap))) << 16;
        val |= (i32::from(read_byte_unchecked(position, mmap))) << 8;
        val |= i32::from(read_byte_unchecked(position, mmap));
        trace!(
            target: "SkillBaseTypeReading",
            "#R# i32:|{:?}| position:{:?} end:{:?} mmap:{:?}",
            val,
            position,
            end,
            mmap
        );
        Ok(val)
    } else {
        Err(SkillFail::internal(InternalFail::UnexpectedEndOfInput))
    }
}

pub fn read_i64(position: &mut usize, end: usize, mmap: &Mmap) -> Result<i64, SkillFail> {
    if *position + 7 < end {
        let mut val: i64 = (i64::from(read_byte_unchecked(position, mmap))) << 56;
        val |= (i64::from(read_byte_unchecked(position, mmap))) << 48;
        val |= (i64::from(read_byte_unchecked(position, mmap))) << 40;
        val |= (i64::from(read_byte_unchecked(position, mmap))) << 32;
        val |= (i64::from(read_byte_unchecked(position, mmap))) << 24;
        val |= (i64::from(read_byte_unchecked(position, mmap))) << 16;
        val |= (i64::from(read_byte_unchecked(position, mmap))) << 8;
        val |= i64::from(read_byte_unchecked(position, mmap));
        trace!(
            target: "SkillBaseTypeReading",
            "#R# i64:|{:?}| position:{:?} end:{:?} mmap:{:?}",
            val,
            position,
            end,
            mmap
        );
        Ok(val)
    } else {
        Err(SkillFail::internal(InternalFail::UnexpectedEndOfInput))
    }
}

pub fn read_v64(position: &mut usize, end: usize, mmap: &Mmap) -> Result<i64, SkillFail> {
    let mut byte_val: i64 = 0;
    let mut val: i64 = 0;
    {
        let mut read_byte = |v: &mut i64| {
            if *position < end {
                *v = read_byte_unchecked(position, mmap).into();
                Ok(*v)
            } else {
                Err(SkillFail::internal(InternalFail::UnexpectedEndOfInput))
            }
        };

        // TODO check if the unrolled loop is indeed needed or the loop is as optimized
        // TODO check if this can be optimized by removing the lambda
        val = read_byte(&mut val)?;
        if val & 0x80 != 0 {
            val = (val & 0x7f) | (read_byte(&mut byte_val)? & 0x7f) << 7;
            if byte_val & 0x80 != 0 {
                val |= (read_byte(&mut byte_val)? & 0x7f) << 14;
                if byte_val & 0x80 != 0 {
                    val |= (read_byte(&mut byte_val)? & 0x7f) << 21;
                    if byte_val & 0x80 != 0 {
                        val |= (read_byte(&mut byte_val)? & 0x7f) << 28;
                        if byte_val & 0x80 != 0 {
                            val |= (read_byte(&mut byte_val)? & 0x7f) << 35;
                            if byte_val & 0x80 != 0 {
                                val |= (read_byte(&mut byte_val)? & 0x7f) << 42;
                                if byte_val & 0x80 != 0 {
                                    val |= (read_byte(&mut byte_val)? & 0x7f) << 49;
                                    if byte_val & 0x80 != 0 {
                                        val |= read_byte(&mut byte_val)? << 56;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    trace!(
        target: "SkillBaseTypeReading",
        "#R# v64:|{:?}| position:{:?} end:{:?} mmap:{:?}",
        val,
        position,
        end,
        mmap
    );
    Ok(val)
}

// float types
pub fn read_f32(position: &mut usize, end: usize, mmap: &Mmap) -> Result<f32, SkillFail> {
    #[repr(C)]
    union U {
        i: i32,
        f: f32,
    };
    let tmp = U {
        i: read_i32(position, end, mmap)?,
    };

    unsafe {
        trace!(
            target: "SkillBaseTypeReading",
            "#R# i32=float:|{:?}| position:{:?} end:{:?} mmap:{:?}",
            tmp.f,
            position,
            end,
            mmap
        );
        Ok(tmp.f)
    }
}

pub fn read_f64(position: &mut usize, end: usize, mmap: &Mmap) -> Result<f64, SkillFail> {
    #[repr(C)]
    union U {
        i: i64,
        f: f64,
    };
    let tmp = U {
        i: read_i64(position, end, mmap)?,
    };
    unsafe {
        trace!(
            target: "SkillBaseTypeReading",
            "#R# i64=double:|{:?}| position:{:?} end:{:?} mmap:{:?}",
            tmp.f,
            position,
            end,
            mmap
        );
        Ok(tmp.f)
    }
}

// string
// TODO replace String with lazy loading
pub fn read_string(
    position: &mut usize,
    end: usize,
    mmap: &Mmap,
    length: u32,
) -> Result<String, SkillFail> {
    let end_offset = *position + length as usize;

    if end_offset > end {
        return Err(SkillFail::internal(InternalFail::UnexpectedEndOfInput));
    }

    match String::from_utf8(mmap[*position..end_offset].to_vec()) {
        Ok(s) => {
            *position = end_offset;
            trace!(
                target: "SkillBaseTypeReading",
                "#R# str:|{:?}| position:{:?} end:{:?} mmap:{:?}",
                s,
                position,
                end,
                mmap
            );
            Ok(s)
        }
        Err(e) => Err(SkillFail::internal(InternalFail::StringDeserialization {
            why: e.description().to_owned(),
        })),
    }
}
