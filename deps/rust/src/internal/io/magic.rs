/*
 * @author Roland Jaeger
 */

use common::error::*;
use common::internal::io::*;
use common::internal::PoolProxy;

use std::cell::RefCell;
use std::fmt;
use std::rc::{Rc, Weak};

/// Enum that represents the different build-in types
pub(crate) enum BuildInType {
    ConstTi8(i8),
    ConstTi16(i16),
    ConstTi32(i32),
    ConstTi64(i64),
    ConstTv64(i64),
    Tannotation,
    Tbool,
    Ti8,
    Ti16,
    Ti32,
    Ti64,
    Tv64,
    Tf32,
    Tf64,
    Tstring,
    ConstTarray(u64, Box<FieldType>),
    Tarray(Box<FieldType>),
    Tlist(Box<FieldType>),
    Tset(Box<FieldType>),
    // TODO this should be changed to a vec - the encapsulation is not needed
    Tmap(Box<FieldType>, Box<FieldType>),
    // NOTE user types start from >32
}

impl fmt::Display for BuildInType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BuildInType::ConstTi8(val) => write!(f, "ConstTi8:{}", val),
            BuildInType::ConstTi16(val) => write!(f, "ConstTi16:{}", val),
            BuildInType::ConstTi32(val) => write!(f, "ConstTi32:{}", val),
            BuildInType::ConstTi64(val) => write!(f, "ConstTi64:{}", val),
            BuildInType::ConstTv64(val) => write!(f, "ConstTv64:{}", val),
            BuildInType::Tannotation => f.write_str("Tannotation"),
            BuildInType::Tbool => f.write_str("Tbool"),
            BuildInType::Ti8 => f.write_str("Ti8"),
            BuildInType::Ti16 => f.write_str("Ti16"),
            BuildInType::Ti32 => f.write_str("Ti32"),
            BuildInType::Ti64 => f.write_str("Ti64"),
            BuildInType::Tv64 => f.write_str("Tv64"),
            BuildInType::Tf32 => f.write_str("Tf32"),
            BuildInType::Tf64 => f.write_str("Tf64"),
            BuildInType::Tstring => f.write_str("Tstring"),
            BuildInType::ConstTarray(length, box_v) => write!(f, "{}[{}]", length, *box_v),
            BuildInType::Tarray(box_v) => write!(f, "v[{}]", *box_v),
            BuildInType::Tlist(box_v) => write!(f, "List[{}]", *box_v),
            BuildInType::Tset(box_v) => write!(f, "Set{{{}}}", *box_v),
            BuildInType::Tmap(key_box_v, box_v) => write!(f, "Map{{{},{}}}", *key_box_v, *box_v),
        }
    }
}

/// enum that is used to deserialize data, in case of Build in types the
/// right methods have to be selected. In case a user type has to be read
/// the correct Pool to get the instance from is provided
pub(crate) enum FieldType {
    BuildIn(BuildInType),
    User(Weak<RefCell<PoolProxy>>),
}

impl FieldType {
    /// used to parse restrictions correctly -- this code only reads without
    /// creating usable information about the restrictions. To implement
    /// restrictions the read data has to be processed further...
    pub(crate) fn read(&self, reader: &mut FileReader) -> Result<(), SkillFail> {
        match self {
            FieldType::BuildIn(ref field) => match field {
                BuildInType::ConstTi8(val) => {
                    debug!(
                        target: "SkillParsing",
                        "~~~~~FieldRestriction::read ConstTi8:{}",
                        val
                    );
                    Ok(())
                }
                BuildInType::ConstTi16(val) => {
                    debug!(
                        target: "SkillParsing",
                        "~~~~~FieldRestriction::read ConstTi16:{}",
                        val
                    );
                    Ok(())
                }
                BuildInType::ConstTi32(val) => {
                    debug!(
                        target: "SkillParsing",
                        "~~~~~FieldRestriction::read ConstTi32:{}",
                        val
                    );
                    Ok(())
                }
                BuildInType::ConstTi64(val) => {
                    debug!(
                        target: "SkillParsing",
                        "~~~~~FieldRestriction::read ConstTi64:{}",
                        val
                    );
                    Ok(())
                }
                BuildInType::ConstTv64(val) => {
                    debug!(
                        target: "SkillParsing",
                        "~~~~~FieldRestriction::read ConstTv64:{}",
                        val
                    );
                    Ok(())
                }
                BuildInType::Tbool => {
                    debug!(
                        target: "SkillParsing",
                        "~~~~~FieldRestriction::read Tbool "
                    );
                    reader.read_bool()?;
                    Ok(())
                }
                BuildInType::Ti8 => {
                    debug!(
                        target: "SkillParsing",
                        "~~~~~FieldRestriction::read Ti8 "
                    );
                    reader.read_i8()?;
                    Ok(())
                }
                BuildInType::Ti16 => {
                    debug!(
                        target: "SkillParsing",
                        "~~~~~FieldRestriction::read Ti16 "
                    );
                    reader.read_i16()?;
                    Ok(())
                }
                BuildInType::Ti32 => {
                    debug!(
                        target: "SkillParsing",
                        "~~~~~FieldRestriction::read Ti32 "
                    );
                    reader.read_i32()?;
                    Ok(())
                }
                BuildInType::Ti64 => {
                    debug!(
                        target: "SkillParsing",
                        "~~~~~FieldRestriction::read Ti64 "
                    );
                    reader.read_i64()?;
                    Ok(())
                }
                BuildInType::Tv64 => {
                    debug!(
                        target: "SkillParsing",
                        "~~~~~FieldRestriction::read Tv64 "
                    );
                    reader.read_v64()?;
                    Ok(())
                }
                BuildInType::Tf32 => {
                    debug!(
                        target: "SkillParsing",
                        "~~~~~FieldRestriction::read Tf32 "
                    );
                    reader.read_f32()?;
                    Ok(())
                }
                BuildInType::Tf64 => {
                    debug!(
                        target: "SkillParsing",
                        "~~~~~FieldRestriction::read Tf64 "
                    );
                    reader.read_f64()?;
                    Ok(())
                }
                BuildInType::Tannotation => {
                    debug!(
                        target: "SkillParsing",
                        "~~~~~FieldRestriction::read Tannotation "
                    );
                    reader.read_v64()?;
                    reader.read_v64()?;
                    Ok(())
                }
                BuildInType::Tstring => {
                    debug!(
                        target: "SkillParsing",
                        "~~~~~FieldRestriction::read Tstring "
                    );
                    reader.read_v64()?;
                    Ok(())
                }
                BuildInType::ConstTarray(length, box_v) => {
                    debug!(
                        target: "SkillParsing",
                        "~~~~~FieldRestriction::read ConstTarray length:{}",
                        length
                    );
                    for i in 0..*length as usize {
                        box_v.read(reader)?;
                    }
                    Ok(())
                }
                BuildInType::Tarray(box_v) => {
                    let elements = reader.read_v64()? as usize;
                    debug!(
                        target: "SkillParsing",
                        "~~~~~FieldRestriction::read Tarray length:{}",
                        elements
                    );
                    for _ in 0..elements {
                        box_v.read(reader)?;
                    }
                    Ok(())
                }
                BuildInType::Tlist(box_v) => {
                    let elements = reader.read_v64()? as usize;
                    debug!(
                        target: "SkillParsing",
                        "~~~~~FieldRestriction::read Tlist length:{}",
                        elements
                    );
                    for _ in 0..elements {
                        box_v.read(reader)?;
                    }
                    Ok(())
                }
                BuildInType::Tset(box_v) => {
                    let elements = reader.read_v64()? as usize;
                    debug!(
                        target: "SkillParsing",
                        "~~~~~FieldRestriction::read Tset length:{}",
                        elements
                    );
                    for _ in 0..elements {
                        box_v.read(reader)?;
                    }
                    Ok(())
                }
                BuildInType::Tmap(key_box_v, box_v) => {
                    let elements = reader.read_v64()? as usize;
                    debug!(
                        target: "SkillParsing",
                        "~~~~~FieldRestriction::read Tmap length:{}",
                        elements
                    );
                    for _ in 0..elements {
                        key_box_v.read(reader)?;
                        box_v.read(reader)?;
                    }
                    Ok(())
                }
            },
            FieldType::User(ref pool) => {
                reader.read_v64()?;
                Ok(())
            }
        }
    }
}

/// calculates the bytes a v64 needs to be written to file
pub(crate) fn bytes_v64(what: i64) -> usize {
    if (what as u64) < 0x80 {
        1
    } else if (what as u64) < 0x4000 {
        2
    } else if (what as u64) < 0x200000 {
        3
    } else if (what as u64) < 0x10000000 {
        4
    } else if (what as u64) < 0x800000000 {
        5
    } else if (what as u64) < 0x40000000000 {
        6
    } else if (what as u64) < 0x2000000000000 {
        7
    } else if (what as u64) < 0x100000000000000 {
        8
    } else {
        9
    }
}

impl fmt::Display for FieldType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FieldType::BuildIn(build_in) => write!(f, "{}", build_in),
            FieldType::User(pool) => match pool.upgrade().unwrap().try_borrow() {
                Ok(pool) => write!(f, "User{}", pool.pool().name().as_str()),
                Err(_) => {
                    write!(
                        f,
                        "Some UserType - the pool is borrowed \
                         mutable so there is no more information \
                         available than its pointer: {:?} Good Luck!",
                        pool as *const Weak<RefCell<PoolProxy>>,
                        // NOTE use with care! It destroyes all guarantees
                        // unsafe { (*pool.upgrade().unwrap().as_ptr()).pool().name().as_str() }
                    )
                }
            },
        }
    }
}

/// enum that represents type restrictions in case restrictions are
/// implemented this will have to be adjusted
enum TypeRestrictions {
    Runique,
    Rsingleton,
    Rmonotone,
    Rabstract,
    Rdefault,
}

/// enum that represents field restrictions in case restrictions are
/// implemented this will have to be adjusted
enum FieldRestrictions {
    RnonNull,
    Rdefault,
    Rrange,
    Rcoding,
    RconstantLengthPointer,
    RoneOf,
}
