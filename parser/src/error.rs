//! Define internal parse error types
//! The goal is to provide a matching and a safe error API, maksing errors from LALR
use lalrpop_util::ParseError as LalrpopError;

use crate::ast::Location;
use crate::token::Tok;

use alloc::{boxed::Box, string::String};
use core::fmt;
#[cfg(feature = "std")]
use std::error::Error;

/// Represents an error during lexical scanning.
#[derive(Debug, PartialEq)]
pub struct LexicalError {
    pub error: LexicalErrorType,
    pub location: Location,
}

#[derive(Debug, PartialEq)]
pub enum LexicalErrorType {
    StringError,
    UnicodeError,
    NestingError,
    IndentationError,
    TabError,
    DefaultArgumentError,
    PositionalArgumentError,
    DuplicateKeywordArgumentError,
    UnrecognizedToken { tok: char },
    FStringError(FStringErrorType),
    LineContinuationError,
    EOF,
    OtherError(String),
}

impl fmt::Display for LexicalErrorType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LexicalErrorType::StringError => write!(f, "Got unexpected string"),
            LexicalErrorType::FStringError(error) => write!(f, "Got error in f-string: {}", error),
            LexicalErrorType::UnicodeError => write!(f, "Got unexpected unicode"),
            LexicalErrorType::NestingError => write!(f, "Got unexpected nesting"),
            LexicalErrorType::IndentationError => {
                write!(f, "unindent does not match any outer indentation level")
            }
            LexicalErrorType::TabError => {
                write!(f, "inconsistent use of tabs and spaces in indentation")
            }
            LexicalErrorType::DefaultArgumentError => {
                write!(f, "non-default argument follows default argument")
            }
            LexicalErrorType::DuplicateKeywordArgumentError => {
                write!(f, "keyword argument repeated")
            }
            LexicalErrorType::PositionalArgumentError => {
                write!(f, "positional argument follows keyword argument")
            }
            LexicalErrorType::UnrecognizedToken { tok } => {
                write!(f, "Got unexpected token {}", tok)
            }
            LexicalErrorType::LineContinuationError => {
                write!(f, "unexpected character after line continuation character")
            }
            LexicalErrorType::EOF => write!(f, "unexpected EOF while parsing"),
            LexicalErrorType::OtherError(msg) => write!(f, "{}", msg),
        }
    }
}

#[cfg(feature = "std")]
impl Error for LexicalErrorType {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            LexicalErrorType::FStringError(e) => Some(e),
            _ => None,
        }
    }
}

// TODO: consolidate these with ParseError
#[derive(Debug, PartialEq)]
pub struct FStringError {
    pub error: FStringErrorType,
    pub location: Location,
}

#[derive(Debug, PartialEq)]
pub enum FStringErrorType {
    UnclosedLbrace,
    UnopenedRbrace,
    ExpectedRbrace,
    InvalidExpression(Box<ParseErrorType>),
    InvalidConversionFlag,
    EmptyExpression,
    MismatchedDelimiter,
    ExpressionNestedTooDeeply,
}

impl fmt::Display for FStringErrorType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FStringErrorType::UnclosedLbrace => write!(f, "Unclosed '{{'"),
            FStringErrorType::UnopenedRbrace => write!(f, "Unopened '}}'"),
            FStringErrorType::ExpectedRbrace => write!(f, "Expected '}}' after conversion flag."),
            FStringErrorType::InvalidExpression(error) => {
                write!(f, "Invalid expression: {}", error)
            }
            FStringErrorType::InvalidConversionFlag => write!(f, "Invalid conversion flag"),
            FStringErrorType::EmptyExpression => write!(f, "Empty expression"),
            FStringErrorType::MismatchedDelimiter => write!(f, "Mismatched delimiter"),
            FStringErrorType::ExpressionNestedTooDeeply => {
                write!(f, "expressions nested too deeply")
            }
        }
    }
}

impl From<FStringError> for LalrpopError<Location, Tok, LexicalError> {
    fn from(err: FStringError) -> Self {
        lalrpop_util::ParseError::User {
            error: LexicalError {
                error: LexicalErrorType::FStringError(err.error),
                location: err.location,
            },
        }
    }
}

#[cfg(feature = "std")]
impl Error for FStringErrorType {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            FStringErrorType::InvalidExpression(e) => Some(e.as_ref()),
            _ => None,
        }
    }
}

/// Represents an error during parsing
#[derive(Debug, PartialEq)]
pub struct ParseError {
    pub error: ParseErrorType,
    pub location: Location,
}

#[derive(Debug, PartialEq)]
pub enum ParseErrorType {
    /// Parser encountered an unexpected end of input
    EOF,
    /// Parser encountered an extra token
    ExtraToken(Tok),
    /// Parser encountered an invalid token
    InvalidToken,
    /// Parser encountered an unexpected token
    UnrecognizedToken(Tok, Option<String>),
    /// Maps to `User` type from `lalrpop-util`
    Lexical(LexicalErrorType),
}

/// Convert `lalrpop_util::ParseError` to our internal type
impl From<LalrpopError<Location, Tok, LexicalError>> for ParseError {
    fn from(err: LalrpopError<Location, Tok, LexicalError>) -> Self {
        match err {
            // TODO: Are there cases where this isn't an EOF?
            LalrpopError::InvalidToken { location } => ParseError {
                error: ParseErrorType::EOF,
                location,
            },
            LalrpopError::ExtraToken { token } => ParseError {
                error: ParseErrorType::ExtraToken(token.1),
                location: token.0,
            },
            LalrpopError::User { error } => ParseError {
                error: ParseErrorType::Lexical(error.error),
                location: error.location,
            },
            LalrpopError::UnrecognizedToken { token, expected } => {
                // Hacky, but it's how CPython does it. See PyParser_AddToken,
                // in particular "Only one possible expected token" comment.
                let expected = if expected.len() == 1 {
                    Some(expected[0].clone())
                } else {
                    None
                };
                ParseError {
                    error: ParseErrorType::UnrecognizedToken(token.1, expected),
                    location: token.0,
                }
            }
            LalrpopError::UnrecognizedEOF { location, .. } => ParseError {
                error: ParseErrorType::EOF,
                location,
            },
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} at {}", self.error, self.location)
    }
}

impl fmt::Display for ParseErrorType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParseErrorType::EOF => write!(f, "Got unexpected EOF"),
            ParseErrorType::ExtraToken(ref tok) => write!(f, "Got extraneous token: {:?}", tok),
            ParseErrorType::InvalidToken => write!(f, "Got invalid token"),
            ParseErrorType::UnrecognizedToken(ref tok, ref expected) => {
                if *tok == Tok::Indent {
                    write!(f, "unexpected indent")
                } else if expected.as_deref() == Some("Indent") {
                    write!(f, "expected an indented block")
                } else {
                    write!(f, "Got unexpected token {}", tok)
                }
            }
            ParseErrorType::Lexical(ref error) => write!(f, "{}", error),
        }
    }
}

#[cfg(feature = "std")]
impl Error for ParseErrorType {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ParseErrorType::Lexical(e) => Some(e),
            _ => None,
        }
    }
}

impl ParseErrorType {
    pub fn is_indentation_error(&self) -> bool {
        match self {
            ParseErrorType::Lexical(LexicalErrorType::IndentationError) => true,
            ParseErrorType::UnrecognizedToken(token, expected) => {
                *token == Tok::Indent || expected.as_ref().map_or(false, |s| s == "Indent")
            }
            _ => false,
        }
    }
    pub fn is_tab_error(&self) -> bool {
        matches!(self, ParseErrorType::Lexical(LexicalErrorType::TabError))
    }
}

impl core::ops::Deref for ParseError {
    type Target = ParseErrorType;
    fn deref(&self) -> &Self::Target {
        &self.error
    }
}

#[cfg(feature = "std")]
impl Error for ParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.error.source()
    }
}
