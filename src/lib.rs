use byteorder::{LittleEndian, ReadBytesExt};
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::string::FromUtf8Error;

#[repr(C)]
#[derive(Debug)]
pub struct ArrayInfo {
    pub magic: u32,
    pub major_version: u32,
    pub feature_map: u32,
    pub pad0: u32,
    pub set_uuid: [u8; 16],
    pub set_name: [u8; 32],
    pub ctime: u64,      // /* lo 40 bits are seconds, top 24 are microseconds or 0*/
    pub level: u32,      /* -4 (multipath), -1 (linear), 0,1,4,5 */
    pub layout: u32,     /* used for raid5, raid6, raid10, and raid0 */
    pub size: u64,       // in 512b sectors
    pub chunksize: u32,  // in 512b sectors
    pub raid_disks: u32, // count
    // Union of offset + size for MD_FEATURE_PPL
    // TODO
    pub bitmap_offset: u32,
}

impl ArrayInfo {
    pub fn name(&self) -> Result<String, FromUtf8Error> {
        let filtered: Vec<u8> = self
            .set_name
            .iter()
            .take_while(|x| **x > 0)
            .cloned()
            .collect();
        String::from_utf8(filtered)
    }
    pub fn read_from_file(path: &str) -> io::Result<Self> {
        let mut file = File::open(path)?;

        // Seek to the 0x1000 (4096) byte offset
        file.seek(SeekFrom::Start(0x1000))?;
        let magic = file.read_u32::<LittleEndian>()?;
        let major_version = file.read_u32::<LittleEndian>()?;
        let feature_map = file.read_u32::<LittleEndian>()?;
        let pad0 = file.read_u32::<LittleEndian>()?;

        let mut set_uuid = [0u8; 16];
        file.read_exact(&mut set_uuid)?;

        let mut set_name = [0u8; 32];
        file.read_exact(&mut set_name)?;

        let ctime = file.read_u64::<LittleEndian>()?;
        let level = file.read_u32::<LittleEndian>()?;
        let layout = file.read_u32::<LittleEndian>()?;
        let size = file.read_u64::<LittleEndian>()?;
        let chunksize = file.read_u32::<LittleEndian>()?;
        let raid_disks = file.read_u32::<LittleEndian>()?;
        let bitmap_offset = file.read_u32::<LittleEndian>()?;

        Ok(ArrayInfo {
            magic,
            major_version,
            feature_map,
            pad0,
            set_uuid,
            set_name,
            ctime,
            level,
            layout,
            size,
            chunksize,
            raid_disks,
            bitmap_offset,
        })
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct FeatureBit4 {
    pub new_level: u32,
    pub reshape_position: u64,
    pub delta_disks: u32,
    pub new_layout: u32,
    pub new_chunk: u32,
    pub new_offset: u32,
}

#[repr(C)]
#[derive(Debug)]
pub struct DeviceInfo {
    pub data_offset: u64,
    pub data_size: u64,
    pub super_offset: u64,
    // TODO
    // Union of recovery_offset and journal_tail
    pub recovery_offset: u64, // Using recovery_offset instead of the union
    pub dev_number: u32,
    pub cnt_corrected_read: u32,
    pub device_uuid: [u8; 16],
    pub devflags: u8,
    pub bblog_shift: u8,
    pub bblog_size: u16,
    pub bblog_offset: u32,
}

impl DeviceInfo {
    pub fn read_from_file(path: &str) -> io::Result<Self> {
        let mut file = File::open(path)?;

        // Calculate the offset for DeviceInfo
        // It starts after ArrayInfo and FeatureBit4
        let sizeof_arrayinfo = 96;
        let sizeof_featurebit4 = 32;
        //DeviceInfo: Size: 64 bytes
        //ArrayStateInfo: Size: 64 bytes
        let offset = 0x1000 + sizeof_featurebit4 + sizeof_arrayinfo;
        file.seek(SeekFrom::Start(offset as u64))?;

        // 128
        let data_offset = file.read_u64::<LittleEndian>()?;
        // 136
        let data_size = file.read_u64::<LittleEndian>()?;
        let super_offset = file.read_u64::<LittleEndian>()?;
        let recovery_offset = file.read_u64::<LittleEndian>()?;
        let dev_number = file.read_u32::<LittleEndian>()?;
        let cnt_corrected_read = file.read_u32::<LittleEndian>()?;

        let mut device_uuid = [0u8; 16];
        file.read_exact(&mut device_uuid)?;

        let devflags = file.read_u8()?;
        let bblog_shift = file.read_u8()?;
        let bblog_size = file.read_u16::<LittleEndian>()?;
        let bblog_offset = file.read_u32::<LittleEndian>()?;

        Ok(DeviceInfo {
            data_offset,
            data_size,
            super_offset,
            recovery_offset,
            dev_number,
            cnt_corrected_read,
            device_uuid,
            devflags,
            bblog_shift,
            bblog_size,
            bblog_offset,
        })
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct ArrayStateInfo {
    pub utime: u64,
    pub events: u64,
    pub resync_offset: u64,
    pub sb_csum: u32,
    pub max_dev: u32,
    pub pad3: [u8; 32],
    // Note: We're using a fixed size for pad3 instead of 64-32
    // You may want to adjust this based on your specific needs
}

#[derive(Debug)]
pub struct MdpSuperblock1 {
    pub array_info: ArrayInfo,
    pub feature_bit4: FeatureBit4,
    pub device_info: DeviceInfo,
    pub array_state_info: ArrayStateInfo,
    pub dev_roles: Vec<u16>,
}
