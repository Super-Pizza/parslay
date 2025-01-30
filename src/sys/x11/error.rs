use std::io;

use crate::Error;
use x11rb::{
    errors::{
        ConnectError, ConnectionError, DisplayParsingError, ParseError, ReplyError, ReplyOrIdError,
    },
    x11_utils::X11Error,
};

impl From<X11Error> for Error {
    fn from(value: X11Error) -> Self {
        Self::X11(ReplyError::X11Error(value))
    }
}

impl From<ConnectionError> for Error {
    fn from(value: ConnectionError) -> Self {
        Self::X11(ReplyError::ConnectionError(value))
    }
}

impl From<ReplyError> for Error {
    fn from(value: ReplyError) -> Self {
        Self::X11(value)
    }
}

impl From<ReplyOrIdError> for Error {
    fn from(value: ReplyOrIdError) -> Self {
        match value {
            ReplyOrIdError::ConnectionError(c) => Self::X11(ReplyError::ConnectionError(c)),
            ReplyOrIdError::X11Error(x) => Self::X11(ReplyError::X11Error(x)),
            ReplyOrIdError::IdsExhausted => {
                Self::Io(io::Error::new(io::ErrorKind::Other, "X11 IDs exhausted"))
            }
        }
    }
}

impl From<ConnectError> for Error {
    fn from(value: ConnectError) -> Self {
        let error = match value {
            ConnectError::ParseError(parse_error) => ConnectionError::ParseError(parse_error),
            ConnectError::InsufficientMemory => ConnectionError::InsufficientMemory,
            ConnectError::DisplayParsingError(DisplayParsingError::MalformedValue(_)) => {
                ConnectionError::ParseError(ParseError::InvalidValue)
            }
            ConnectError::DisplayParsingError(DisplayParsingError::DisplayNotSet) => {
                return Self::Io(io::Error::new(io::ErrorKind::NotFound, "DISPLAY not set"))
            }
            ConnectError::DisplayParsingError(DisplayParsingError::NotUnicode) => {
                ConnectionError::ParseError(ParseError::InvalidValue)
            }
            ConnectError::DisplayParsingError(DisplayParsingError::Unknown) => {
                ConnectionError::UnknownError
            }
            ConnectError::InvalidScreen => {
                return Self::Io(io::Error::new(io::ErrorKind::InvalidData, "Invalid Screen"))
            }
            ConnectError::IoError(error) => return Self::Io(error),
            ConnectError::ZeroIdMask => {
                return Self::Io(io::Error::new(io::ErrorKind::InvalidInput, "Zero ID Mask"))
            }
            ConnectError::SetupAuthenticate(setup_authenticate) => {
                return Self::Io(io::Error::new(
                    io::ErrorKind::ConnectionAborted,
                    String::from_utf8_lossy(&setup_authenticate.reason),
                ))
            }
            ConnectError::SetupFailed(setup_failed) => {
                return Self::Io(io::Error::new(
                    io::ErrorKind::ConnectionAborted,
                    String::from_utf8_lossy(&setup_failed.reason),
                ))
            }
            ConnectError::Incomplete { .. } => {
                ConnectionError::ParseError(ParseError::InsufficientData)
            }
            _ => ConnectionError::UnknownError,
        };
        Self::X11(ReplyError::ConnectionError(error))
    }
}
