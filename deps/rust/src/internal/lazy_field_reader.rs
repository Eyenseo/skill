use common::internal::InstancePool;
use common::internal::SkillObject;
use common::io::{Block, FieldChunk, FieldReader, FileReader};
use common::Ptr;
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

impl FieldReader for LazyFieldReader {
    fn read(
        &self,
        file_reader: &Vec<FileReader>,
        string_block: &StringBlock,
        blocks: &Vec<Block>,
        type_pools: &Vec<Rc<RefCell<InstancePool>>>,
        instances: &[Ptr<SkillObject>],
    ) -> Result<(), SkillError> {
        // FIXME
        Ok(())
    }
    fn add_chunk(&mut self, chunk: FieldChunk) {
        self.chunks.push(chunk);
    }
    fn name_id(&self) -> usize {
        self.name_id
    }
}
