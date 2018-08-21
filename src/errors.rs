//! Error handling.

use errno;
use std::{ffi, io};

error_chain!{
    errors {
        /// Syscall error, as `errno(3)`.
        Sys(errno: errno::Errno) {
            description("syscall failed")
            display("{}", errno)
        }
    }

    // doc attributes are required to workaround
    // https://github.com/rust-lang-nursery/error-chain/issues/63
    foreign_links {
        Io(io::Error) #[doc = "I/O error."];
        NulChar(ffi::NulError) #[doc = "NULL byte in conversion to C string."];
    }
}
