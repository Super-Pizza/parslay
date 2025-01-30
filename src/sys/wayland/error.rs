use std::io;

use crate::Error;

use wayland_client::{
    backend::{protocol::ProtocolError, WaylandError},
    globals::GlobalError,
    ConnectError, DispatchError,
};

impl From<ProtocolError> for Error {
    fn from(value: ProtocolError) -> Self {
        Self::WaylandError(WaylandError::Protocol(value))
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
            GlobalError::Backend(e) => Self::WaylandError(e),
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
            DispatchError::Backend(e) => Self::WaylandError(e),
            DispatchError::BadMessage { .. } => Self::Io(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Bad Message Sent",
            )),
        }
    }
}
