//! This module contains types and functions that are used to handle generic reading
//! and writing of user types
mod base_reader;
mod base_writer;
mod field_declaration;
mod file_reader;
mod file_writer;
pub(crate) mod magic;

pub(crate) use self::base_reader::*;
pub(crate) use self::base_writer::*;
pub use self::field_declaration::FieldDeclaration;
pub(crate) use self::field_declaration::{
    Block, ContinuationFieldChunk, DeclarationFieldChunk, FieldChunk, FieldIO,
};
pub(crate) use self::file_reader::FileReader;
pub(crate) use self::file_writer::FileWriter;
pub(crate) use self::magic::*;
