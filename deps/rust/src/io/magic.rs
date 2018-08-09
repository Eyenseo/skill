use common::internal::InstancePool;

use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

pub enum BuildInType {
    ConstTi8,
    ConstTi16,
    ConstTi32,
    ConstTi64,
    ConstTv64,
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
    // TODO this should be changed to a vec - the encapulation is not needed
    Tmap(Box<FieldType>, Box<FieldType>),
    // NOTE user types start from >32
}

impl fmt::Display for BuildInType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BuildInType::ConstTi8 => f.write_str("ConstTi8"),
            BuildInType::ConstTi16 => f.write_str("ConstTi16"),
            BuildInType::ConstTi32 => f.write_str("ConstTi32"),
            BuildInType::ConstTi64 => f.write_str("ConstTi64"),
            BuildInType::ConstTv64 => f.write_str("ConstTv64"),
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

pub enum FieldType {
    BuildIn(BuildInType),
    User(Rc<RefCell<InstancePool>>),
}

pub fn bytes_v64(what: i64) -> usize {
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
            FieldType::User(pool) => match pool.try_borrow() {
                Ok(pool) => write!(f, "User{}", pool.name().as_str()),
                Err(_) => {
                    write!(
                        f,
                        "Some UserType - the pool is borrowed \
                         mutable so there is no more information \
                         available than its pointer: {:?} Good Luck!",
                        pool as *const Rc<RefCell<InstancePool>>,
                        // NOTE use with care! It destroyes all guarantees
                        // unsafe { (*pool.as_ptr()).name().as_str() }
                    )
                }
            },
        }
    }
}

enum TypeRestrictions {
    Runique,
    Rsingleton,
    Rmonotone,
    Rabstract,
    Rdefault,
}

enum FieldRestrictions {
    RnonNull,
    Rdefault,
    Rrange,
    Rcoding,
    RconstantLengthPointer,
    RoneOf,
}
