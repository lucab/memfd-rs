use std::collections::HashSet;

/// An `HashSet` specialized on `FileSeal`.
pub type SealsHashSet = HashSet<FileSeal>;

/// Seal that can be applied to a [`Memfd`].
///
/// [`Memfd`]: crate::Memfd
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum FileSeal {
    /// File cannot be reduced in size.
    ///
    /// Corresponds to `F_SEAL_SHRINK`.
    SealShrink,
    /// File cannot be grown in size.
    ///
    /// Corresponds to `F_SEAL_GROW`.
    SealGrow,
    /// File cannot be written.
    ///
    /// Corresponds to `F_SEAL_WRITE`.
    SealWrite,
    /// File sealing cannot be further manipulated.
    ///
    /// Corresponds to `F_SEAL_SEAL`.
    SealSeal,
}

impl FileSeal {
    /// Return the bit-wise flag value of this seal.
    pub(crate) fn bitflags(self) -> u32 {
        let b = match self {
            FileSeal::SealSeal => libc::F_SEAL_SEAL,
            FileSeal::SealShrink => libc::F_SEAL_SHRINK,
            FileSeal::SealGrow => libc::F_SEAL_GROW,
            FileSeal::SealWrite => libc::F_SEAL_WRITE,
        };
        b as u32
    }
}

/// Convert a set of seals into a bitflags value.
pub(crate) fn seals_to_bitflags(set: &SealsHashSet) -> u32 {
    let mut bits = 0;
    for seal in set.iter() {
        bits |= seal.bitflags();
    }
    bits
}

/// Convert a bitflags value to a set of seals.
pub(crate) fn bitflags_to_seals(bitflags: u64) -> SealsHashSet {
    let mut sset = SealsHashSet::new();
    if bitflags & (libc::F_SEAL_SEAL as u64) != 0 {
        sset.insert(FileSeal::SealSeal);
    }
    if bitflags & (libc::F_SEAL_SHRINK as u64) != 0 {
        sset.insert(FileSeal::SealShrink);
    }
    if bitflags & (libc::F_SEAL_GROW as u64) != 0 {
        sset.insert(FileSeal::SealGrow);
    }
    if bitflags & (libc::F_SEAL_WRITE as u64) != 0 {
        sset.insert(FileSeal::SealWrite);
    }
    sset
}
