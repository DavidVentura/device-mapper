use device_mapper::{DeviceInfo, MdpSuperblock1};
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
/*
use device_mapper::ioctl::*;
use std::mem::MaybeUninit;
use std::os::fd::{AsRawFd, RawFd};
*/

fn main() {
    //let res = MdpSuperblock1::from_file("device1", 0x1000).unwrap();
    //println!(
    //    "{res:?}, array uuid {}, dev uuid {}, set name {}",
    //    res.array_info.uuid(),
    //    res.device_info.uuid(),
    //    res.array_info.name().unwrap(),
    //);

    //let oo = OpenOptions::new().read(true).write(true).open("/dev/md111");
    /*
    let f = File::open("/dev/md111").unwrap();
    let fd = f.as_raw_fd();
    println!("fd is {fd}");
    // unsafe { stop_array(fd) };

    let mut ai: MaybeUninit<mdu_array_info_t> = unsafe { std::mem::MaybeUninit::uninit() };
    let mut ab = unsafe { ai.assume_init() };
    let p_mut: *mut mdu_array_info_t = &mut ab;
    unsafe { get_array_info(fd, p_mut) };

    println!("{:?}", unsafe { *p_mut });

    drop(f);
    */
    // Create a new DeviceInfo with reasonable values
    let device_info_1 = DeviceInfo::new(
        0xa00000, // device_size = 10M
        0,        // dev_number: First device in the array
        None,     // device_uuid: Generate a new UUID
    );
    let device_info_2 = DeviceInfo::new(
        0xa00000, // device_size = 10M
        1,        // dev_number: First device in the array
        None,     // device_uuid: Generate a new UUID
    );

    let host = "computer";
    let array_name = "the-array";

    let sb1 = MdpSuperblock1::new(host, array_name, 0xa00000, 512, 2, device_info_1).unwrap();
    let sb2 = MdpSuperblock1::new(host, array_name, 0xa00000, 512, 2, device_info_2).unwrap();

    let mut f1 = File::create("my-device-1").unwrap();
    let mut f2 = File::create("my-device-2").unwrap();

    println!("superblock len {}", sb1.as_bytes().len());
    f1.seek(std::io::SeekFrom::Start(0x1000)).unwrap();
    f1.write_all(&sb1.as_bytes()).unwrap();

    f2.seek(std::io::SeekFrom::Start(0x1000)).unwrap();
    f2.write_all(&sb2.as_bytes()).unwrap();
}
