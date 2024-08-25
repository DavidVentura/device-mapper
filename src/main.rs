use anyhow::{bail, Context, Result};
use chrono::Utc;
use device_mapper::ioctl::blkgetsize64;
use device_mapper::{ArrayLevel, DeviceInfo, MdpSuperblock1};
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::os::unix::io::AsRawFd;
use std::path::Path;
use uuid::Uuid;

mod assemble;

fn main() {
    /*
    let bad = format!(
        "BAD\n{:#?}",
        MdpSuperblock1::from_file("my-device-3", 0x1000)
    );
    let good = format!(
        "GOOD\n{:#?}",
        MdpSuperblock1::from_file("good-device-3", 0x1000)
    );
    std::fs::File::create("a")
        .unwrap()
        .write_all(&bad.as_bytes())
        .unwrap();
    std::fs::File::create("b")
        .unwrap()
        .write_all(&good.as_bytes())
        .unwrap();
        */
    _create_example_array();
    assemble::assemble_array(&vec!["/dev/loop1", "/dev/loop8", "/dev/loop9"], 123).unwrap();
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
    let host = "worklaptop";
    let array_name = "0";
    let array_uuid = Some(Uuid::new_v4());

    let mut min_disk_size = u64::MAX;
    for dev in backing_devs {
        let path = Path::new(dev);
        let device_size = get_size(&path)?;
        min_disk_size = min_disk_size.min(device_size);
        println!("Device {} size {}", dev, device_size);
    }
    if min_disk_size < 10_240 {
        // not sure why, but `mdadm --examine` will say
        // mdadm: No md superblock detected on my-device-1.
        // if there are less than 8KiB after the header
        // this is not a problem for normal block devices, only artificial cases using files
        bail!("Smallest block device is smaller than minimum acceptable (10KiB)")
    }
    let data_offset = match level {
        ArrayLevel::Raid1 => 0x800,  // why 1MB on top of superblock?
        ArrayLevel::Raid5 => 0x1000, // why 2MB on top of superblock?
        _ => todo!("unsupported"),
    };

    let now = Utc::now();
    for (i, dev) in backing_devs.iter().enumerate() {
        let block_size = 512;
        let path = Path::new(dev);
        let device_size = get_size(&path)?;

        let device_info = DeviceInfo::new(device_size / block_size, data_offset, i as u32, None);
        let sb = MdpSuperblock1::new(
            host,
            array_name,
            array_uuid,
            now,
            min_disk_size,
            block_size,
            backing_devs.len() as u32,
            device_info,
            level,
        )?;

        let mut file = OpenOptions::new().write(true).open(dev)?;
        file.seek(std::io::SeekFrom::Start(0x1000))?;
        file.write_all(&sb.as_bytes())?;
    }
    Ok(())
}

fn _create_example_array() {
    create_array(
        ArrayLevel::Raid1,
        &vec!["my-device-1", "my-device-2", "my-device-3"],
        //&vec!["my-device-1", "my-device-2"],
    )
    .unwrap();
    //create_array(ArrayLevel::Raid1, &vec!["my-device-2", "/dev/loop1"]).unwrap();
}
