pub mod magic;

pub mod base_reader;
pub mod field_reader;
pub mod file_reader;

pub use self::base_reader::*;
pub use self::magic::*;

pub use self::field_reader::{
    Block, BlockIndex, ContinuationFieldChunk, DeclarationFieldChunk, FieldChunk, FieldReader,
};
pub use self::file_reader::FileReader;
