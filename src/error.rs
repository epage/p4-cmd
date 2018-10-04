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

type ErrorCause = Error + Send + Sync + 'static;

/// For programmatically processing failures.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ErrorKind {
    SpawnFailed,
    ParseFailed,
}

impl ErrorKind {
    pub(crate) fn error(self) -> P4Error {
        P4Error::new(self)
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ErrorKind::SpawnFailed => write!(f, "Failed to launch P4 command."),
            ErrorKind::ParseFailed => write!(f, "Failed to parse P4 output."),
        }
    }
}

#[derive(Debug)]
pub struct P4Error {
    kind: ErrorKind,
    context: Option<String>,
    cause: Option<Box<ErrorCause>>,
}

impl P4Error {
    pub(crate) fn new(kind: ErrorKind) -> Self {
        Self {
            kind,
            context: None,
            cause: None,
        }
    }

    pub(crate) fn set_context<S>(mut self, context: S) -> Self
    where
        S: Into<String>,
    {
        let context = context.into();
        self.context = Some(context);
        self
    }

    pub(crate) fn set_cause<E>(mut self, cause: E) -> Self
    where
        E: Error + Send + Sync + 'static,
    {
        let cause = Box::new(cause);
        self.cause = Some(cause);
        self
    }

    /// Programmtically process failure.
    pub fn kind(&self) -> ErrorKind {
        self.kind
    }
}

impl Error for P4Error {
    fn description(&self) -> &str {
        "Staging failed."
    }

    fn cause(&self) -> Option<&Error> {
        self.cause.as_ref().map(|c| {
            let c: &Error = c.as_ref();
            c
        })
    }
}

impl fmt::Display for P4Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Operation failed: {}", self.kind)?;
        if let Some(ref context) = self.context {
            writeln!(f, "{}", context)?;
        }
        if let Some(ref cause) = self.cause {
            writeln!(f, "Cause: {}", cause)?;
        }
        Ok(())
    }
}
