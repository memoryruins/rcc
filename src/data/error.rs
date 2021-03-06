use std::collections::VecDeque;
use thiserror::Error;

use super::{lex::Token, Expr, Locatable, Location, Type};

/// RecoverableResult is a type that represents a Result that can be recovered from.
///
/// See the [`Recover`] trait for more information.
///
/// [`Recover`]: trait.Recover.html
pub type RecoverableResult<T, E = CompileError> = Result<T, (E, T)>;
pub type CompileResult<T> = Result<T, CompileError>;
pub type CompileError = Locatable<Error>;
pub type CompileWarning = Locatable<Warning>;

/// ErrorHandler is a struct that hold errors generated by the compiler
///
/// An error handler is used because multiple errors may be generated by each
/// part of the compiler, this cannot be represented well with Rust's normal
/// `Result`.
#[derive(Debug, Default, PartialEq)]
pub(crate) struct ErrorHandler {
    errors: VecDeque<CompileError>,
    pub(crate) warnings: VecDeque<CompileWarning>,
}

impl ErrorHandler {
    /// Construct a new error handler.
    pub(crate) fn new() -> ErrorHandler {
        Default::default()
    }

    /// Add an error to the error handler.
    pub(crate) fn push_back<E: Into<CompileError>>(&mut self, error: E) {
        self.errors.push_back(error.into());
    }

    /// Remove the first error from the queue
    pub(crate) fn pop_front(&mut self) -> Option<CompileError> {
        self.errors.pop_front()
    }

    /// Stopgap to make it easier to transition to lazy warnings.
    ///
    /// TODO: Remove this method
    pub(crate) fn warn<W: Into<Warning>>(&mut self, warning: W, location: Location) {
        self.warnings.push_back(location.with(warning.into()));
    }
    /// Add an iterator of errors to the error queue
    pub(crate) fn extend<E: Into<CompileError>>(&mut self, iter: impl Iterator<Item = E>) {
        self.errors.extend(iter.map(Into::into));
    }
}

impl Iterator for ErrorHandler {
    type Item = CompileError;

    fn next(&mut self) -> Option<CompileError> {
        self.pop_front()
    }
}

#[derive(Clone, Debug, Error, PartialEq)]
pub enum Error {
    #[error("invalid program: {0}")]
    Semantic(#[from] SemanticError),

    #[error("invalid syntax: {0}")]
    Syntax(#[from] SyntaxError),

    #[error("invalid macro: {0}")]
    PreProcessor(#[from] CppError),

    #[error("invalid token: {0}")]
    Lex(#[from] LexError),
}

/// Semantic errors are non-exhaustive and may have new variants added at any time
#[derive(Clone, Debug, Error, PartialEq)]
pub enum SemanticError {
    #[error("{0}")]
    Generic(String),

    #[error("cannot have empty program")]
    EmptyProgram,

    #[error("use of undeclared identifier '{0}'")]
    UndeclaredVar(crate::intern::InternedStr),

    #[error("{} overflow in expresson", if *(.is_positive) { "positive" } else { "negative" })]
    ConstOverflow { is_positive: bool },

    #[error("not a constant expression: {0}")]
    NotConstant(Expr),

    // String is the reason it couldn't be assigned
    #[error("cannot assign to {0}")]
    NotAssignable(String),

    #[error("cannot take address of {0}")]
    InvalidAddressOf(&'static str),

    #[error("cannot divide by zero")]
    DivideByZero,

    #[error("cannot shift {} by a negative amount", if *(.is_left) { "left" } else { "right" })]
    NegativeShift { is_left: bool },

    #[error("cannot shift {} by {maximum} or more bits for type '{ctype}' (got {current})", if *(.is_left) { "left" } else { "right" })]
    TooManyShiftBits {
        is_left: bool,
        maximum: u64,
        ctype: Type,
        current: u64,
    },

    #[error("unreachable statement")]
    UnreachableStatement,

    #[error("redeclaration of label {0}")]
    LabelRedeclaration(cranelift::prelude::Ebb),

    #[error("use of undeclared label {0}")]
    UndeclaredLabel(crate::intern::InternedStr),

    #[error("{}case outside of switch statement", if *(.is_default) { "default " } else { "" })]
    CaseOutsideSwitch { is_default: bool },

    #[error("cannot have multiple default cases in a switch statement")]
    MultipleDefaultCase,

    #[error("void must be the first and only parameter if specified")]
    InvalidVoidParameter,

    #[error("expected expression, got typedef")]
    TypedefInExpressionContext,

    #[doc(hidden)]
    #[error("internal error: do not construct nonexhaustive variants")]
    __Nonexhaustive,
}

/// Syntax errors are non-exhaustive and may have new variants added at any time
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum SyntaxError {
    #[error("{0}")]
    Generic(String),

    #[error("expected {0}, got <end-of-file>")]
    EndOfFile(&'static str),

    #[error("expected statement, got {0}")]
    NotAStatement(super::Keyword),

    #[doc(hidden)]
    #[error("internal error: do not construct nonexhaustive variants")]
    __Nonexhaustive,
}

/// Preprocessing errors are non-exhaustive and may have new variants added at any time
#[derive(Clone, Debug, Error, PartialEq)]
pub enum CppError {
    #[error("{0}")]
    Generic(String),

    #[error("invalid preprocessing directive")]
    InvalidDirective,

    // valid token in the wrong position
    #[error("expected {0}, got {1}")]
    UnexpectedToken(&'static str, Token),

    #[error("expected {0}, got <end-of-file>")]
    EndOfFile(&'static str),

    // invalid token
    #[error("invalid preprocessor token {0}")]
    InvalidCppToken(Token),

    #[error("{0} is never terminated")]
    UnterminatedDirective(&'static str),

    #[error("expected expression for #if")]
    EmptyExpression,

    #[error("#endif without #if")]
    UnexpectedEndIf,

    #[doc(hidden)]
    #[error("internal error: do not construct nonexhaustive variants")]
    __Nonexhaustive,
}

/// Lex errors are non-exhaustive and may have new variants added at any time
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum LexError {
    #[error("{0}")]
    Generic(String),

    #[error("unterminated /* comment")]
    UnterminatedComment,

    #[doc(hidden)]
    #[error("internal error: do not construct nonexhaustive variants")]
    __Nonexhaustive,
}

#[derive(Debug, Error, PartialEq, Eq)]
/// errors are non-exhaustive and may have new variants added at any time
pub enum Warning {
    // for compatibility
    #[error("{0}")]
    Generic(String),

    #[doc(hidden)]
    #[error("internal error: do not construct nonexhaustive variants")]
    __Nonexhaustive,
}

impl<T: Into<String>> From<T> for Warning {
    fn from(msg: T) -> Warning {
        Warning::Generic(msg.into())
    }
}

impl CompileError {
    pub(crate) fn semantic(err: Locatable<String>) -> Self {
        Self::from(err)
    }
    pub fn location(&self) -> Location {
        self.location
    }
    pub fn is_lex_err(&self) -> bool {
        self.data.is_lex_err()
    }
    pub fn is_syntax_err(&self) -> bool {
        self.data.is_syntax_err()
    }
    pub fn is_semantic_err(&self) -> bool {
        self.data.is_semantic_err()
    }
}

impl Error {
    pub fn is_lex_err(&self) -> bool {
        if let Error::Lex(_) = self {
            true
        } else {
            false
        }
    }
    pub fn is_syntax_err(&self) -> bool {
        if let Error::Syntax(_) = self {
            true
        } else {
            false
        }
    }
    pub fn is_semantic_err(&self) -> bool {
        if let Error::Semantic(_) = self {
            true
        } else {
            false
        }
    }
}

impl From<Locatable<String>> for CompileError {
    fn from(err: Locatable<String>) -> Self {
        err.map(|s| SemanticError::Generic(s).into())
    }
}

impl From<Locatable<SemanticError>> for CompileError {
    fn from(err: Locatable<SemanticError>) -> Self {
        err.map(Error::Semantic)
    }
}

impl From<Locatable<SyntaxError>> for CompileError {
    fn from(err: Locatable<SyntaxError>) -> Self {
        err.map(Error::Syntax)
    }
}

impl From<Locatable<CppError>> for CompileError {
    fn from(err: Locatable<CppError>) -> Self {
        err.map(Error::PreProcessor)
    }
}

impl From<Locatable<String>> for Locatable<SemanticError> {
    fn from(err: Locatable<String>) -> Self {
        err.map(SemanticError::Generic)
    }
}

impl<S: Into<String>> From<S> for SemanticError {
    fn from(err: S) -> Self {
        SemanticError::Generic(err.into())
    }
}

impl<S: Into<String>> From<S> for SyntaxError {
    fn from(err: S) -> Self {
        SyntaxError::Generic(err.into())
    }
}

pub(crate) trait Recover {
    type Ok;
    fn recover(self, error_handler: &mut ErrorHandler) -> Self::Ok;
}

impl<T, E: Into<CompileError>> Recover for RecoverableResult<T, E> {
    type Ok = T;
    fn recover(self, error_handler: &mut ErrorHandler) -> T {
        self.unwrap_or_else(|(e, i)| {
            error_handler.push_back(e);
            i
        })
    }
}

impl<T, E: Into<CompileError>> Recover for RecoverableResult<T, Vec<E>> {
    type Ok = T;
    fn recover(self, error_handler: &mut ErrorHandler) -> T {
        self.unwrap_or_else(|(es, i)| {
            error_handler.extend(es.into_iter());
            i
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_error() -> CompileError {
        Location::default().with(Error::Lex(LexError::UnterminatedComment))
    }

    fn new_error(error: Error) -> CompileError {
        Location::default().with(error)
    }

    #[test]
    fn test_error_handler_push_err() {
        let mut error_handler = ErrorHandler::new();
        error_handler.push_back(dummy_error());

        assert_eq!(
            error_handler,
            ErrorHandler {
                errors: vec_deque![dummy_error()],
                warnings: VecDeque::new(),
            }
        );
    }

    #[test]
    fn test_error_handler_into_iterator() {
        let mut error_handler = ErrorHandler::new();
        error_handler.push_back(dummy_error());
        let errors = error_handler.collect::<Vec<_>>();
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_compile_error_semantic() {
        assert_eq!(
            CompileError::semantic(Location::default().with("".to_string())).data,
            Error::Semantic(SemanticError::Generic("".to_string())),
        );
    }

    #[test]
    fn test_compile_error_is_kind() {
        let e = Error::Lex(LexError::Generic("".to_string()));
        assert!(e.is_lex_err());
        assert!(!e.is_semantic_err());
        assert!(!e.is_syntax_err());

        let e = Error::Semantic(SemanticError::Generic("".to_string()));
        assert!(!e.is_lex_err());
        assert!(e.is_semantic_err());
        assert!(!e.is_syntax_err());

        let e = Error::Syntax(SyntaxError::Generic("".to_string()));
        assert!(!e.is_lex_err());
        assert!(!e.is_semantic_err());
        assert!(e.is_syntax_err());
    }

    #[test]
    fn test_compile_error_display() {
        assert_eq!(
            dummy_error().data.to_string(),
            "invalid token: unterminated /* comment"
        );

        assert_eq!(
            Error::Semantic(SemanticError::Generic("bad code".to_string())).to_string(),
            "invalid program: bad code"
        );
    }

    #[test]
    fn test_compile_error_from_locatable_string() {
        let _ = CompileError::from(Location::default().with("apples".to_string()));
    }

    #[test]
    fn test_compile_error_from_syntax_error() {
        let _ = Location::default().error(SyntaxError::from("oranges".to_string()));
    }

    #[test]
    fn test_recover_error() {
        let mut error_handler = ErrorHandler::new();
        let r: RecoverableResult<i32> = Ok(1);
        assert_eq!(r.recover(&mut error_handler), 1);
        assert_eq!(error_handler.pop_front(), None);

        let mut error_handler = ErrorHandler::new();
        let r: RecoverableResult<i32> = Err((dummy_error(), 42));
        assert_eq!(r.recover(&mut error_handler), 42);
        let errors = error_handler.collect::<Vec<_>>();
        assert_eq!(errors, vec![dummy_error()]);
    }

    #[test]
    fn test_recover_multiple_errors() {
        let mut error_handler = ErrorHandler::new();
        let r: RecoverableResult<i32, Vec<CompileError>> = Ok(1);
        assert_eq!(r.recover(&mut error_handler), 1);
        assert_eq!(error_handler.pop_front(), None);

        let mut error_handler = ErrorHandler::new();
        let r: RecoverableResult<i32, Vec<CompileError>> = Err((
            vec![
                dummy_error(),
                new_error(Error::Semantic(SemanticError::Generic("pears".to_string()))),
            ],
            42,
        ));
        assert_eq!(r.recover(&mut error_handler), 42);
        let errors = error_handler.collect::<Vec<_>>();
        assert_eq!(
            errors,
            vec![
                dummy_error(),
                new_error(Error::Semantic(SemanticError::Generic("pears".to_string()))),
            ]
        );
    }
}
