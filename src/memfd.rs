use either;
use errno;
use errors;
use libc;
use nr;
use sealing;
use std::ffi;
use std::fs;
use std::os::unix::io::{AsRawFd, FromRawFd, RawFd};

/// A `Memfd` builder, providing advanced options and flags for specifying its behavior.
#[derive(Clone, Debug)]
pub struct MemfdOptions {
    allow_sealing: bool,
    cloexec: bool,
    hugetlb: Option<HugetlbSize>,
}

impl MemfdOptions {
    /// Default set of options for `Memfd` creation.
    ///
    /// The default options are:
    ///  * sealing: `F_SEAL_SEAL` (i.e. no further sealing)
    ///  * close-on-exec: false
    ///  * hugetlb: false
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether to allow sealing on the final memfd.
    pub fn allow_sealing(mut self, value: bool) -> Self {
        self.allow_sealing = value;
        self
    }

    /// Whether to set the `FD_CLOEXEC` flag on the final memfd.
    pub fn close_on_exec(mut self, value: bool) -> Self {
        self.cloexec = value;
        self
    }

    /// Optional hugetlb support and page size for the final memfd.
    pub fn hugetlb(mut self, size: Option<HugetlbSize>) -> Self {
        self.hugetlb = size;
        self
    }

    /// Translates the current options into a bitflags value for `memfd_create`.
    fn bitflags(&self) -> u32 {
        let mut bits = 0;
        if self.allow_sealing {
            bits |= nr::MFD_ALLOW_SEALING;
        }
        if self.cloexec {
            bits |= nr::MFD_CLOEXEC;
        }
        if let Some(ref hugetlb) = self.hugetlb {
            bits |= hugetlb.bitflags();
            bits |= nr::MFD_HUGETLB;
        }
        bits
    }

    /// Create a memfd according to configuration.
    pub fn create<T: AsRef<str>>(&self, name: T) -> errors::Result<Memfd> {
        let cname = ffi::CString::new(name.as_ref())?;
        let name_ptr = cname.as_ptr();
        let flags = self.bitflags();

        // UNSAFE(lucab): name_ptr points to memory owned by cname.
        let r = unsafe { libc::syscall(libc::SYS_memfd_create, name_ptr, flags) };
        if r < 0 {
            return Err(
                errors::Error::from_kind(errors::ErrorKind::Sys(errno::errno()))
                    .chain_err(|| "memfd_create error"),
            );
        };

        // UNSAFE(lucab): returned from kernel, checked for non-negative value.
        let mfd = unsafe { Memfd::from_raw_fd(r as RawFd) };
        Ok(mfd)
    }
}

impl Default for MemfdOptions {
    fn default() -> Self {
        Self {
            allow_sealing: false,
            cloexec: false,
            hugetlb: None,
        }
    }
}

/// Page size for a hugetlb anonymous file.
#[derive(Copy, Clone, Debug)]
pub enum HugetlbSize {
    /// 64KB hugetlb page.
    Huge64KB,
    /// 64KB hugetlb page.
    Huge512KB,
    /// 1MB hugetlb page.
    Huge1MB,
    /// 2MB hugetlb page.
    Huge2MB,
    /// 8MB hugetlb page.
    Huge8MB,
    /// 16MB hugetlb page.
    Huge16MB,
    /// 256MB hugetlb page.
    Huge256MB,
    /// 1GB hugetlb page.
    Huge1GB,
    /// 2GB hugetlb page.
    Huge2GB,
    /// 16GB hugetlb page.
    Huge16GB,
}

impl HugetlbSize {
    fn bitflags(self) -> u32 {
        match self {
            HugetlbSize::Huge64KB => nr::MFD_HUGE_64KB,
            HugetlbSize::Huge512KB => nr::MFD_HUGE_512KB,
            HugetlbSize::Huge1MB => nr::MFD_HUGE_1MB,
            HugetlbSize::Huge2MB => nr::MFD_HUGE_2MB,
            HugetlbSize::Huge8MB => nr::MFD_HUGE_8MB,
            HugetlbSize::Huge16MB => nr::MFD_HUGE_16MB,
            HugetlbSize::Huge256MB => nr::MFD_HUGE_256MB,
            HugetlbSize::Huge1GB => nr::MFD_HUGE_1GB,
            HugetlbSize::Huge2GB => nr::MFD_HUGE_2GB,
            HugetlbSize::Huge16GB => nr::MFD_HUGE_16GB,
        }
    }
}

/// An anonymous volatile file, with sealing capabilities.
#[derive(Debug)]
pub struct Memfd {
    file: fs::File,
}

impl Memfd {
    /// Try to convert a `File` object into a `Memfd`.
    ///
    /// This requires transferring ownership of the `File`.
    /// If the underlying file-descriptor is compatible with
    /// memfd/sealing, it returns a proper `Memfd` object,
    /// otherwise it transfers back ownership of the original
    /// `File` for further usage.
    pub fn try_from_file(fp: fs::File) -> either::Either<Self, fs::File> {
        // Check if the fd supports F_GET_SEALS;
        // if so, it is safely compatible with `Memfd`.
        match Self::file_get_seals(&fp) {
            Ok(_) => either::Either::Left(Self { file: fp }),
            Err(_) => either::Either::Right(fp),
        }
    }

    /// Return a `File` object for this memfd.
    pub fn as_file(&self) -> &fs::File {
        &self.file
    }

    /// Consume this `Memfd`, returning the underlying `File`.
    pub fn into_file(self) -> fs::File {
        self.file
    }

    /// Return the current set of seals.
    pub fn seals(&self) -> errors::Result<sealing::SealsHashSet> {
        let flags = Self::file_get_seals(&self.file)?;
        Ok(sealing::bitflags_to_seals(flags))
    }

    /// Add a single seal to the existing set of seals.
    pub fn add_seal(&self, seal: sealing::FileSeal) -> errors::Result<()> {
        use std::iter::FromIterator;

        let set = sealing::SealsHashSet::from_iter(vec![seal]);
        self.add_seals(&set)
    }

    /// Add some seals to the existing set of seals.
    pub fn add_seals(&self, seals: &sealing::SealsHashSet) -> errors::Result<()> {
        let fd = self.file.as_raw_fd();
        let flags = sealing::seals_to_bitflags(seals);
        // UNSAFE(lucab): required syscall.
        let r = unsafe { libc::syscall(libc::SYS_fcntl, fd, libc::F_ADD_SEALS, flags) };
        if r < 0 {
            return Err(
                errors::Error::from_kind(errors::ErrorKind::Sys(errno::errno()))
                    .chain_err(|| "F_ADD_SEALS error"),
            );
        };
        Ok(())
    }

    /// Return the current sealing bitflags.
    fn file_get_seals(fp: &fs::File) -> errors::Result<u64> {
        let fd = fp.as_raw_fd();
        // UNSAFE(lucab): required syscall.
        let r = unsafe { libc::syscall(libc::SYS_fcntl, fd, libc::F_GET_SEALS) };
        if r < 0 {
            return Err(
                errors::Error::from_kind(errors::ErrorKind::Sys(errno::errno()))
                    .chain_err(|| "F_GET_SEALS error"),
            );
        };

        Ok(r as u64)
    }

    /// Assemble a `File` object from a raw file-descriptor.
    ///
    /// Safety: `fd` must be a valid file-descriptor for the calling
    /// process at the time of invocation.
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        let file = fs::File::from_raw_fd(fd);
        Self { file }
    }
}
