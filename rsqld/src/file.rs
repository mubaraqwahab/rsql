use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

#[derive(Debug, PartialEq, Eq, Hash)]
struct BlockId<'a> {
    filename: &'a str,
    block_num: usize,
}

struct Page {
    buffer: Box<[u8]>,
}

impl Page {
    fn new(size: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(size).into_boxed_slice(),
        }
    }
}

impl From<&[u8]> for Page {
    fn from(value: &[u8]) -> Self {
        Self {
            buffer: Box::from(value),
        }
    }
}

#[derive(Debug)]
struct FileMgr<'a> {
    dir: &'a Path,
    block_size: usize,
    open_files: HashMap<&'a str, File>,
}

impl<'a> FileMgr<'a> {
    fn new(dir: &'a Path, block_size: usize) -> Self {
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
            block_size,
            open_files: HashMap::new(),
        }
    }

    // TODO: make this "synchronized" a la java
    fn read(&self, block: &BlockId, page: &mut Page) {
        let mut file = File::open(block.filename).unwrap();

        let offset = (block.block_num * self.block_size) as u64;
        file.seek(SeekFrom::Start(offset)).unwrap();

        file.read(&mut page.buffer).unwrap();
    }

    // TODO: make this "synchronized" a la java
    fn write(&self, block: &BlockId, page: &Page) {
        let mut file = File::options().write(true).open(block.filename).unwrap();

        let offset = (block.block_num * self.block_size) as u64;
        file.seek(SeekFrom::Start(offset)).unwrap();

        file.write(&page.buffer).unwrap();
        file.sync_all().unwrap();
    }

    // TODO: make this "synchronized" a la java
    fn append(&self, filename: &'a str) -> BlockId<'a> {
        let block = BlockId {
            filename,
            // TODO: I don't get this
            block_num: self.length(filename),
        };

        let mut file = File::options().write(true).open(block.filename).unwrap();
        let offset = (block.block_num * self.block_size) as u64;
        file.seek(SeekFrom::Start(offset)).unwrap();

        let bytes = Vec::with_capacity(self.block_size).into_boxed_slice();
        file.write(&bytes).unwrap();
        file.sync_all().unwrap();

        block
    }

    fn length(&self, filename: &str) -> usize {
        let file = File::open(filename).unwrap();
        file.metadata().unwrap().len() as usize / self.block_size
    }
}
