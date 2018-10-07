/*
 * @author Roland Jaeger
 */

use common::error::*;

use failure::Fail;
use memmap::Mmap;

use std::error::Error;
use std::rc::Rc;

// TODO fastpath for bigedian?

/// Lowest level read function that is unchecked -- apart from Rusts checks that panic
fn read_byte_unchecked(position: &mut usize, mmap: &Mmap) -> u8 {
    let ret = unsafe { *mmap.get_unchecked(*position) };
    *position += 1;
    ret
}

/// Function to read a boolean
pub(crate) fn read_bool(position: &mut usize, end: usize, mmap: &Mmap) -> Result<bool, SkillFail> {
    if *position < end {
        let val = read_byte_unchecked(position, mmap) != 0;
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

/// Function to read a i8/u8
pub(crate) fn read_i8(position: &mut usize, end: usize, mmap: &Mmap) -> Result<i8, SkillFail> {
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

/// Function to read a i16/u16
pub(crate) fn read_i16(position: &mut usize, end: usize, mmap: &Mmap) -> Result<i16, SkillFail> {
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

/// Function to read a i32/u32
pub(crate) fn read_i32(position: &mut usize, end: usize, mmap: &Mmap) -> Result<i32, SkillFail> {
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

/// Function to read a i64/u64
pub(crate) fn read_i64(position: &mut usize, end: usize, mmap: &Mmap) -> Result<i64, SkillFail> {
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

/// Function to read a v64
pub(crate) fn read_v64(position: &mut usize, end: usize, mmap: &Mmap) -> Result<i64, SkillFail> {
    let mut val: i64;

    // TODO check if the unrolled loop is indeed needed or the loop is as optimized
    // TODO check if this can be optimized by removing the lambda

    if *position < end && end - *position > 8 {
        let mut read_byte = || read_byte_unchecked(position, mmap).into();

        val = read_byte();
        if val >= 0x80 {
            val = (val & 0x80 - 1) | read_byte() << 7;
            if val >= 0x4000 {
                val = (val & 0x4000 - 1) | read_byte() << 14;
                if val >= 0x200000 {
                    val = (val & 0x200000 - 1) | read_byte() << 21;
                    if val >= 0x10000000 {
                        val = (val & 0x10000000 - 1) | read_byte() << 28;
                        if val >= 0x800000000 {
                            val = (val & 0x800000000 - 1) | read_byte() << 35;
                            if val >= 0x40000000000 {
                                val = (val & 0x40000000000 - 1) | read_byte() << 42;
                                if val >= 0x2000000000000 {
                                    val = (val & 0x2000000000000 - 1) | read_byte() << 49;
                                    if val >= 0x100000000000000 {
                                        val = (val & 0x100000000000000 - 1) | read_byte() << 56;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    } else {
        let mut read_byte = || {
            if *position < end {
                Ok(read_byte_unchecked(position, mmap).into())
            } else {
                Err(SkillFail::internal(InternalFail::UnexpectedEndOfInput))
            }
        };

        val = read_byte()?;
        if val >= 0x80 {
            val = (val & 0x80 - 1) | read_byte()? << 7;
            if val >= 0x4000 {
                val = (val & 0x4000 - 1) | read_byte()? << 14;
                if val >= 0x200000 {
                    val = (val & 0x200000 - 1) | read_byte()? << 21;
                    if val >= 0x10000000 {
                        val = (val & 0x10000000 - 1) | read_byte()? << 28;
                        if val >= 0x800000000 {
                            val = (val & 0x800000000 - 1) | read_byte()? << 35;
                            if val >= 0x40000000000 {
                                val = (val & 0x40000000000 - 1) | read_byte()? << 42;
                                if val >= 0x2000000000000 {
                                    val = (val & 0x2000000000000 - 1) | read_byte()? << 49;
                                    if val >= 0x100000000000000 {
                                        val = (val & 0x100000000000000 - 1) | read_byte()? << 56;
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

/// Function to read a f32
pub(crate) fn read_f32(position: &mut usize, end: usize, mmap: &Mmap) -> Result<f32, SkillFail> {
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

/// Function to read a f64
pub(crate) fn read_f64(position: &mut usize, end: usize, mmap: &Mmap) -> Result<f64, SkillFail> {
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

/// Function to read a string
pub(crate) fn read_string(
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
