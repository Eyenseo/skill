#[macro_use]
pub mod ptr;

mod string_pool;
mod type_pool;

mod book;
mod error;
pub mod internal;
pub mod io;
mod skill_file;
mod skill_string;

pub use self::book::Book;
pub use self::error::SkillError;
pub use self::ptr::{BorrowError, BorrowMutError, Ptr, Ref, RefMut, TraitObject, WeakPtr};
pub use self::skill_file::PoolMaker;
pub use self::skill_file::SkillFile;
pub use self::skill_string::SkillString;
pub use self::string_pool::StringBlock;
pub use self::type_pool::TypeBlock;
