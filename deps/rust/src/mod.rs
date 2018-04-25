#[macro_use]
pub mod ptr;

mod string_pool;
mod type_pool;

mod error;
mod skill_file;
pub mod internal;
pub mod io;

pub use self::error::SkillError;
pub use self::ptr::{BorrowError, BorrowMutError, Ptr, Ref, RefMut, TraitObject, WeakPtr};
pub use self::skill_file::PoolMaker;
pub use self::skill_file::SkillFile;
pub use self::string_pool::StringBlock;
pub use self::type_pool::TypeBlock;

