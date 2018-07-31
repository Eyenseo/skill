use common::error::*;

use std::error::Error;
use std::io::Write;
use std::ops::{Add, AddAssign, Sub, SubAssign};
use std::rc::Rc;

// TODO fastpath for bigedian?
// writing
pub(crate) fn write_byte_unchecked(position: &mut usize, out: &mut [u8], what: u8) {
    out[*position] = what;
    *position += 1;
}

// boolean
pub fn write_bool(position: &mut usize, out: &mut [u8], what: bool) {
    trace!(
        target: "SkillBaseTypewriting",
        "#W# Bool:|{:?}| position:{:?} out:{:?}",
        what,
        position,
        out
    );
    write_byte_unchecked(position, out, what as u8);
}

// integer types
pub fn write_i8(position: &mut usize, out: &mut [u8], what: i8) {
    trace!(
        target: "SkillBaseTypewriting",
        "#W# i8:|{:?}| position:{:?} out:{:?}",
        what,
        position,
        out
    );
    write_byte_unchecked(position, out, what as u8);
}

pub fn write_i16(position: &mut usize, out: &mut [u8], what: i16) {
    trace!(
        target: "SkillBaseTypewriting",
        "#W# i16:|{:?}| position:{:?} out:{:?}",
        what,
        position,
        out
    );
    write_byte_unchecked(position, out, (what >> 8) as u8);
    write_byte_unchecked(position, out, (what) as u8);
}

pub fn write_i32(position: &mut usize, out: &mut [u8], what: i32) {
    trace!(
        target: "SkillBaseTypewriting",
        "#W# i32:|{:?}| position:{:?} out:{:?}",
        what,
        position,
        out
    );
    write_byte_unchecked(position, out, (what >> 24) as u8);
    write_byte_unchecked(position, out, (what >> 16) as u8);
    write_byte_unchecked(position, out, (what >> 8) as u8);
    write_byte_unchecked(position, out, what as u8);
}

pub fn write_i64(position: &mut usize, out: &mut [u8], what: i64) {
    trace!(
        target: "SkillBaseTypewriting",
        "#W# i64:|{:?}| position:{:?} out:{:?}",
        what,
        position,
        out
    );
    write_byte_unchecked(position, out, (what >> 56) as u8);
    write_byte_unchecked(position, out, (what >> 48) as u8);
    write_byte_unchecked(position, out, (what >> 40) as u8);
    write_byte_unchecked(position, out, (what >> 32) as u8);
    write_byte_unchecked(position, out, (what >> 24) as u8);
    write_byte_unchecked(position, out, (what >> 16) as u8);
    write_byte_unchecked(position, out, (what >> 8) as u8);
    write_byte_unchecked(position, out, what as u8);
}

pub fn write_v64(position: &mut usize, out: &mut [u8], what: i64) {
    trace!(
        target: "SkillBaseTypewriting",
        "#W# v64:|{:?}| position:{:?} out:{:?}",
        what,
        position,
        out
    );

    // TODO check out of bounds?
    // is this handeld through rust as rust will check the bounds of the slice ?
    if (what as u64) < 0x80 {
        write_byte_unchecked(position, out, what as u8);
    } else {
        write_byte_unchecked(position, out, ((what) | 0x80) as u8);
        if (what as u64) < 0x4000 {
            write_byte_unchecked(position, out, (what >> 7) as u8);
        } else {
            write_byte_unchecked(position, out, ((what >> 7) | 0x80) as u8);
            if (what as u64) < 0x200000 {
                write_byte_unchecked(position, out, (what >> 14) as u8);
            } else {
                write_byte_unchecked(position, out, ((what >> 14) | 0x80) as u8);
                if (what as u64) < 0x10000000 {
                    write_byte_unchecked(position, out, (what >> 21) as u8);
                } else {
                    write_byte_unchecked(position, out, ((what >> 21) | 0x80) as u8);
                    if (what as u64) < 0x800000000 {
                        write_byte_unchecked(position, out, (what >> 28) as u8);
                    } else {
                        write_byte_unchecked(position, out, ((what >> 28) | 0x80) as u8);
                        if (what as u64) < 0x40000000000 {
                            write_byte_unchecked(position, out, (what >> 35) as u8);
                        } else {
                            write_byte_unchecked(position, out, ((what >> 35) | 0x80) as u8);
                            if (what as u64) < 0x2000000000000 {
                                write_byte_unchecked(position, out, (what >> 42) as u8);
                            } else {
                                write_byte_unchecked(position, out, ((what >> 42) | 0x80) as u8);
                                if (what as u64) < 0x100000000000000 {
                                    write_byte_unchecked(position, out, (what >> 49) as u8);
                                } else {
                                    write_byte_unchecked(
                                        position,
                                        out,
                                        ((what >> 49) | 0x80) as u8,
                                    );
                                    write_byte_unchecked(position, out, (what >> 56) as u8);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// float types
pub fn write_f32(position: &mut usize, out: &mut [u8], what: f32) {
    trace!(
        target: "SkillBaseTypewriting",
        "#W# i32=float:|{:?}| position:{:?} out:{:?}",
        what,
        position,
        out
    );
    #[repr(C)]
    union U {
        i: i32,
        f: f32,
    };
    write_i32(position, out, unsafe { U { f: what }.i });
}

pub fn write_f64(position: &mut usize, out: &mut [u8], what: f64) {
    trace!(
        target: "SkillBaseTypewriting",
        "#W# i64=double:|{:?}| position:{:?} out:{:?}",
        what,
        position,
        out
    );
    #[repr(C)]
    union U {
        i: i64,
        f: f64,
    };
    write_i64(position, out, unsafe { U { f: what }.i });
}

// string
// TODO replace String with lazy loading
pub fn write_string(position: &mut usize, out: &mut [u8], what: &str) -> Result<(), SkillFail> {
    trace!(
        target: "SkillBaseTypewriting",
        "#W# str:|{:?}| position:{:?} out:{:?}",
        what,
        position,
        out
    );
    match (&mut out[*position..]).write_all(what.as_bytes()) {
        Ok(_) => {}
        Err(e) => {
            return Err(SkillFail::internal(InternalFail::BadWrite {
                why: e.description().to_owned(),
            }))
        }
    }
    *position += what.len();
    Ok(())
}
