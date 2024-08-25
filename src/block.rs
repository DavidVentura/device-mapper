use anyhow::{bail, Context, Result};
use device_mapper::ioctl::blkgetsize64;
use std::fs::OpenOptions;
use std::os::linux::fs::MetadataExt;
use std::os::unix::io::AsRawFd;
use std::path::Path;

pub fn is_block<P: AsRef<Path>>(path: P) -> Result<bool> {
    let md = std::fs::metadata(path)?;
    Ok((md.st_mode() & libc::S_IFMT) == libc::S_IFBLK)
}
pub fn get_size(path: &Path) -> Result<u64> {
    let metadata = path.metadata()?;
    if metadata.is_file() {
        return Ok(metadata.len());
    }
    if !is_block(path)? {
        bail!("Not a file and not a block device");
    }

    let file = OpenOptions::new()
        .read(true)
        .open(path)
        .context(format!("Can't get fd (open) from {:?}", path))?;
    let fd = file.as_raw_fd();
    let mut size: u64 = 0;
    let size_ptr = &mut size as *mut u64;

    unsafe { blkgetsize64(fd, size_ptr) };
    Ok(size)
}
