use std::os::unix::io::{AsRawFd, FromRawFd, IntoRawFd, RawFd};
use std::{ffi, fs, os::raw};
use crate::{nr, sealing};

#[cfg(any(target_os = "android", target_os="linux"))]
unsafe fn memfd_create(name: *const raw::c_char, flags: raw::c_uint) -> raw::c_int {
    libc::syscall(libc::SYS_memfd_create, name, flags) as raw::c_int
}

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
    ///  * [`FileSeal::SealSeal`] (i.e. no further sealing);
    ///  * close-on-exec is disabled;
    ///  * hugetlb is disabled.
    ///
    /// [`FileSeal::SealSeal`]: sealing::FileSeal::SealSeal
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether to allow adding seals to the created `Memfd`.
    pub fn allow_sealing(mut self, value: bool) -> Self {
        self.allow_sealing = value;
        self
    }

    /// Whether to set the `FD_CLOEXEC` flag on the created `Memfd`.
    pub fn close_on_exec(mut self, value: bool) -> Self {
        self.cloexec = value;
        self
    }

    /// Optional hugetlb support and page size for the created `Memfd`.
    pub fn hugetlb(mut self, size: Option<HugetlbSize>) -> Self {
        self.hugetlb = size;
        self
    }

    /// Translate the current options into a bitflags value for `memfd_create`.
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

    /// Create a [`Memfd`] according to configuration.
    ///
    /// [`Memfd`]: Memfd
    pub fn create<T: AsRef<str>>(&self, name: T) -> Result<Memfd, crate::Error> {
        let flags = self.bitflags();
        // SAFETY: A syscall is being invoked. It has soundness implications – in particular
        // `name_ptr` must be pointing to a valid null-terminated string. We construct a `CString`
        // in this `unsafe` block, ensuring both all invariants for the `name_ptr`.
        //
        // Furthermore `from_raw_fd` requires a valid file descriptor representing a `memfd`. This
        // is true by definition as we obtain said file descriptor from `memfd_create` syscall and
        // check the result for errors.
        unsafe {
            let cname =
                ffi::CString::new(name.as_ref()).map_err(crate::Error::NameCStringConversion)?;
            let name_ptr = cname.as_ptr();
            let fd = memfd_create(name_ptr, flags);
            if fd < 0 {
                return Err(crate::Error::Create(std::io::Error::last_os_error()));
            }
            Ok(Memfd::from_raw_fd(fd))
        }
    }
}

impl Default for MemfdOptions {
    fn default() -> Self {
        Self {
            allow_sealing: false,
            cloexec: true,
            hugetlb: None,
        }
    }
}

/// Page size for a hugetlb anonymous file.
#[allow(clippy::all)]
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
    /// Try to convert an object that owns a file descriptor into a `Memfd`.
    ///
    /// This function consumes the ownership of the specified object. If the underlying
    /// file-descriptor is compatible with memfd/sealing, a `Memfd` object is returned.
    /// Otherwise the supplied object is returned as error.
    pub fn try_from_fd<F>(fd: F) -> Result<Self, F>
    where
        F: AsRawFd + IntoRawFd,
    {
        if !is_memfd(&fd) {
            Err(fd)
        } else {
            // SAFETY: from_raw_fd requires a valid, uniquely owned file descriptor.
            // The IntoRawFd trait guarantees both conditions.
            let file = unsafe { fs::File::from_raw_fd(fd.into_raw_fd()) };
            Ok(Self { file })
        }
    }

    /// Try to convert a [`File`] object into a `Memfd`.
    ///
    /// This function consumes the ownership of the specified `File`.  If the underlying
    /// file-descriptor is compatible with memfd/sealing, a `Memfd` object is returned.
    /// Otherwise the supplied `File` is returned for further usage.
    ///
    /// [`File`]: fs::File
    pub fn try_from_file(file: fs::File) -> Result<Self, fs::File> {
        Self::try_from_fd(file)
    }

    /// Return a reference to the backing [`File`].
    ///
    /// [`File`]: fs::File
    pub fn as_file(&self) -> &fs::File {
        &self.file
    }

    /// Convert `Memfd` to the backing [`File`].
    ///
    /// [`File`]: fs::File
    pub fn into_file(self) -> fs::File {
        self.file
    }

    /// Obtain the current set of seals for the `Memfd`.
    pub fn seals(&self) -> Result<sealing::SealsHashSet, crate::Error> {
        let flags = Self::file_get_seals(&self.file)?;
        Ok(sealing::bitflags_to_seals(flags))
    }

    /// Add a seal to the existing set of seals.
    pub fn add_seal(&self, seal: sealing::FileSeal) -> Result<(), crate::Error> {
        use std::iter::FromIterator;

        let set = sealing::SealsHashSet::from_iter(vec![seal]);
        self.add_seals(&set)
    }

    /// Add some seals to the existing set of seals.
    pub fn add_seals(&self, seals: &sealing::SealsHashSet) -> Result<(), crate::Error> {
        let fd = self.file.as_raw_fd();
        let flags = sealing::seals_to_bitflags(seals);
        // UNSAFE(lucab): required syscall.
        let r = unsafe { libc::syscall(libc::SYS_fcntl, fd, libc::F_ADD_SEALS, flags) };
        if r < 0 {
            return Err(crate::Error::AddSeals(std::io::Error::last_os_error()));
        };
        Ok(())
    }

    /// Return the current sealing bitflags.
    fn file_get_seals(fp: &fs::File) -> Result<u64, crate::Error> {
        let fd = fp.as_raw_fd();
        // SAFETY: The syscall called has no soundness implications (i.e. does not mess with
        // process memory in weird ways, checks its arguments for correctness, etc.). Furthermore
        // due to invariants of `Memfd` this syscall is provided a valid file descriptor.
        let r = unsafe {
            libc::syscall(libc::SYS_fcntl, fd, libc::F_GET_SEALS)
        };
        if r < 0 {
            return Err(crate::Error::GetSeals(std::io::Error::last_os_error()));
        };
        Ok(r as u64)
    }
}

impl FromRawFd for Memfd {
    /// Convert a raw file-descriptor to a [`Memfd`].
    ///
    /// This function consumes ownership of the specified file descriptor. `Memfd` will take
    /// responsibility for closing it when the object goes out of scope.
    ///
    /// # Safety
    ///
    /// `fd` must be a valid file descriptor representing a memfd file.
    ///
    /// [`Memfd`]: Memfd
    unsafe fn from_raw_fd(fd: RawFd) -> Memfd {
        let file = fs::File::from_raw_fd(fd);
        Memfd { file }
    }
}

impl AsRawFd for Memfd {
    fn as_raw_fd(&self) -> RawFd {
        self.file.as_raw_fd()
    }
}

impl IntoRawFd for Memfd {
    fn into_raw_fd(self) -> RawFd {
        self.into_file().into_raw_fd()
    }
}

/// Check if a file descriptor is a memfd.
///
/// Implemented by trying to retrieve the seals.
/// If that fails, the fd is not a memfd.
fn is_memfd<F: AsRawFd>(fd: &F) -> bool {
    // SAFETY: The syscall called has no soundness implications (i.e. does not mess with
    // process memory in weird ways, checks its arguments for correctness, etc.).
    // The `AsRawFd` trait guarantees that the input is a valid file descriptor.
    let ret = unsafe { libc::syscall(libc::SYS_fcntl, fd.as_raw_fd(), libc::F_GET_SEALS) };
    ret >= 0
}
