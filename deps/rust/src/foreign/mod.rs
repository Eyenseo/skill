//! Foreign types are used to provide the user with an API that enables interaction with user types
//! that were not known at generation / compile time.
//!
//! # Note
//! The types in this module can be roughly compared to the _Unknown_ types from C++.

mod field_data;
mod field_io;
mod object;
mod pool;

pub use self::field_data::FieldData;
pub(crate) use self::field_io::FieldIO;
pub use self::object::{Foreign, ForeignObject};
pub use self::pool::Pool;
