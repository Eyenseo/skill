use common::error::*;
use common::internal::InstancePool;
use common::internal::SkillObject;
use common::io::{
    Block, BlockIndex, DeclarationFieldChunk, FieldChunk, FieldDeclaration, FieldType, FileReader,
    FileWriter,
};
use common::iterator::static_data;
use common::Ptr;
use common::SkillString;
use common::StringBlock;

use std::cell::RefCell;
use std::rc::Rc;

pub struct LazyFieldDeclaration {
    name: Rc<SkillString>,
    index: usize,
    chunks: Vec<FieldChunk>,
    field_type: FieldType,
}

impl LazyFieldDeclaration {
    pub fn new(name: Rc<SkillString>, index: usize, field_type: FieldType) -> LazyFieldDeclaration {
        LazyFieldDeclaration {
            name,
            index,
            chunks: Vec::new(),
            field_type,
        }
    }
}

impl FieldDeclaration for LazyFieldDeclaration {
    fn read(
        &self,
        file_reader: &Vec<FileReader>,
        string_block: &StringBlock,
        blocks: &Vec<Block>,
        type_pools: &Vec<Rc<RefCell<InstancePool>>>,
        instances: &[Ptr<SkillObject>],
    ) -> Result<(), SkillFail> {
        // FIXME
        Ok(())
    }
    fn add_chunk(&mut self, chunk: FieldChunk) {
        self.chunks.push(chunk);
    }
    fn name(&self) -> &Rc<SkillString> {
        &self.name
    }
    fn index(&self) -> usize {
        self.index
    }
    fn compress_chunks(&mut self, total_count: usize) {
        self.chunks = Vec::with_capacity(1);
        self.chunks
            .push(FieldChunk::Declaration(DeclarationFieldChunk {
                begin: 0,
                end: 0,
                count: total_count,
                appearance: BlockIndex::from(1),
            }));
    }
    fn offset(&self, iter: static_data::Iter) -> usize {
        unimplemented!();
    }
    fn write_meta(
        &mut self,
        writer: &mut FileWriter,
        iter: static_data::Iter,
        offset: usize,
    ) -> Result<usize, SkillFail> {
        writer.write_v64(self.index as i64)?;
        writer.write_v64(self.name.get_skill_id() as i64)?;
        writer.write_field_type(&self.field_type)?;
        writer.write_i8(0)?; // TODO write restrictions
        let end_offset = offset + self.offset(iter);
        writer.write_v64(end_offset as i64)?;

        match self.chunks.first_mut().unwrap() {
            FieldChunk::Declaration(ref mut dec) => {
                dec.begin = offset;
                dec.end = end_offset;
                Ok(())
            }
            _ => Err(SkillFail::internal(InternalFail::BadChunk)),
        }?;

        Ok(end_offset)
    }
    fn write_data(
        &self,
        writer: &mut FileWriter,
        iter: static_data::Iter,
    ) -> Result<(), SkillFail> {
        unimplemented!();
    }
}
