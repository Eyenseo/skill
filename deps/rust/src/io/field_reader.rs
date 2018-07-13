use common::internal::InstancePool;
use common::io::{FileReader, Offset};
use common::SkillError;
use common::StringBlock;

use std::ops::{Add, AddAssign};

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
    pub offset_start: Offset,
    pub offset_end: Offset,
    pub count: usize,
    pub appearance: BlockIndex,
}

#[derive(Default, Debug, Clone)]
pub struct ContinuationFieldChunk {
    pub offset_start: Offset,
    pub offset_end: Offset,
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

pub trait FieldReader<T> {
    fn read(
        &mut self,
        file_reader: &Vec<FileReader>,
        string_block: &StringBlock,
        blocks: &Vec<Block>,
        instances: &mut Vec<T>,
    ) -> Result<(), SkillError>;
    fn add_chunk(&mut self, chunk: FieldChunk);
    fn name_id(&self) -> usize;
}
