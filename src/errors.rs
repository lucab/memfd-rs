//! Error handling.
use std::fmt;

/// Enumeration of errors possible in this library
#[derive(Debug)]
pub enum Error {
    /// Cannot convert the `name` argument to a C String!
    NameCStringConversion(std::ffi::NulError),
    /// Cannot create the memfd
    Create(std::io::Error),
    /// Cannot add new seals to the memfd
    AddSeals(std::io::Error),
    /// Cannot read the seals of a memfd
    GetSeals(std::io::Error),
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        use Error::*;
        match self {
            NameCStringConversion(ref e) => Some(e),
            Create(ref e) => Some(e),
            AddSeals(ref e) => Some(e),
            GetSeals(ref e) => Some(e),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;
        f.write_str(match self {
            NameCStringConversion(_) => "cannot convert `name` to a C string",
            Create(_) => "cannot create a memfd",
            AddSeals(_) => "cannot add seals to the memfd",
            GetSeals(_) => "cannot read seals for a memfd",
        })
    }
}

#[cfg(test)]
#[test]
fn error_send_sync() {
    fn assert_error<E: std::error::Error + Send + Sync + fmt::Display + fmt::Debug + 'static>() {}
    assert_error::<Error>();
}
