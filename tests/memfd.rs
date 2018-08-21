extern crate memfd;
use std::os::unix::io::AsRawFd;

#[test]
fn test_memfd_default() {
    let opts = memfd::MemfdOptions::default();
    let m0 = opts.create("default").unwrap();
    let meta0 = m0.as_file().metadata().unwrap();
    assert_eq!(meta0.len(), 0);
    assert_eq!(meta0.is_file(), true);
    drop(m0)
}

#[test]
fn test_memfd_multi() {
    let opts = memfd::MemfdOptions::default();
    let m0 = opts.create("default").unwrap();
    let f0 = m0.as_file().as_raw_fd();

    let m1 = opts.create("default").unwrap();
    let f1 = m1.as_file().as_raw_fd();
    assert!(f0 != f1);

    let m0_file = m0.into_file();
    assert_eq!(f0, m0_file.as_raw_fd());
}
