//! Common modules and types for all bindings.
//!
//! # Note
//! This code can not be separated into a standalone crate as `Ptr` depends on generated code

#[macro_use]
pub(crate) mod ptr;

mod skill_string;
mod string_pool;
pub(crate) mod internal;
pub(crate) mod skill_object;
pub mod error;
pub mod iterator;
pub mod foreign;

pub(crate) use self::skill_object::Deletable;
pub use self::ptr::{Ptr, WeakPtr, BorrowError, BorrowMutError, Ref, RefMut};
pub use self::skill_object::SkillObject;
pub use self::skill_string::SkillString;
pub use self::string_pool::StringPool;
