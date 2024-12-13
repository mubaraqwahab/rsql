use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

pub const BLOCK_SIZE: usize = 4096;

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Block<'a> {
    pub filename: &'a str,
    pub num: usize,
}

impl Block<'_> {
    // TODO: make this "synchronized" a la java
    pub fn read(&self, page: &mut Page) {
        let offset = (self.num * BLOCK_SIZE) as u64;
        let mut file = File::open(self.filename).unwrap();
        file.seek(SeekFrom::Start(offset)).unwrap();
        file.read(&mut page.buffer).unwrap();
    }

    // TODO: make this "synchronized" a la java
    pub fn write(&mut self, page: &Page) {
        let offset = (self.num * BLOCK_SIZE) as u64;
        let mut file = File::options().write(true).open(self.filename).unwrap();
        file.seek(SeekFrom::Start(offset)).unwrap();

        file.write(&page.buffer).unwrap();
        file.sync_all().unwrap();
    }

    // TODO: consider adding a new(filename) function that's equivalent to fm.append(filename)
}

pub struct Page {
    buffer: [u8; BLOCK_SIZE],
}

#[derive(Debug)]
pub struct FileMgr<'a> {
    dir: &'a Path,
    open_files: HashMap<&'a str, File>,
}

impl<'a> FileMgr<'a> {
    pub fn new(dir: &'a Path) -> Self {
        fs::create_dir_all(dir).unwrap();

        // Remove any leftover temp files
        for entry in dir.read_dir().unwrap() {
            let entry_name = entry.unwrap().file_name().into_string().unwrap();
            if entry_name.starts_with("temp") {
                fs::remove_file(dir.join(entry_name)).unwrap();
            }
        }

        Self {
            dir,
            open_files: HashMap::new(),
        }
    }

    // TODO: make this "synchronized" a la java
    pub fn append(&self, filename: &'a str) -> Block<'a> {
        let block = Block {
            filename,
            num: self.length(filename),
        };

        let mut file = File::options().write(true).open(block.filename).unwrap();
        let offset = (block.num * BLOCK_SIZE) as u64;
        file.seek(SeekFrom::Start(offset)).unwrap();

        let bytes = Vec::with_capacity(BLOCK_SIZE).into_boxed_slice();
        file.write(&bytes).unwrap();
        file.sync_all().unwrap();

        block
    }

    /// Return the number of blocks that comprise a file.
    pub fn length(&self, filename: &str) -> usize {
        let file = File::open(filename).unwrap();
        file.metadata().unwrap().len() as usize / self.block_size
    }
}
