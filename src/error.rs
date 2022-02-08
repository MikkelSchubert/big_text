use std;
use std::error::Error;
use std::fmt::{Display, Formatter};
use walkdir;

#[derive(Debug)]
pub enum ProcErrorCause {
    Other,
    IoError(std::io::Error),
    WalkdirError(walkdir::Error),
}

#[derive(Debug)]
pub struct ProcError {
    message: String,
    cause: ProcErrorCause,
}

impl ProcError {
    pub fn new<T>(message: &str, cause: T) -> ProcError
    where
        T: Into<ProcErrorCause>,
    {
        ProcError {
            message: message.into(),
            cause: cause.into(),
        }
    }
}

impl Display for ProcError {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(fmt, "{}", self.description())?;
        match self.cause {
            ProcErrorCause::Other => {}
            ProcErrorCause::IoError(ref error) => {
                write!(fmt, " due to IO error: {}", error.description())?
            }
            ProcErrorCause::WalkdirError(ref error) => {
                if let Some(path) = error.path() {
                    write!(fmt, " at {:?}", path)?
                }

                write!(fmt, " due to walkdir error: {}", error.description())?
            }
        }

        Ok(())
    }
}

impl Error for ProcError {
    fn description(&self) -> &str {
        &self.message
    }

    fn cause(&self) -> Option<&Error> {
        match self.cause {
            ProcErrorCause::Other => None,
            ProcErrorCause::IoError(ref error) => Some(error),
            ProcErrorCause::WalkdirError(ref error) => Some(error),
        }
    }
}

impl Into<ProcErrorCause> for std::io::Error {
    fn into(self) -> ProcErrorCause {
        ProcErrorCause::IoError(self)
    }
}

impl Into<ProcErrorCause> for walkdir::Error {
    fn into(self) -> ProcErrorCause {
        ProcErrorCause::WalkdirError(self)
    }
}
