use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};

#[derive(Debug)]
struct BlockId<'a> {
    file_name: &'a str,
    block_num: usize,
}

struct Page<'a> {
    buffer: &'a mut [u8],
}

#[derive(Debug)]
struct FileMgr<'a> {
    // dir: DirEntry,
    block_size: usize,
    open_files: HashMap<&'a str, File>,
}

impl FileMgr<'_> {
    fn new(dir_name: &str, block_size: usize) -> Self {
        Self {
            // dir:
            block_size,
            open_files: HashMap::new(),
        }
    }

    fn read(&self, block: &BlockId, page: &mut Page) {
        let mut file = File::open(block.file_name).unwrap();
        let offset = (block.block_num * self.block_size) as u64;
        file.seek(SeekFrom::Start(offset)).unwrap();
        file.read(&mut page.buffer).unwrap();
    }

    fn write(&self, block: &BlockId, page: &Page) {
        let mut file = File::options().write(true).open(block.file_name).unwrap();
        let offset = (block.block_num * self.block_size) as u64;
        file.seek(SeekFrom::Start(offset)).unwrap();
        file.write(&page.buffer).unwrap();
    }
}
