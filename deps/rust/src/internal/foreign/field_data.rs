/*
 * @author Roland Jaeger
 */

use common::*;

use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

/// Used to manage field values that where not known at compile time
pub enum FieldData {
    Bool(bool),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
    String(Option<Rc<SkillString>>),
    Set(HashSet<FieldData>),
    Map(HashMap<FieldData, FieldData>),
    Array(Vec<FieldData>),
    User(Option<WeakPtr<SkillObject>>),
}

impl PartialEq for FieldData {
    #[inline(always)]
    fn eq(&self, other: &FieldData) -> bool {
        match &self {
            FieldData::Bool(val) => match other {
                FieldData::Bool(oval) => val == oval,
                _ => false,
            },
            FieldData::I8(val) => match other {
                FieldData::I8(oval) => val == oval,
                _ => false,
            },
            FieldData::I16(val) => match other {
                FieldData::I16(oval) => val == oval,
                _ => false,
            },
            FieldData::I32(val) => match other {
                FieldData::I32(oval) => val == oval,
                _ => false,
            },
            FieldData::I64(val) => match other {
                FieldData::I64(oval) => val == oval,
                _ => false,
            },
            FieldData::F32(val) => match other {
                FieldData::F32(oval) => *val as u32 == *oval as u32,
                _ => false,
            },
            FieldData::F64(val) => match other {
                FieldData::F64(oval) => *val as u64 == *oval as u64,
                _ => false,
            },
            FieldData::String(val) => match other {
                FieldData::String(oval) => val == oval,
                _ => false,
            },
            FieldData::Set(val) => match other {
                FieldData::Set(oval) => val == oval,
                _ => false,
            },
            FieldData::Map(val) => match other {
                FieldData::Map(oval) => val == oval,
                _ => false,
            },
            FieldData::Array(val) => match other {
                FieldData::Array(oval) => val == oval,
                _ => false,
            },
            FieldData::User(val) => match other {
                FieldData::User(oval) => val == oval,
                _ => false,
            },
        }
    }
}

impl Eq for FieldData {}

impl Hash for FieldData {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match &self {
            FieldData::Bool(val) => val.hash(state),
            FieldData::I8(val) => val.hash(state),
            FieldData::I16(val) => val.hash(state),
            FieldData::I32(val) => val.hash(state),
            FieldData::I64(val) => val.hash(state),
            FieldData::F32(val) => (*val as u32).hash(state),
            FieldData::F64(val) => (*val as u64).hash(state),
            FieldData::String(val) => val.hash(state),
            FieldData::Set(val) => (self as *const FieldData).hash(state),
            FieldData::Map(val) => (self as *const FieldData).hash(state),
            FieldData::Array(val) => (self as *const FieldData).hash(state),
            FieldData::User(val) => val.hash(state),
        }
    }
}

impl fmt::Debug for FieldData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Foreign Data")
    }
}
