//! 用于从操作系统镜像中抽取文件内容

use std::{fs::File, io::Read};

use fatfs::{FileSystem, FsOptions};

fn main() {
    let fs = File::options()
        .read(true)
        .write(true)
        .open("../res/fat32.img")
        .unwrap();
    let fs = FileSystem::new(fs, FsOptions::new()).unwrap();
    let root_dir = fs.root_dir();
    let mut data = Vec::new();
    root_dir
        .open_file("lua")
        .unwrap()
        .read_to_end(&mut data)
        .unwrap();
    std::fs::write("lua.elf", data).unwrap();
}
