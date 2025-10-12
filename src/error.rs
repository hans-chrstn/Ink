use std::{fmt::{self, Display}, io};

#[derive(Debug)]
pub enum Error {
    Lua(mlua::Error),
    Io(io::Error),
    RootNotWindow,
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Lua(e) => write!(f, "Lua error: {}", e),
            Error::Io(e) => write!(f, "I/O error: {}", e),
            Error::RootNotWindow => {
                write!(f, "The root widget in the lua file must be a GtkApplicationWindow")
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Lua(e) => Some(e),
            Error::Io(e) => Some(e),
            Error::RootNotWindow => None,
        }
    }
}

impl From<mlua::Error> for Error {
    fn from(value: mlua::Error) -> Self {
        Error::Lua(value)
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Error::Io(value)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
