//! Detect kernel features at runtime.
//!
//! This module exposes methods to detect kernel features at runtime.
//! This allows applications to auto-detect whether recent options are supported by the currently running kernel.

use std::sync::OnceLock;

use rustix::fs::MemfdFlags;

use crate::MemfdOptions;

static NOEXEC_SEAL_SUPPORTED: OnceLock<bool> = OnceLock::new();

/// Returns whether the `MFD_NOEXEC_SEAL`/`MFD_EXEC` flags are supported by the current kernel.
/// 
/// This has been introduced in Linux kernel 6.3, and allows controlling the `exec` permission
/// bits when creating a memfd.
/// 
/// See <https://docs.kernel.org/userspace-api/mfd_noexec.html> for more details.
pub fn create_noexec_supported() -> bool {
    *NOEXEC_SEAL_SUPPORTED.get_or_init(|| {
        let flags = MemfdFlags::CLOEXEC | MemfdFlags::NOEXEC_SEAL;
        MemfdOptions::create_inner("memfd-rs-noexec-seal-probe", flags).is_ok()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_noexec_seal_supported() {
        let flags = MemfdFlags::CLOEXEC | MemfdFlags::NOEXEC_SEAL;
        let res = MemfdOptions::create_inner("probe-test", flags);
        let is_supported = create_noexec_supported();
        assert_eq!(res.is_ok(), is_supported);

        if is_supported {
            let mfd = res.unwrap();
            assert!(crate::memfd::check_memfd_seals(&mfd));
        }
    }
}
