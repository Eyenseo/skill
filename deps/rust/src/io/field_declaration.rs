// TODO rename
use common::error::*;
use common::internal::InstancePool;
use common::internal::SkillObject;
use common::internal::StringBlock;
use common::io::FileReader;
use common::io::FileWriter;
use common::iterator::dynamic_data;
use common::Ptr;
use common::SkillString;

use std::cell::RefCell;
use std::ops::{Add, AddAssign, Sub, SubAssign};
use std::rc::Rc;

#[derive(Default, Debug, Clone, Copy)]
pub struct BlockIndex {
    pub block: usize,
}
impl From<usize> for BlockIndex {
    fn from(val: usize) -> BlockIndex {
        BlockIndex { block: val }
    }
}
impl Add<usize> for BlockIndex {
    type Output = Self;
    fn add(mut self, other: usize) -> Self::Output {
        self.block += other;
        self
    }
}
impl AddAssign<usize> for BlockIndex {
    fn add_assign(&mut self, other: usize) {
        self.block += other;
    }
}
impl Sub<usize> for BlockIndex {
    type Output = Self;
    fn sub(mut self, other: usize) -> Self::Output {
        self.block -= other;
        self
    }
}
impl SubAssign<usize> for BlockIndex {
    fn sub_assign(&mut self, other: usize) {
        self.block -= other;
    }
}
impl AddAssign<BlockIndex> for BlockIndex {
    fn add_assign(&mut self, other: BlockIndex) {
        self.block += other.block;
    }
}

#[derive(Default, Debug, Clone)]
pub struct Block {
    pub block: BlockIndex,
    pub bpo: usize,           // TODO Strongly type
    pub static_count: usize,  // TODO rename
    pub dynamic_count: usize, // TODO rename
}

#[derive(Default, Debug, Clone)]
pub struct DeclarationFieldChunk {
    pub begin: usize,
    pub end: usize,
    pub count: usize,
    pub appearance: BlockIndex,
}

#[derive(Default, Debug, Clone)]
pub struct ContinuationFieldChunk {
    pub begin: usize,
    pub end: usize,
    pub count: usize,
    pub bpo: usize, // TODO strongly type
}

#[derive(Debug, Clone)]
pub enum FieldChunk {
    Declaration(DeclarationFieldChunk),
    Continuation(ContinuationFieldChunk),
}

impl From<ContinuationFieldChunk> for FieldChunk {
    fn from(val: ContinuationFieldChunk) -> FieldChunk {
        FieldChunk::Continuation(val)
    }
}
impl From<DeclarationFieldChunk> for FieldChunk {
    fn from(val: DeclarationFieldChunk) -> FieldChunk {
        FieldChunk::Declaration(val)
    }
}

pub trait FieldDeclaration {
    fn read(
        &self,
        file_reader: &Vec<FileReader>,
        string_pool: &StringBlock,
        blocks: &Vec<Block>,
        type_pools: &Vec<Rc<RefCell<InstancePool>>>,
        instances: &[Ptr<SkillObject>],
    ) -> Result<(), SkillFail>;
    fn deserialize(
        &mut self,
        block_reader: &Vec<FileReader>,
        string_pool: &StringBlock,
        blocks: &Vec<Block>,
        type_pools: &Vec<Rc<RefCell<InstancePool>>>,
        instances: &[Ptr<SkillObject>],
    ) -> Result<(), SkillFail>;

    fn name(&self) -> &Rc<SkillString>;
    fn field_id(&self) -> usize;

    fn add_chunk(&mut self, chunk: FieldChunk);

    fn compress_chunks(&mut self, total_count: usize);
    fn offset(&self, iter: dynamic_data::Iter) -> Result<usize, SkillFail>;

    /// This call will also update the offsets of the chunk
    fn write_meta(
        &mut self,
        writer: &mut FileWriter,
        iter: dynamic_data::Iter,
        offset: usize,
    ) -> Result<usize, SkillFail>;
    fn write_data(
        &self,
        writer: &mut FileWriter,
        iter: dynamic_data::Iter,
    ) -> Result<(), SkillFail>;
}
