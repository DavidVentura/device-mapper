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
    println!("{:?}", sb.array_info.creation());
    println!("{:?}", sb.array_info);
    assert_eq!(level, 1);
    assert_eq!(dev_size, 18432); // in 512b sectors
    assert_eq!(disks, 2);

    println!("dev {:?}", sb.device_info);
    println!("arr state {:?}", sb.array_state_info);
    assert_eq!(sb.array_state_info.sb_csum, 0x9741e5f7);
    assert_eq!(
        sb.array_info.uuid(),
        Uuid::parse_str("24d684dd-bc67-60fc-a5d3-a49f592b1b42").unwrap()
    );

    assert_eq!(
        sb.device_info.uuid(),
        Uuid::parse_str("201e03cf-4205-c8bf-e714-52f868f6b6cd").unwrap()
    );
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
}
