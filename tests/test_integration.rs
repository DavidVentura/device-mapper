use device_mapper::MdpSuperblock1;
use flate2;
use flate2::read::GzDecoder;
use std::io::prelude::*;
use uuid::Uuid;

fn read_gzipped_superblock(path: &str) -> MdpSuperblock1 {
    let compressed = std::fs::read(path).unwrap();
    let mut d = GzDecoder::new(compressed.as_slice());
    let mut buf = vec![0; 4096 + MdpSuperblock1::MAX_SIZE]; // first 4KiB are empty
    d.read(&mut buf).unwrap();
    let sb = MdpSuperblock1::from_bytes(&buf[0x1000..]).unwrap();
    sb
}

#[test]
fn test_raid1_device_1() {
    let sb = read_gzipped_superblock("tests/testdata/r1_d1.gz");
    assert_eq!(sb.array_info.name().unwrap(), "worklaptop:0");
    let fm = sb.array_info.feature_map;
    let level = sb.array_info.level;
    let dev_size = sb.array_info.size;
    let disks = sb.array_info.raid_disks;
    assert_eq!(fm, 0);
    assert_eq!(level, 1);
    assert_eq!(dev_size, 18432); // in 512b sectors
    assert_eq!(disks, 2);

    assert_eq!(sb.array_state_info.sb_csum, 0x9741e5f7);
    assert_eq!(
        sb.array_info.uuid(),
        Uuid::parse_str("24d684dd-bc67-60fc-a5d3-a49f592b1b42").unwrap()
    );

    assert_eq!(
        sb.device_info.uuid(),
        Uuid::parse_str("201e03cf-4205-c8bf-e714-52f868f6b6cd").unwrap()
    );
    assert_eq!(sb.calculate_sb_csum(), 0x9741e5f7);
}

#[test]
fn test_raid1_device_2() {
    let sb = read_gzipped_superblock("tests/testdata/r1_d2.gz");
    assert_eq!(sb.array_info.name().unwrap(), "worklaptop:0");
    assert_eq!(
        sb.array_info.uuid(),
        Uuid::parse_str("24d684dd-bc67-60fc-a5d3-a49f592b1b42").unwrap()
    );

    assert_eq!(
        sb.device_info.uuid(),
        Uuid::parse_str("fc9b0876-925c-3729-5f47-971af9ce24fc").unwrap()
    );
    assert_eq!(sb.calculate_sb_csum(), 0xf869c62b);
}

#[test]
fn test_array_info_consistency() {
    let sb1 = read_gzipped_superblock("tests/testdata/r1_d1.gz");
    let sb2 = read_gzipped_superblock("tests/testdata/r1_d2.gz");

    assert_eq!(sb1.array_info.name(), sb2.array_info.name());
    assert_eq!(sb1.array_info.uuid(), sb2.array_info.uuid());
    /*
    assert_eq!(sb1.array_info.level, sb2.array_info.level);
    assert_eq!(sb1.array_info.raid_disks, sb2.array_info.raid_disks);
    assert_eq!(sb1.array_info.size, sb2.array_info.size);
    */
}

#[test]
fn test_device_info_differences() {
    let sb1 = read_gzipped_superblock("tests/testdata/r1_d1.gz");
    let sb2 = read_gzipped_superblock("tests/testdata/r1_d2.gz");

    assert_ne!(sb1.device_info.uuid(), sb2.device_info.uuid());
    assert_ne!(sb1.array_state_info.sb_csum, sb2.array_state_info.sb_csum);
}

#[test]
fn test_creation_time() {
    let sb = read_gzipped_superblock("tests/testdata/r1_d1.gz");
    let creation_time = sb.array_info.creation();

    assert_eq!(
        creation_time.format("%Y-%m-%d %H:%M:%S").to_string(),
        "2024-08-13 09:34:43"
    );
    // creation is in UTC but mdadm shows in local timezone
    // can run mdadm with TZ=UTC env set
}

#[test]
fn test_feature_map() {
    let sb = read_gzipped_superblock("tests/testdata/r1_d1.gz");
    let fm = sb.array_info.feature_map;
    assert_eq!(fm, 0);
}

#[test]
fn test_array_size() {
    let sb = read_gzipped_superblock("tests/testdata/r1_d1.gz");
    let size = sb.array_info.size;
    assert_eq!(size, 18432); // 9.00 MiB in 512b sectors
}

#[test]
fn test_events_count() {
    let sb1 = read_gzipped_superblock("tests/testdata/r1_d1.gz");
    let sb2 = read_gzipped_superblock("tests/testdata/r1_d2.gz");

    assert_eq!(sb1.array_state_info.events, 16);
    assert_eq!(sb2.array_state_info.events, 16);
}
/*
TZ=UTC mdadm --examine testdata/r1_d1
r1_d1:
          Magic : a92b4efc
        Version : 1.2
    Feature Map : 0x0
     Array UUID : 24d684dd:bc6760fc:a5d3a49f:592b1b42
           Name : worklaptop:0  (local to host worklaptop)
  Creation Time : Tue Aug 13 09:34:43 2024
     Raid Level : raid1
   Raid Devices : 2

 Avail Dev Size : 18432 sectors (9.00 MiB 9.44 MB)
     Array Size : 9216 KiB (9.00 MiB 9.44 MB)
    Data Offset : 2048 sectors
   Super Offset : 8 sectors
   Unused Space : before=1968 sectors, after=0 sectors
          State : clean
    Device UUID : 201e03cf:4205c8bf:e71452f8:68f6b6cd

    Update Time : Tue Aug 13 09:34:43 2024
  Bad Block Log : 512 entries available at offset 16 sectors
       Checksum : 9741e5f7 - correct
         Events : 16


   Device Role : Active device 0
   Array State : AA ('A' == active, '.' == missing, 'R' == replacing)
*/

/*
r1_d2:
          Magic : a92b4efc
        Version : 1.2
    Feature Map : 0x0
     Array UUID : 24d684dd:bc6760fc:a5d3a49f:592b1b42
           Name : worklaptop:0  (local to host worklaptop)
  Creation Time : Tue Aug 13 09:34:43 2024
     Raid Level : raid1
   Raid Devices : 2

 Avail Dev Size : 18432 sectors (9.00 MiB 9.44 MB)
     Array Size : 9216 KiB (9.00 MiB 9.44 MB)
    Data Offset : 2048 sectors
   Super Offset : 8 sectors
   Unused Space : before=1968 sectors, after=0 sectors
          State : clean
    Device UUID : fc9b0876:925c3729:5f47971a:f9ce24fc

    Update Time : Tue Aug 13 09:34:43 2024
  Bad Block Log : 512 entries available at offset 16 sectors
       Checksum : f869c62b - correct
         Events : 16


   Device Role : Active device 1
   Array State : AA ('A' == active, '.' == missing, 'R' == replacing)
*/
