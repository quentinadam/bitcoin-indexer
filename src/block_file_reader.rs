use crate::{iterate_transactions, BlockHeader, BlockTrait, HashMap, HashingBufferReader, Logger, ThreadPool, Transaction, TryInto};
use regex::Regex;
use std::{
    fs::{read_dir, File},
    io::{prelude::*, SeekFrom},
    path::Path,
};

#[derive(Debug, Clone)]
pub struct FileBlock {
    file_path: String,
    offset: u64,
    length: usize,
    header: BlockHeader,
    height: usize,
}

impl BlockTrait for FileBlock {
    fn header(&self) -> &BlockHeader {
        &self.header
    }

    fn height(&self) -> usize {
        self.height
    }

    fn transactions<F: FnMut(&Transaction)>(&self, callback: &mut F) -> () {
        let mut file = File::open(&self.file_path).unwrap();
        file.seek(SeekFrom::Start(self.offset)).unwrap();
        let mut buffer = vec![0; self.length];
        file.read_exact(&mut buffer).unwrap();
        iterate_transactions(&buffer, &mut |transaction| {
            callback(&transaction);
        });
    }
}

#[derive(Debug, Clone)]
pub struct BlockFileReader {
    path: String,
}

impl BlockFileReader {
    pub fn new(path: &str) -> Self {
        Self { path: path.to_string() }
    }

    fn file_names(&self) -> Vec<String> {
        let regex = Regex::new(r"^blk[0-9]{5}\.dat$").unwrap();
        let mut file_names: Vec<_> = read_dir(&self.path)
            .expect(&format!("Could not list files of directory {}", &self.path))
            .map(|entry| entry.unwrap().file_name().into_string().unwrap())
            .filter(|file_name| regex.is_match(file_name))
            .collect();
        file_names.sort();
        file_names
    }

    fn find_longest_chain(&self, blocks: Vec<FileBlock>, logger: &Logger) -> Vec<FileBlock> {
        logger.log("finding longest block chain...");
        let mut hashmap = HashMap::with_capacity(blocks.len());
        let mut reverse_hashmap = HashMap::with_capacity(blocks.len());
        for block in blocks {
            match reverse_hashmap.get_mut(&block.previous_block_hash()) {
                None => {
                    reverse_hashmap.insert(block.previous_block_hash(), vec![block.hash()]);
                }
                Some(blocks) => {
                    blocks.push(block.hash());
                }
            }
            hashmap.insert(block.hash(), block);
        }
        let mut height = 0;
        let mut heads = vec![[0u8; 32]];
        loop {
            let mut new_heads = Vec::new();
            for head in &heads {
                if let Some(blocks) = reverse_hashmap.get(head) {
                    for block in blocks {
                        new_heads.push(*block);
                    }
                }
            }
            if new_heads.len() > 0 {
                height += 1;
                heads = new_heads;
            } else {
                break;
            }
        }
        let mut current = heads[0];
        let mut blocks = Vec::new();
        while current != [0; 32] {
            let mut block = hashmap.remove(&current).unwrap();
            block.height = height - 1;
            current = block.previous_block_hash();
            blocks.push(block);
            height -= 1;
        }
        blocks.reverse();
        logger.log("finding longest block chain done!");
        blocks
    }

    fn scan_block_file(&self, file_name: &str, logger: &Logger) -> Vec<FileBlock> {
        logger.log(format!("scanning {}...", file_name));
        let file_path = Path::new(&self.path).join(file_name).to_str().unwrap().to_string();
        let mut file = File::open(&file_path).unwrap();
        let mut buffer = [0u8; 88];
        let mut offset = 0;
        let mut blocks = Vec::new();
        loop {
            let bytes = file.read(&mut buffer).unwrap();
            if bytes == 0 {
                break;
            }
            assert!(bytes == 88);
            let mut reader = HashingBufferReader::new(&buffer);
            let magic = reader.read_u32_le(&mut None);
            if magic == 0 {
                break;
            }
            assert!(magic == 3652501241);
            offset += 8;
            let length: usize = reader.read_u32_le(&mut None).try_into().unwrap();
            let header = BlockHeader::from_buffer(&buffer[8..]);
            blocks.push(FileBlock {
                file_path: file_path.clone(),
                offset,
                length,
                header,
                height: 0,
            });
            let length: u64 = length.try_into().unwrap();
            offset += length;
            file.seek(SeekFrom::Start(offset)).unwrap();
        }
        blocks
    }

    fn scan_block_files(&self, threads: usize, logger: &Logger) -> Vec<FileBlock> {
        logger.log("scanning block files...");
        let file_names = self.file_names().into_iter();
        let blocks = if threads > 1 {
            let this = self.clone();
            let logger = logger.clone();
            let thread_pool = ThreadPool::new(
                threads,
                move |file_name: String| this.scan_block_file(&file_name, &logger),
                file_names,
            );
            thread_pool.flatten().collect()
        } else {
            file_names
                .map(|file_name| self.scan_block_file(&file_name, logger))
                .flatten()
                .collect()
        };
        logger.log("scanning block files done!");
        blocks
    }

    pub fn blocks(&self, threads: usize, logger: &Logger) -> Vec<FileBlock> {
        let blocks = self.scan_block_files(threads, logger);
        self.find_longest_chain(blocks, &logger)
    }
}
