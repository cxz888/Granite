use crate::sync::UPSafeCell;
use crate::utils::error::{code, Result};
use crate::{driver_impl::BLOCK_DEVICE, memory::UserBuffer};

use alloc::borrow::ToOwned;
use alloc::string::String;
use alloc::{sync::Arc, vec::Vec};
use bitflags::bitflags;
use fat32::{Fat32, Fat32Entry};
use lazy_static::lazy_static;
use vfs::{Entry, Fs};

use super::{File, Stdout};

type OsEntry = Fat32Entry;
type Vfs = Fat32;

/// A wrapper around a filesystem inode
/// to implement File trait atop
pub struct OSFile {
    readable: bool,
    writable: bool,
    path: String,
    inner: UPSafeCell<OSFileInner>,
}

/// The OS inode inner in 'UPSafeCell'
pub struct OSFileInner {
    entry: OsEntry,
    flags: OpenFlags,
}

impl OSFile {
    /// Construct an OS inode from a inode
    pub fn new(path: String, readable: bool, writable: bool, entry: OsEntry) -> Self {
        Self {
            path,
            readable,
            writable,
            inner: unsafe {
                UPSafeCell::new(OSFileInner {
                    entry,
                    flags: OpenFlags::empty(),
                })
            },
        }
    }
    /// Read all data inside a inode into vector
    pub fn read_all(&self) -> Vec<u8> {
        let mut inner = self.inner.exclusive_access();
        inner.entry.read_all().unwrap()
    }
}

lazy_static! {
    /// The root of all inodes, or '/' in short
    pub static ref VIRTUAL_FS: Vfs = {
        Vfs::new(Arc::clone(&BLOCK_DEVICE))
    };
}

/// List all files in the filesystems
pub fn list_apps() {
    println!("/**** APPS ****");
    for app in VIRTUAL_FS.root_dir().ls().unwrap() {
        println!("{}", app);
    }
    println!("**************/");
}

bitflags! {
    /// 注意低 2 位指出文件的打开模式
    /// 0、1、2 分别对应只读、只写、可读可写。3 为错误。
    pub struct OpenFlags: u32 {
        const O_RDONLY      = 0;
        const O_WRONLY      = 1 << 0;
        const O_RDWR        = 1 << 1;

        /// 如果所查询的路径不存在，则在该路径创建一个常规文件
        const O_CREAT       = 1 << 6;
        /// 在创建文件的情况下，保证该文件之前已经已存在，否则返回错误
        const O_EXCL        = 1 << 7;
        /// 如果路径指向一个终端设备，那么它不会称为本进程的控制终端
        const O_NOCTTY      = 1 << 8;
        /// 如果是常规文件，且允许写入，则将该文件长度截断为 0
        const O_TRUNC       = 1 << 9;
        /// 写入追加到文件末尾，可能在每次 `sys_write` 都有影响，暂时不支持
        const O_APPEND      = 1 << 10;
        /// 保持文件数据与磁盘阻塞同步。但如果该写操作不影响读取刚写入的数据，则不会等到元数据更新，暂不支持
        const O_DSYNC       = 1 << 12;
        /// 文件操作完成时发出信号，暂时不支持
        const O_ASYNC       = 1 << 13;
        /// 不经过缓存，直接写入磁盘中。目前实现仍然经过缓存
        const O_DIRECT      = 1 << 14;
        /// 允许打开文件大小超过 32 位表示范围的大文件。在 64 位系统上此标志位应永远为真
        const O_LARGEFILE   = 1 << 15;
        /// 如果打开的文件不是目录，那么就返回失败
        const O_DIRECTORY   = 1 << 16;
        // /// 如果路径的 basename 是一个符号链接，则打开失败并返回 `ELOOP`，目前不支持
        // const O_NOFOLLOW    = 1 << 17;
        // /// 读文件时不更新文件的 last access time，暂不支持
        // const O_NOATIME     = 1 << 18;
        /// 设置打开的文件描述符的 close-on-exec 标志
        const O_CLOEXEC     = 1 << 19;
        // /// 仅打开一个文件描述符，而不实际打开文件。后续只允许进行纯文件描述符级别的操作
        // const O_PATH        = 1 << 21;
    }
}

impl OpenFlags {
    /// Get the current read write permission on an inode
    /// does not check validity for simplicity
    /// returns (readable, writable)
    pub fn read_write(&self) -> (bool, bool) {
        match self.bits & 0b11 {
            0 => (true, false),
            1 => (false, true),
            2 => (true, true),
            _ => unreachable!(),
        }
    }
}

impl File for OSFile {
    fn readable(&self) -> bool {
        self.readable
    }
    fn writable(&self) -> bool {
        self.writable
    }
    fn read(&self, mut buf: UserBuffer) -> usize {
        todo!()
        // let mut inner = self.inner.exclusive_access();
        // let mut total_read_size = 0usize;
        // for slice in buf.buffers.iter_mut() {
        //     let read_size = inner.entry.read_at(inner.offset, slice);
        //     if read_size == 0 {
        //         break;
        //     }
        //     inner.offset += read_size;
        //     total_read_size += read_size;
        // }
        // total_read_size
    }
    fn write(&self, buf: UserBuffer) -> usize {
        todo!()
        // let mut inner = self.inner.exclusive_access();
        // let mut total_write_size = 0usize;
        // for slice in buf.buffers.iter() {
        //     let write_size = inner.inode.write_at(inner.offset, slice);
        //     assert_eq!(write_size, slice.len());
        //     inner.offset += write_size;
        //     total_write_size += write_size;
        // }
        // total_write_size
    }
    fn set_close_on_exec(&self, bit: bool) {
        self.inner
            .exclusive_access()
            .flags
            .set(OpenFlags::O_CLOEXEC, bit);
    }
    fn status(&self) -> OpenFlags {
        self.inner.exclusive_access().flags
    }
    fn is_dir(&self) -> bool {
        self.inner.exclusive_access().entry.is_dir()
    }
    fn path(&self) -> Option<&str> {
        Some(&self.path)
    }
}

/// 打开一个磁盘上的文件
pub fn open_osfile(path: String, flags: OpenFlags) -> Result<OSFile> {
    let (readable, writable) = flags.read_write();
    let mut curr = VIRTUAL_FS.root_dir();
    let mut path_split = path.split('/');
    while let Some(name) = path_split.next() {
        match curr.find(&name) {
            Ok(Some(next)) => {
                curr = next;
            }
            Ok(None) => {
                // 最后一节路径未找到，若有 O_CREAT 则创建；否则返回 ENOENT
                if path_split.next().is_none() && flags.contains(OpenFlags::O_CREAT) {
                    let file = curr.create(&name).unwrap();
                    return Ok(OSFile::new(path, readable, writable, file));
                } else {
                    return Err(code::ENOENT);
                }
            }
            Err(vfs::Error::InvalidType) => {
                return Err(code::ENOTDIR);
            }
            Err(e) => {
                panic!("文件系统内部错误：{e:?}");
            }
        }
    }
    // 运行到此处说明未创建文件
    if flags.contains(OpenFlags::O_CREAT | OpenFlags::O_EXCL) {
        return Err(code::EEXIST);
    }
    if flags.contains(OpenFlags::O_TRUNC) && flags.read_write().1 {
        curr.clear();
    }
    Ok(OSFile::new(path, readable, writable, curr))
}

/// 根据路径打开一个文件。包括特殊文件
pub fn open_file(path: String, flags: OpenFlags) -> Result<Arc<dyn File>> {
    if path.starts_with("/dev") {
        match path.as_str() {
            "/dev/tty" => return Ok(Arc::new(Stdout)),
            _ => return Err(code::ENOENT),
        }
    }
    let osfile = open_osfile(path, flags)?;
    Ok(Arc::new(osfile))
}
