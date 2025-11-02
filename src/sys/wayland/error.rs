use std::io;

use crate::Error;

use wayland_client::{
    ConnectError, DispatchError,
    backend::{WaylandError, protocol::ProtocolError},
    globals::GlobalError,
};

impl From<WaylandError> for Error {
    fn from(value: WaylandError) -> Self {
        match value {
            WaylandError::Io(e) => Self::Io(e),
            WaylandError::Protocol(e) => Self::WaylandError(e),
        }
    }
}

impl From<ProtocolError> for Error {
    fn from(value: ProtocolError) -> Self {
        Self::from(WaylandError::Protocol(value))
    }
}

impl From<ConnectError> for Error {
    fn from(value: ConnectError) -> Self {
        Self::WaylandConnect(value)
    }
}

impl From<GlobalError> for Error {
    fn from(value: GlobalError) -> Self {
        match value {
            GlobalError::Backend(e) => Self::from(e),
            GlobalError::InvalidId(_) => Self::Io(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Invalid Wayland ID",
            )),
        }
    }
}

impl From<DispatchError> for Error {
    fn from(value: DispatchError) -> Self {
        match value {
            DispatchError::Backend(e) => Self::from(e),
            DispatchError::BadMessage { .. } => Self::Io(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Bad Message Sent",
            )),
        }
    }
}
