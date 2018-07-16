use common::internal::InstancePool;
use common::internal::SkillObject;
use common::io::{FileReader, Offset};
use common::Ptr;
use common::SkillError;
use common::StringBlock;

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
    pub begin: Offset,
    pub end: Offset,
    pub count: usize,
    pub appearance: BlockIndex,
}

#[derive(Default, Debug, Clone)]
pub struct ContinuationFieldChunk {
    pub begin: Offset,
    pub end: Offset,
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

pub trait FieldReader {
    fn read(
        &self,
        file_reader: &Vec<FileReader>,
        string_block: &StringBlock,
        blocks: &Vec<Block>,
        type_pools: &Vec<Rc<RefCell<InstancePool>>>,
        instances: &[Ptr<SkillObject>],
    ) -> Result<(), SkillError>;
    fn add_chunk(&mut self, chunk: FieldChunk);
    fn name_id(&self) -> usize;
}
