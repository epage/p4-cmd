use std::error::Error;
use std::fmt;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct OperationError {
    code: i32,
}

impl OperationError {
    pub(crate) fn new(code: i32) -> Self {
        Self { code }
    }

    // Keeping around for future use.
    #[allow(dead_code)]
    pub(crate) fn code(&self) -> i32 {
        self.code
    }
}

impl Error for OperationError {
    fn description(&self) -> &str {
        "Operation failed."
    }

    fn cause(&self) -> Option<&Error> {
        None
    }
}

impl fmt::Display for OperationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Operation failed")
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MessageLevel {
    Error,
    Warning,

    #[doc(hidden)]
    __Nonexhaustive,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    level: MessageLevel,
    msg: String,
}

impl Message {
    pub(crate) fn new(level: MessageLevel, msg: String) -> Self {
        Self { level, msg }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item<T> {
    Data(T),
    Message(Message),
    Error(OperationError),

    #[doc(hidden)]
    __Nonexhaustive,
}

impl<T> Item<T> {
    pub fn as_data(&self) -> Option<&T> {
        match self {
            Item::Data(t) => Some(t),
            _ => None,
        }
    }

    pub fn as_message(&self) -> Option<&Message> {
        match self {
            Item::Message(m) => Some(m),
            _ => None,
        }
    }

    pub fn as_error(&self) -> Option<&OperationError> {
        match self {
            Item::Error(e) => Some(e),
            _ => None,
        }
    }
}
