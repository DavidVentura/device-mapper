use device_mapper::{DeviceInfo, MdpSuperblock1};
use std::fs::File;
use std::io::prelude::*;
use std::os::unix::fs::FileExt;
use uuid::Uuid;

mod assemble;
/*
use std::os::unix::fs::FileExt;
use std::fs::{OpenOptions};
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

    /*
    let mut buf = [0x55u8; 256];
    let dev1f = File::open("device1").unwrap();
    dev1f.read_exact_at(&mut buf, 0x1000).unwrap();
    */
    //assemble::assemble_array(&vec!["my-device-1", "my-device-2"], "md123").unwrap();
    assemble::assemble_array(&vec!["/dev/loop1", "/dev/loop8"], 123).unwrap();
}

fn _create_example_array() {
    let target_size = 0xa00000; //10MiB

    let device_info_1 = DeviceInfo::new(
        target_size / 512, // device_size = 10M
        0,                 // dev_number: First device in the array
        None,              // device_uuid: Generate a new UUID
    );
    let device_info_2 = DeviceInfo::new(
        target_size / 512, // device_size = 10M
        1,                 // dev_number: First device in the array
        None,              // device_uuid: Generate a new UUID
    );

    let host = "computer";
    let array_name = "the-array";
    let array_uuid = Some(Uuid::new_v4());

    let sb1 = MdpSuperblock1::new(
        host,
        array_name,
        array_uuid,
        target_size / 512,
        2,
        device_info_1,
    )
    .unwrap();
    let sb2 = MdpSuperblock1::new(
        host,
        array_name,
        array_uuid,
        target_size / 512,
        2,
        device_info_2,
    )
    .unwrap();

    let mut f1 = File::create("my-device-1").unwrap();
    let mut f2 = File::create("my-device-2").unwrap();

    f1.seek(std::io::SeekFrom::Start(0x1000)).unwrap();
    f1.write_all(&sb1.as_bytes()).unwrap();

    // not sure why, but `mdadm --examine` will say
    // mdadm: No md superblock detected on my-device-1.
    // if there are less than 8KiB after the header
    // this is not a problem for normal block devices, only artificial cases using files
    f1.write_at(&[0], target_size - 1).unwrap();

    f2.seek(std::io::SeekFrom::Start(0x1000)).unwrap();
    f2.write_all(&sb2.as_bytes()).unwrap();
    f2.write_at(&[0], target_size - 1).unwrap();
}
