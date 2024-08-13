use arrayref::array_ref;
use std::fs::File;
use std::io::{self, Error, Read, Seek, SeekFrom};
use std::string::FromUtf8Error;
use uuid::Uuid;

#[repr(C, packed)]
#[derive(Debug)]
pub struct ArrayInfo {
    pub magic: u32,
    pub major_version: u32,
    pub feature_map: u32,
    _pad0: u32,
    set_uuid: [u8; 16],
    set_name: [u8; 32],
    ctime: u64,          // /* lo 40 bits are seconds, top 24 are microseconds or 0*/
    pub level: u32,      /* -4 (multipath), -1 (linear), 0,1,4,5 */
    pub layout: u32,     /* used for raid5, raid6, raid10, and raid0 */
    pub size: u64,       // in 512b sectors
    pub chunksize: u32,  // in 512b sectors
    pub raid_disks: u32, // count
    // Union of offset + size for MD_FEATURE_PPL
    // TODO
    opaque_union_bitmap_offset_ppl: u32,
}

impl ArrayInfo {
    const SUPERBLOCK_MAGIC: u32 = 0xa92b4efc;
    const MAJOR_VERSION: u32 = 1;

    pub fn creation(&self) -> chrono::NaiveDateTime {
        let seconds: u64 = self.ctime & 0xffffffff; // bottom 40b
        let micros: u32 = ((self.ctime & 0xffffff00000000) >> 40) as u32; // top 24b
        chrono::DateTime::from_timestamp(seconds as i64, micros * 1000)
            .unwrap()
            .naive_local()
    }
    pub fn name(&self) -> Result<String, FromUtf8Error> {
        let filtered: Vec<u8> = self
            .set_name
            .iter()
            .take_while(|x| **x > 0)
            .cloned()
            .collect();
        String::from_utf8(filtered)
    }
    pub fn uuid(&self) -> Uuid {
        Uuid::from_slice(&self.set_uuid).unwrap()
    }
    pub fn from_bytes(buf: &[u8; 100]) -> io::Result<Self> {
        let res: Self = unsafe { std::mem::transmute(*buf) };
        let magic = res.magic;
        if magic != ArrayInfo::SUPERBLOCK_MAGIC {
            return Err(Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid Magic, got {:x}", magic),
            ));
        }
        let major_version = res.major_version;
        if major_version != ArrayInfo::MAJOR_VERSION {
            return Err(Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid major version, got {:x}", major_version),
            ));
        }
        Ok(res)
    }
}

#[repr(C, packed)]
#[derive(Debug)]
pub struct FeatureBit4 {
    pub new_level: u32,
    pub reshape_position: u64,
    pub delta_disks: u32,
    pub new_layout: u32,
    pub new_chunk: u32,
    pub new_offset: u32,
}

impl FeatureBit4 {
    pub fn from_bytes(buf: &[u8; 28]) -> io::Result<Self> {
        let res: Self = unsafe { std::mem::transmute(*buf) };
        Ok(res)
    }
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
    pub fn from_bytes(buf: &[u8; 64]) -> io::Result<Self> {
        let res: Self = unsafe { std::mem::transmute(*buf) };
        Ok(res)
    }
    pub fn uuid(&self) -> Uuid {
        Uuid::from_slice(&self.device_uuid).unwrap()
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
}

impl ArrayStateInfo {
    pub fn from_bytes(buf: &[u8; 64]) -> io::Result<Self> {
        let res: Self = unsafe { std::mem::transmute(*buf) };
        Ok(res)
    }
}

#[derive(Debug)]
pub struct MdpSuperblock1 {
    pub array_info: ArrayInfo,
    pub feature_bit4: FeatureBit4,
    pub device_info: DeviceInfo,
    pub array_state_info: ArrayStateInfo,
    pub dev_roles: Vec<u16>,
}

impl MdpSuperblock1 {
    pub const MAX_SIZE: usize = 4096;
    pub fn from_bytes(buf: &[u8]) -> io::Result<Self> {
        let array_info = ArrayInfo::from_bytes(array_ref!(buf, 0, 100))?;
        let feature_bit4 = FeatureBit4::from_bytes(array_ref!(buf, 100, 28))?;
        // START of DeviceInfo is def 128
        let device_info = DeviceInfo::from_bytes(array_ref!(buf, 128, 64))?;
        let array_state_info = ArrayStateInfo::from_bytes(array_ref!(buf, 192, 64))?;

        // TODO
        let dev_roles = Vec::new();

        Ok(Self {
            array_info,
            feature_bit4,
            device_info,
            array_state_info,
            dev_roles,
        })
    }
    pub fn from_file(path: &str, offset: u64) -> io::Result<Self> {
        let mut file = File::open(path)?;

        file.seek(SeekFrom::Start(offset))?;
        let mut buf = [0; Self::MAX_SIZE];
        file.read_exact(&mut buf)?;

        Self::from_bytes(&buf)
    }
}
