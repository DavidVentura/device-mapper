use device_mapper::MdpSuperblock1;

fn main() {
    let res = MdpSuperblock1::from_file("device1", 0x1000).unwrap();
    println!(
        "{res:?}, array uuid {}, dev uuid {}, set name {}",
        res.array_info.uuid(),
        res.device_info.uuid(),
        res.array_info.name().unwrap(),
    );
}
