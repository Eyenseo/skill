#[macro_use]
pub(crate) mod ptr;

pub mod error;
pub(crate) mod internal;
pub(crate) mod iterator;
pub(crate) mod skill_object;
mod skill_string;
mod string_pool;

pub(crate) use self::ptr::{BorrowError, BorrowMutError, Ref, RefMut, TraitObject};
pub use self::ptr::{Ptr, WeakPtr};
pub(crate) use self::skill_object::Deletable;
pub use self::skill_object::SkillObject;
pub use self::skill_string::SkillString;
pub use self::string_pool::StringPool;
