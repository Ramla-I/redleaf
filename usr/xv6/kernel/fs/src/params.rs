use core::mem;
use crate::icache;

// https://github.com/mit-pdos/xv6-public/blob/master/param.h
pub const NOFILE: usize =       16;     // open files per process
pub const NFILE: usize =       100;     // open files per system
pub const NINODE: usize =       32;     // maximum number of active i-nodes
pub const NDEV: i16 =           10;     // maximum major device number
pub const ROOTDEV: u32 =       1;     // device number of file system root disk
pub const MAXOPBLOCKS: usize =  10;     // max # of blocks any FS op writes
pub const LOGSIZE: usize =      MAXOPBLOCKS*3;  // max data blocks in on-disk log
pub const NBUF: usize =         MAXOPBLOCKS*3;  // size of disk block cache
pub const BSIZE: usize =        512;   // block size
pub const BPB: usize =          BSIZE * 8; // bits per block
pub const FSSIZE: usize =       1000;   // size of file system in blocks

// https://github.com/mit-pdos/xv6-riscv/blob/riscv/kernel/fs.h
pub const ROOTINO: u16   =      1;      // root i-number
pub const FSMAGIC: usize =      0x1020_3040;
pub const NDIRECT: usize =      12;
pub const NINDIRECT: usize =    BSIZE / mem::size_of::<u32>();
pub const MAXFILE: usize =      NDIRECT + NINDIRECT;

// Inodes per block.
pub const IPB: usize =           BSIZE / mem::size_of::<icache::DINode>();

// Directory is a file containing a sequence of dirent structures.
pub const DIRSIZ: usize =       14;
