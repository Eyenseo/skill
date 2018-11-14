//! Error management through failure
//!
//! There are three enums:
//! - `InternalFail` means that an error was caused by something internal. This should not happen.
//! - `UserFail` means that the user is most probably at fault by using the binding wrong.
//! - `SkillFail` contains a value of either an is used to collapse the verbosity of the different errors into a single type.

use failure::{Backtrace, Fail};

/// Indicates that an error was caused by something internal. This should not happen.
#[derive(Fail, Debug)]
pub enum InternalFail {
    // TODO sort the errors
    #[fail(display = "The input stream ended even though more input was expected")]
    UnexpectedEndOfInput,
    #[fail(display = "The read string was too short, probably because the input ended to early.")]
    StringTooShort,
    #[fail(display = "deserialization failed because:{}", why)]
    StringDeserialization { why: String },

    #[fail(display = "The type '{}' was declared more than once.", name)]
    RedefinitionOfType { name: String },
    #[fail(display = "The type is not implemented.")]
    UnimplementedType,
    #[fail(display = "The type with id:{} is unknown", id)]
    UnknownType { id: usize },
    #[fail(display = "The Field with id:{} is unknown", id)]
    UnknownField { id: usize },
    #[fail(display = "The type restriction:{} is unknown", id)]
    UnknownTypeRestriction { id: usize },
    #[fail(display = "The field restriction:{} is unknown", id)]
    UnknownFieldRestriction { id: usize },
    #[fail(
        display = "Unordered type block; previous ID:{} < current ID:{} with type: {}",
        previous,
        current,
        name
    )]
    UnorderedTypeBlock {
        previous: usize,
        current: usize,
        name: String,
    },
    #[fail(
        display = "Found bad base pool offset of:{} super lbpo:{}",
        local_bpo,
        super_local_bpo
    )]
    BadBasePoolOffset {
        local_bpo: usize,
        super_local_bpo: usize,
    },
    #[fail(
        display = "Found bad field id; previous ID:{}, current ID:{}",
        previous,
        current
    )]
    BadFieldID { previous: usize, current: usize },
    #[fail(display = "Range restriction on non numeric type")]
    BadRangeRestriction,

    #[fail(
        display = "The string:'{}' was already contained in the StringPool",
        string
    )]
    DuplicatedString { string: String },
    #[fail(display = "The ID:{} is reserved", id)]
    ReservedID { id: usize },

    #[fail(display = "The pool has to be a base pool but was not")]
    BasePoolRequired,

    #[fail(display = "Seeking field because:{}", why)]
    BadSeek { why: String },
    #[fail(display = "Writing failed because:{}", why)]
    BadWrite { why: String },

    #[fail(display = "Flushing failed because:{}", why)]
    BadFlush { why: String },

    #[fail(display = "Expected an declaration chunk after compress!")]
    BadChunk,

    #[fail(display = "Failed to resize file because:{}", why)]
    FailedToResizeFile { why: String },
    #[fail(display = "Failed to create mmap because:{}", why)]
    FailedToCreateMMap { why: String },

    #[fail(
        display = "Wrong field type Expected:{} Found:{}",
        expected,
        found
    )]
    BadFieldType {
        expected: &'static str,
        found: String,
    },
    #[fail(
        display = "Wrong constant length of array Expected:{} Found:{}",
        expected,
        found
    )]
    BadConstantLength { expected: usize, found: usize },

    #[fail(display = "Bad cast")]
    BadCast,

    #[fail(
        display = "The type '{}' does not expect a super type. Found:'{}'",
        base,
        super_name
    )]
    UnexpectedSuperType {
        base: &'static str,
        super_name: String,
    },

    #[fail(
        display = "The type '{}' expects a super type of '{}' but none was given",
        base,
        expected
    )]
    MissingSuperType {
        base: &'static str,
        expected: &'static str,
    },
    #[fail(
        display = "Wrong super type for '{}' Expect:'{}' Found:'{}'",
        base,
        expected,
        found
    )]
    WrongSuperType {
        base: &'static str,
        expected: &'static str,
        found: String,
    },

    #[fail(
        display = "The given SkillObjectID:{} doesn't match a SkillObject",
        id
    )]
    BadSkillObjectID { id: usize },

    #[fail(display = "Creating a relative view on a buffer is unsupported")]
    ViewOnBuffer,

    #[fail(
        display = "Inconsistent foreign field indexes found. old:{}, new:{}",
        old,
        new
    )]
    InconsistentForeignIndex { old: usize, new: usize },

    #[fail(display = "Wrong type of foreign field")]
    WrongForeignField,

    #[fail(display = "After a compress there should only be one declaration chunk")]
    OnlyOneChunk,

    #[fail(
        display = "The field '{}' is supposed to be an auto field but the file contains data for it.",
        field
    )]
    AutoNotAuto { field: String },
    #[fail(display = "The field '{}' appeared twice.", field)]
    SameField { field: String },
    #[fail(
        display = "The constant field '{}' expected a value of:{} but found:{}",
        field,
        expected,
        found
    )]
    BadConstantValue {
        field: String,
        expected: String,
        found: String,
    },
    #[fail(
        display = "The constant field '{}' was not expected by '{}'.",
        field,
        type_name
    )]
    UnknownConstantField { field: String, type_name: String },
    #[fail(display = "Type and field names cant be null.")]
    TypeOrFieldNameNull,
    #[fail(display = "Pool is missing the type pools")]
    MissingTypePools,
    #[fail(display = "Pool is missing the block reader")]
    MissingBlockReader,
}

/// Indicates that the user is most probably at fault by using the binding wrong.
#[derive(Fail, Debug)]
pub enum UserFail {
    #[fail(
        display = "The file '{}' couldn't be created because:{}.",
        file,
        why
    )]
    FailedToCreateFile { file: String, why: String },
    #[fail(
        display = "The file '{}' couldn't be opened because:{}.",
        file,
        why
    )]
    FailedToOpenFile { file: String, why: String },
    #[fail(display = "The ID:{} is reserved", id)]
    ReservedID { id: usize },

    #[fail(display = "Access of deleted SkillObject")]
    AccessDeleted,

    #[fail(display = "Accessed object (ID:{}) of foreign type", id)]
    BadCastID { id: usize },

    #[fail(display = "ObjectProper (ID:{}) is unused / unknown", id)]
    UnknownObjectID { id: usize },

    #[fail(
        display = "ObjectProper (ID:{}) was used while marking it for deletion",
        id
    )]
    DeleteInUse { id: usize },
    #[fail(display = "The file was in read only mode but it was tried to write to it.")]
    ReadOnly,
    #[fail(display = "The field:{} is not known.", name)]
    UnknownField { name: String },
}

/// Collapses the verbosity of the different of [`InternalFail`] and [`UserFail`] errors into a single type.
#[derive(Fail, Debug)]
pub enum SkillFail {
    #[fail(display = "An internal error occurred: {}", cause)]
    Internal {
        cause: InternalFail,
        backtrace: failure::Backtrace,
    },
    #[fail(display = "An user caused error occurred: {}", cause)]
    User {
        cause: UserFail,
        backtrace: failure::Backtrace,
    },
}

impl SkillFail {
    /// # Arguments
    /// * `cause` - Cause of the fail
    ///
    /// # Returns
    /// Wrapped Fail
    pub fn internal(cause: InternalFail) -> SkillFail {
        SkillFail::Internal {
            cause,
            backtrace: Backtrace::new(),
        }
    }
    /// # Arguments
    /// * `cause` - Cause of the fail
    ///
    /// # Returns
    /// Wrapped Fail
    pub fn user(cause: UserFail) -> SkillFail {
        SkillFail::User {
            cause,
            backtrace: Backtrace::new(),
        }
    }
}
