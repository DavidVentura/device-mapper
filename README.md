# mdadm-rs

Can create a RAID1/RAID5 over devices (files/blockdevs).
Maybe it can create other types of RAID, but haven't checked.

It can also assemble devices as an array, with ~no error checking.
To assemble as an array, the kernel requires block devices, not files, so you can use a loop device.
