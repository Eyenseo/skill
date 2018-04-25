use std::error::Error as StdError;
use std::fmt;

// TODO https://docs.serde.rs/src/serde_json/error.rs.html

// TODO improve errors - include information - use Box?

#[doc(hidden)]
pub struct Void(());

// TODO look up what this does
use self::SkillError::{
    BadSkillObjectID, NotAFile, RedefinitionOfType, StringContainsInvalidUTF8, StringTooShort,
    UnexpectedEndOfInput, UnimplementedType, UnknownType,
};

// TODO prune unused errors
#[derive(Debug)]
pub enum SkillError {
    UnexpectedEndOfInput,
    StringContainsInvalidUTF8,
    StringTooShort,
    NotAFile,
    RedefinitionOfType,
    UnimplementedType,
    UnknownType,
    BadSkillObjectID,
    // Io(IoError),
    #[doc(hidden)] // TODO check what this does
    _Nonexhaustive(Void),
}

// adapted from https://hyper.rs/hyper/v0.10.5/src/hyper/error.rs.html#32
impl fmt::Debug for Void {
    fn fmt(&self, _: &mut fmt::Formatter) -> fmt::Result {
        unreachable!()
    }
}
impl fmt::Display for SkillError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            //Io(ref e) => fmt::Display::fmt(e, f),
            ref e => f.write_str(e.description()),
        }
    }
}
impl StdError for SkillError {
    fn description(&self) -> &str {
        match *self {
            UnexpectedEndOfInput => "The input stream ended even though more input was expected",
            StringTooShort => {
                "The read string was too short, probably because the input ended to early."
            }
            StringContainsInvalidUTF8 => "The to be read string contained invalid UTF-8 coding.",
            NotAFile => "The provided path doesn't point to a file.",
            RedefinitionOfType => "A type was declared more than once.",
            UnimplementedType => "The type is not implemented.",
            UnknownType => "The specified type is unknown",
            BadSkillObjectID => "The given SkillObjectID doesn't match a SkillObject",

            //Io(ref e) => e.description(),
            SkillError::_Nonexhaustive(..) => unreachable!(),
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            //Io(ref error) => Some(error),
            _ => None,
        }
    }
}
