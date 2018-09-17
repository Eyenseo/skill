/*
 * @author Roland Jaeger
 */


#[macro_use]
pub(crate) mod ptr;

pub mod error;
pub mod iterator;
pub(crate) mod internal;
pub(crate) mod skill_object;
mod skill_string;
mod string_pool;

pub use self::ptr::{Ptr, WeakPtr, BorrowError, BorrowMutError, Ref, RefMut};
pub use self::skill_object::SkillObject;
pub use self::skill_string::SkillString;
pub use self::string_pool::StringPool;
pub(crate) use self::skill_object::Deletable;
