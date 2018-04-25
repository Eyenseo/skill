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
    User(Rc<RefCell<InstancePool>>, usize),
}

impl fmt::Display for FieldType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FieldType::BuildIn(build_in) => write!(f, "{}", build_in),
            FieldType::User(pool, user) => write!(f, "User{}", user), // TODO try to get the name for the specific user type?
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
