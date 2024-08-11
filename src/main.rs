use device_mapper::{ArrayInfo, DeviceInfo};

fn main() {
    println!("Hello, world!");
    let res = ArrayInfo::read_from_file("../blog/device1").unwrap();
    println!("{res:?}");
    println!("{}", res.name().unwrap());

    let res = DeviceInfo::read_from_file("../blog/device1").unwrap();
    println!("{res:?}");
}
