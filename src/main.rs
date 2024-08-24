use anyhow::{Context, Result};
use device_mapper::ioctl::blkgetsize64;
use device_mapper::{ArrayLevel, DeviceInfo, MdpSuperblock1};
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::os::unix::fs::FileExt;
use std::os::unix::io::AsRawFd;
use std::path::Path;
use uuid::Uuid;

mod assemble;

fn main() {
    _create_example_array()
    //assemble::assemble_array(&vec!["/dev/loop1", "/dev/loop8"], 123).unwrap();
}

fn get_size(path: &Path) -> Result<u64> {
    let metadata = path.metadata()?;
    if metadata.is_file() {
        Ok(metadata.len())
    } else {
        // assume it's a block
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
}

fn create_array(level: ArrayLevel, backing_devs: &[&str]) -> Result<()> {
    // minimum of backing_devs
    let target_size = 0xa00000; //10MiB

    let host = "computer";
    let array_name = "the-array";
    let array_uuid = Some(Uuid::new_v4());

    let mut array_size = u64::MAX;
    for dev in backing_devs {
        let path = Path::new(dev);
        let device_size = get_size(&path)?;
        array_size = array_size.min(device_size);
        println!("Device {} size {}", dev, device_size);
    }
    println!("Array size {}", array_size);

    for (i, dev) in backing_devs.iter().enumerate() {
        let block_size = 512;
        let path = Path::new(dev);
        let device_size = get_size(&path)?;

        let device_info = DeviceInfo::new(device_size / block_size, i as u32, None);
        let sb = MdpSuperblock1::new(
            host,
            array_name,
            array_uuid,
            array_size / block_size,
            backing_devs.len() as u32,
            device_info,
            level,
        )?;

        let mut f1 = File::create(dev)?;
        f1.seek(std::io::SeekFrom::Start(0x1000))?;
        f1.write_all(&sb.as_bytes())?;

        // not sure why, but `mdadm --examine` will say
        // mdadm: No md superblock detected on my-device-1.
        // if there are less than 8KiB after the header
        // this is not a problem for normal block devices, only artificial cases using files
        f1.write_at(&[0], target_size - 1)?;
    }
    Ok(())
}

fn _create_example_array() {
    create_array(ArrayLevel::Raid1, &vec!["/dev/loop1", "my-device-2"]).unwrap();
    //create_array(ArrayLevel::Raid1, &vec!["my-device-2", "/dev/loop1"]).unwrap();
}
