use crate::MdpSuperblock1;
use anyhow::{anyhow, bail, Context, Result};
use device_mapper::ioctl;
use libc;
use std::ffi::CString;
use std::fs::OpenOptions;
use std::io::Error;
use std::os::linux::fs::MetadataExt;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::io::AsRawFd;

struct DiskMeta {
    superblock: MdpSuperblock1,
    major: u32,
    minor: u32,
}

pub fn assemble_array(disk_paths: &[&str], md_dev_num: u32) -> Result<()> {
    // Read metadata from disks
    let mut meta = Vec::new();
    for path in disk_paths {
        let md = std::fs::metadata(path).unwrap();
        let is_block = (md.st_mode() & libc::S_IFMT) == libc::S_IFBLK;
        if !is_block {
            bail!("{path} is not a block device, cannot be assembled")
        }
        let rdev = md.st_rdev();
        let major = unsafe { libc::major(rdev) }; // HOW IS THIS UNSAFE???
        let minor = unsafe { libc::minor(rdev) };
        let sb = MdpSuperblock1::from_file(path, 0x1000)?;
        meta.push(DiskMeta {
            superblock: sb,
            major,
            minor,
        });
    }

    // Validate that all superblocks belong to the same array
    let first_sb = &meta[0].superblock;
    let first_uuid = first_sb.array_info.uuid();
    for _meta in &meta[1..] {
        if _meta.superblock.array_info.uuid() != first_uuid {
            return Err(anyhow!("Disks do not belong to the same array"));
        }
    }

    let array_info = ioctl::mdu_array_info_t {
        major_version: 1,  // why
        minor_version: 2,  // why
        patch_version: 0,  // why
        ctime: 0,          // ???
        utime: 0,          // ???
        md_minor: -1,      // from mdadm, idk
        not_persistent: 0, // ??
        state: 1,          // from mdadm, idk
        nr_disks: meta.len() as i32,
        active_disks: meta.len() as i32, // mdadm does 0 at the beginning
        working_disks: meta.len() as i32,
        failed_disks: 0,
        spare_disks: 0,
        layout: first_sb.array_info.layout as i32,
        level: first_sb.array_info.level as i32,
        size: first_sb.array_info.size as i32,
        raid_disks: first_sb.array_info.raid_disks as i32,
        chunk_size: first_sb.array_info.chunksize as i32,
    };

    // Create a temporary device node

    let tmp_path = "/tmp/_tmp_node_pls_no_clobber";
    let tmp_c_path = CString::new(tmp_path)?;
    if std::fs::metadata(&tmp_path).is_ok() {
        std::fs::remove_file(&tmp_path).context("failed to delete tmp path")?;
    }
    unsafe {
        // this 1 == md<1>
        let dev = libc::makedev(9, md_dev_num);
        if libc::mknod(tmp_c_path.as_ptr(), libc::S_IFBLK | 0o660, dev) != 0 {
            return Err(anyhow!(
                "Can't mknod {}: {}",
                tmp_path,
                std::io::Error::last_os_error()
            ));
        }
    }

    // Open the temporary device
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .mode(0o600)
        .open(&tmp_path)
        .context(format!("Can't get fd (open) from {}", tmp_path))?;
    let fd = file.as_raw_fd();

    // remove the file so no one else can touch it - we still have the fd open
    std::fs::remove_file(&tmp_path).context("failed to delete tmp path")?;

    // if we previously did this half-way, then the array is
    // up but 'inactive' - we stop it blindly and ignore errors
    // should probably be cleaner
    unsafe { ioctl::stop_array(fd) };

    // Set array info
    let errno = unsafe { ioctl::set_array_info(fd, &array_info) };

    if errno != 0 {
        return Err(anyhow!(
            "Can't set_array_info: {}",
            Error::from_raw_os_error(errno)
        ));
    }

    // Add disks to the array
    for (i, meta) in meta.iter().enumerate() {
        let disk_info = ioctl::mdu_disk_info_t {
            major: meta.major as i32,
            minor: meta.minor as i32,
            number: i as i32,
            raid_disk: i as i32,
            // Assuming the disk is in a good state
            state: (1 << ioctl::MD_DISK_SYNC) | (1 << ioctl::MD_DISK_ACTIVE),
        };

        let errno = unsafe { ioctl::add_new_disk(fd, &disk_info) };
        if errno != 0 {
            return Err(anyhow!(
                "Can't add_new_disk: {}",
                Error::from_raw_os_error(errno)
            ));
        }
    }

    // Run the array
    let errno = unsafe { ioctl::run_array(fd, std::ptr::null()) };

    if errno != 0 {
        return Err(anyhow!(
            "Can't run_array: {}",
            Error::from_raw_os_error(errno)
        ));
    }

    Ok(())
}
