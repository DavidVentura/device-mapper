use arrayref::array_ref;
use std::convert::From;
use std::fs::File;
use std::io::{self, Error, Read, Seek, SeekFrom};
use std::string::FromUtf8Error;
use std::time::Instant;
use uuid::Uuid;

pub mod ioctl;

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone, Copy)]
pub enum ArrayLevel {
    Linear = -1,
    Raid0 = 0,
    Raid1 = 1,
    Raid4 = 4,
    Raid5 = 5,
    Raid6 = 6,
    Raid10 = 10,
    Multipath = -4,
}

impl From<ArrayLevel> for u32 {
    fn from(level: ArrayLevel) -> Self {
        level as u32
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ArrayLayout {
    LeftAsymmetric = 0,
    RightAsymmetric = 1,
    LeftSymmetric = 2,
    RightSymmetric = 3,
}

impl From<ArrayLayout> for u32 {
    fn from(layout: ArrayLayout) -> Self {
        layout as u32
    }
}

fn instant_to_arrayinfo_format(instant: Instant) -> u64 {
    // FIXME: completely wrong
    let duration = instant.elapsed();
    let seconds = duration.as_secs();
    let microseconds = duration.subsec_micros();

    // Combine seconds (lower 40 bits) and microseconds (upper 24 bits)
    (seconds & 0xFFFFFFFFFF) | ((microseconds as u64) << 40)
}

fn str_to_bytes(s: &str) -> [u8; 32] {
    let mut bytes = [0u8; 32];
    bytes
        .get_mut(..s.len())
        .map(|slice| slice.copy_from_slice(s.as_bytes()));
    bytes
}

impl ArrayInfo {
    const SUPERBLOCK_MAGIC: u32 = 0xa92b4efc;
    const MAJOR_VERSION: u32 = 1;

    fn new(
        uuid: Uuid,
        name: &str,
        ctime: Instant,
        level: ArrayLevel,
        layout: ArrayLayout,
        size: u64,
        raid_disks: u32,
    ) -> ArrayInfo {
        ArrayInfo {
            magic: ArrayInfo::SUPERBLOCK_MAGIC,
            major_version: ArrayInfo::MAJOR_VERSION,
            feature_map: 0x0,
            _pad0: 0,
            set_uuid: uuid.into_bytes(),
            set_name: str_to_bytes(name),
            ctime: instant_to_arrayinfo_format(ctime),
            level: level.into(),
            layout: layout.into(),
            size: size - 2048, // reserve 1MB?
            chunksize: 0,
            raid_disks,
            opaque_union_bitmap_offset_ppl: 0, // TODO?
        }
    }
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
    pub fn as_bytes(&self) -> [u8; 100] {
        unsafe { std::mem::transmute(*self) }
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
#[derive(Debug, Clone, Copy)]
pub struct FeatureBit4 {
    pub new_level: u32,
    pub reshape_position: u64,
    pub delta_disks: u32,
    pub new_layout: u32,
    pub new_chunk: u32,
    pub new_offset: u32,
}

impl FeatureBit4 {
    pub fn as_bytes(&self) -> [u8; 28] {
        unsafe { std::mem::transmute(*self) }
    }
    pub fn from_bytes(buf: &[u8; 28]) -> io::Result<Self> {
        let res: Self = unsafe { std::mem::transmute(*buf) };
        Ok(res)
    }
}
#[repr(C)]
#[derive(Debug, Clone, Copy)]
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
    pub fn new(device_size: u64, dev_number: u32, device_uuid: Option<Uuid>) -> Self {
        DeviceInfo {
            data_offset: 0x800,            // why; on top of superblock offset?
            data_size: device_size - 2048, // reserve 1MB? why?
            super_offset: 0x8,             // 8* 512b block = 4KiB = 0x1000
            recovery_offset: 0,
            dev_number,
            cnt_corrected_read: 0, // Initialize to 0
            device_uuid: device_uuid.unwrap_or_else(Uuid::new_v4).into_bytes(),
            devflags: 0,
            bblog_shift: 0,
            bblog_size: 8,    // ?
            bblog_offset: 16, // ?
        }
    }

    pub fn as_bytes(&self) -> [u8; 64] {
        unsafe { std::mem::transmute(*self) }
    }
    pub fn from_bytes(buf: &[u8; 64]) -> io::Result<Self> {
        let res: Self = unsafe { std::mem::transmute(*buf) };
        Ok(res)
    }
    pub fn uuid(&self) -> Uuid {
        Uuid::from_slice(&self.device_uuid).unwrap()
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
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
    pub fn as_bytes(&self) -> [u8; 64] {
        unsafe { std::mem::transmute(*self) }
    }
}

#[derive(Debug, Clone)]
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
        if buf.len() < 256 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Buffer too short for MdpSuperblock1",
            ));
        }

        let array_info = ArrayInfo::from_bytes(array_ref!(buf, 0, 100))?;
        let feature_bit4 = FeatureBit4::from_bytes(array_ref!(buf, 100, 28))?;
        let device_info = DeviceInfo::from_bytes(array_ref!(buf, 128, 64))?;
        let array_state_info = ArrayStateInfo::from_bytes(array_ref!(buf, 192, 64))?;

        let dev_roles_count = array_state_info.max_dev as usize;

        if buf.len() < 256 + dev_roles_count * 2 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Buffer too short for dev_roles",
            ));
        }

        // fixed-size part of the superblock is 256b
        let dev_roles = buf[256..]
            .chunks_exact(2)
            .take(dev_roles_count)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .collect();

        Ok(Self {
            array_info,
            feature_bit4,
            device_info,
            array_state_info,
            dev_roles,
        })
    }

    pub fn new(
        host: &str,
        name: &str,
        uuid: Option<Uuid>,
        size_bytes: u64,
        disk_count: u32,
        device_info: DeviceInfo,
        raid_level: ArrayLevel,
    ) -> Result<MdpSuperblock1, impl std::error::Error> {
        if host.len() + name.len() > 32 {
            return Err(std::io::Error::new(
                io::ErrorKind::InvalidInput,
                "Length of host + name must be <=32",
            ));
        }
        let array_uuid = uuid.unwrap_or_else(Uuid::new_v4);

        let now = Instant::now();
        let array_info = ArrayInfo::new(
            array_uuid,
            &format!("{host}:{name}"),
            now,
            raid_level,
            ArrayLayout::LeftAsymmetric,
            size_bytes,
            disk_count,
        );

        // Create dummy FeatureBit4 -- no idea what for
        let feature_bit4 = FeatureBit4 {
            new_level: 0,
            reshape_position: 0,
            delta_disks: 0,
            new_layout: 0,
            new_chunk: 0,
            new_offset: 0,
        };

        let max_dev = 0x80; // 128
        let array_state_info = ArrayStateInfo {
            utime: 0,
            events: 16, // why?
            resync_offset: 0xffffffffffffffff,
            sb_csum: 0,
            max_dev,
            pad3: [0; 32],
        };

        let mut dev_roles = vec![0xffff as u16; max_dev as usize];
        for i in 0..disk_count {
            dev_roles[i as usize] = i as u16;
        }

        let mut sb = MdpSuperblock1 {
            array_info,
            feature_bit4,
            device_info,
            array_state_info,
            dev_roles,
        };
        let csum = sb.calculate_sb_csum();
        sb.array_state_info.sb_csum = csum;
        Ok(sb)
    }

    pub fn calculate_sb_csum(&self) -> u32 {
        // checksum is calculated with checksum set to 0
        let mut new_sb = self.clone();
        new_sb.array_state_info.sb_csum = 0;

        let bytes = &new_sb.as_bytes();

        let mut csum: u64 = 0;
        for chunk in bytes.chunks(4) {
            let value = match chunk.len() {
                4 => u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]),
                2 => u16::from_le_bytes([chunk[0], chunk[1]]) as u32,
                _ => 0,
            };
            csum += value as u64;
        }

        // Fold the upper 32 bits into the lower 32 bits
        ((csum & 0xffffffff) + (csum >> 32)) as u32
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut byte_vec: Vec<u8> = Vec::new();

        // Assuming you have dev_roles as Vec<u16>
        for &role in &self.dev_roles {
            byte_vec.extend_from_slice(&role.to_le_bytes());
        }

        let mut ret = Vec::with_capacity(4096);
        ret.extend_from_slice(&self.array_info.as_bytes());
        ret.extend_from_slice(&self.feature_bit4.as_bytes());
        ret.extend_from_slice(&self.device_info.as_bytes());
        ret.extend_from_slice(&self.array_state_info.as_bytes());
        ret.extend_from_slice(&byte_vec);
        ret
    }
    pub fn from_file(path: &str, offset: u64) -> io::Result<Self> {
        let mut file = File::open(path)?;

        file.seek(SeekFrom::Start(offset))?;
        let mut buf = [0; Self::MAX_SIZE];
        file.read_exact(&mut buf)?;

        Self::from_bytes(&buf)
    }
}
