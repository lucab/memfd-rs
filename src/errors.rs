//! Error handling.

/// Enumeration of errors possible in this library
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Cannot convert the `name` argument to a C String!
    #[error("cannot convert `name` to a C string")]
    NameCStringConversion(#[source] std::ffi::NulError),
    /// Cannot create the memfd
    #[error("cannot create memfd")]
    Create(#[source] std::io::Error),
    /// Cannot add new seals to the memfd
    #[error("cannot add seals to the memfd")]
    AddSeals(#[source] std::io::Error),
    /// Cannot read the seals of a memfd
    #[error("cannot read seals for a memfd")]
    GetSeals(#[source] std::io::Error),
}

#[cfg(test)]
#[test]
fn error_send_sync() {
    fn assert_error<E: std::error::Error + Send + Sync + 'static>() {}
    assert_error::<Error>();
}
