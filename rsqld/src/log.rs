use crate::file::{Block, FileMgr, Page};

pub struct LogMgr<'a> {
    file_mgr: &'a FileMgr,
    log_file_name: &'a str,
    log_page: Page,
    current_block: Block,
}

impl<'a> LogMgr<'a> {
    pub fn new(file_mgr: &'a FileMgr, log_file_name: &'a str) -> Self {
        let log_size = file_mgr.length(log_file_name);
        let current_block = if log_size == 0 {
            // TODO:
            Block
        } else {
            Block {
                filename: log_file_name,
                block_num: log_size - 1,
            }
        };
        Self {
            file_mgr,
            log_file_name,
            log_page: Page::new(file_mgr.block_size),
            current_block,
        }
    }

    fn append_new_block(&self) -> Block<'a> {
        let block = self.file_mgr.append(self.log_file_name);
        // self.log_page.
        self.file_mgr.write(&block, &self.log_page);
        return block;
    }
}

// private BlockId appendNewBlock() {
//    BlockId blk = fm.append(logfile);
//    logpage.setInt(0, fm.blockSize());
//    fm.write(blk, logpage);
//    return blk;
// }
