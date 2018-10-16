/*
 * @author Roland Jaeger
 */


#[macro_use]
pub(crate) mod ptr;

mod skill_string;
mod string_pool;
pub(crate) mod internal;
pub(crate) mod skill_object;
pub mod error;
pub mod iterator;

pub(crate) use self::skill_object::Deletable;
pub use self::ptr::{Ptr, WeakPtr, BorrowError, BorrowMutError, Ref, RefMut};
pub use self::skill_object::SkillObject;
pub use self::skill_string::SkillString;
pub use self::string_pool::StringPool;
pub use self::internal::foreign;
