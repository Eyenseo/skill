use common::internal::InstancePool;
use common::io::{Block, FieldChunk, FieldReader, FileReader};
use common::SkillError;
use common::StringBlock;

use std::cell::RefCell;
use std::rc::Rc;

pub struct LazyFieldReader {
    name_id: usize,
    chunks: Vec<FieldChunk>,
}

impl LazyFieldReader {
    pub fn new(name_id: usize) -> LazyFieldReader {
        LazyFieldReader {
            name_id,
            chunks: Vec::new(),
        }
    }
}

impl<T> FieldReader<T> for LazyFieldReader {
    fn read(
        &mut self,
        file_reader: &Vec<FileReader>,
        string_block: &StringBlock,
        blocks: &Vec<Block>,
        type_pools: &Vec<Rc<RefCell<InstancePool>>>,
        instances: &mut Vec<T>,
    ) -> Result<(), SkillError> {
        unimplemented!();
        // TODO - do more?
        // Ok(())
    }
    fn add_chunk(&mut self, chunk: FieldChunk) {
        self.chunks.push(chunk);
    }
    fn name_id(&self) -> usize {
        self.name_id
    }
}
