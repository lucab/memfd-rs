/* from <sys/memfd.h> */

pub(super) const MFD_CLOEXEC: u32 = 1;
pub(super) const MFD_ALLOW_SEALING: u32 = 2;
pub(super) const MFD_HUGETLB: u32 = 4;

/* from <asm-generic/hugetlb_encode.h> */

pub(super) const MFD_HUGE_SHIFT: u32 = 26;

pub(super) const MFD_HUGE_64KB: u32 = 16 << MFD_HUGE_SHIFT;
pub(super) const MFD_HUGE_512KB: u32 = 19 << MFD_HUGE_SHIFT;
pub(super) const MFD_HUGE_1MB: u32 = 20 << MFD_HUGE_SHIFT;
pub(super) const MFD_HUGE_2MB: u32 = 21 << MFD_HUGE_SHIFT;
pub(super) const MFD_HUGE_8MB: u32 = 23 << MFD_HUGE_SHIFT;
pub(super) const MFD_HUGE_16MB: u32 = 24 << MFD_HUGE_SHIFT;
pub(super) const MFD_HUGE_256MB: u32 = 28 << MFD_HUGE_SHIFT;
pub(super) const MFD_HUGE_1GB: u32 = 30 << MFD_HUGE_SHIFT;
pub(super) const MFD_HUGE_2GB: u32 = 31 << MFD_HUGE_SHIFT;
pub(super) const MFD_HUGE_16GB: u32 = 34 << MFD_HUGE_SHIFT;
