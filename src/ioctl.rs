/* usage */
/*
#define RUN_ARRAY               _IOW (MD_MAJOR, 0x30, mdu_param_t)
#define STOP_ARRAY              _IO (MD_MAJOR, 0x32)
#define SET_ARRAY_INFO          _IOW (MD_MAJOR, 0x23, mdu_array_info_t)
#define ADD_NEW_DISK            _IOW (MD_MAJOR, 0x21, mdu_disk_info_t)
#define GET_DISK_INFO           _IOR (MD_MAJOR, 0x12, mdu_disk_info_t)
#define GET_ARRAY_INFO          _IOR (MD_MAJOR, 0x11, mdu_array_info_t)

*/

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use ioctl_sys::ioctl;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

// Assuming MD_MAJOR is defined elsewhere, let's define it here for completeness
const MD_MAJOR: u8 = 9; // This value may need to be adjusted

// IOCTL definitions
ioctl!(write run_array with MD_MAJOR, 0x30; mdu_param_t);
ioctl!(none stop_array with MD_MAJOR, 0x32);

ioctl!(read get_array_info with MD_MAJOR, 0x11; mdu_array_info_t);
ioctl!(write set_array_info with MD_MAJOR, 0x23; mdu_array_info_t);

ioctl!(write add_new_disk with MD_MAJOR, 0x21; mdu_disk_info_t);

ioctl!(read get_disk_info with MD_MAJOR, 0x12; mdu_disk_info_t);

const BLKGETSIZE64_CODE: u8 = 0x12; // Defined in linux/fs.h
const BLKGETSIZE64_SEQ: u8 = 114;
ioctl!(read blkgetsize64 with BLKGETSIZE64_CODE, BLKGETSIZE64_SEQ; u64);
