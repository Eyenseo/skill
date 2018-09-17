/*
 * @author Roland Jaeger
 */

use common::error::*;
use common::internal::io::*;
use common::internal::*;
use common::iterator::dynamic_instances;
use common::*;

use std::cell::RefCell;
use std::ops::{Add, AddAssign, Sub, SubAssign};
use std::rc::Rc;

#[derive(Default, Debug, Clone, Copy)]
pub(crate) struct BlockIndex {
    pub(crate) block: usize,
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
pub(crate) struct Block {
    pub(crate) block: BlockIndex,
    pub(crate) bpo: usize, // TODO Strongly type
    pub(crate) static_count: usize,
    pub(crate) dynamic_count: usize,
}

#[derive(Default, Debug, Clone)]
pub(crate) struct DeclarationFieldChunk {
    pub(crate) begin: usize,
    pub(crate) end: usize,
    pub(crate) count: usize,
    pub(crate) appearance: BlockIndex,
}

#[derive(Default, Debug, Clone)]
pub(crate) struct ContinuationFieldChunk {
    pub(crate) begin: usize,
    pub(crate) end: usize,
    pub(crate) count: usize,
    pub(crate) bpo: usize, // TODO strongly type
}

#[derive(Debug, Clone)]
pub(crate) enum FieldChunk {
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

pub(crate) trait FieldDeclaration {
    fn read(
        &self,
        file_reader: &Vec<FileReader>,
        string_pool: &StringBlock,
        blocks: &Vec<Block>,
        type_pools: &Vec<Rc<RefCell<PoolProxy>>>,
        instances: &[Ptr<SkillObject>],
    ) -> Result<(), SkillFail>;
    fn deserialize(
        &mut self,
        block_reader: &Vec<FileReader>,
        string_pool: &StringBlock,
        blocks: &Vec<Block>,
        type_pools: &Vec<Rc<RefCell<PoolProxy>>>,
        instances: &[Ptr<SkillObject>],
    ) -> Result<(), SkillFail>;

    fn name(&self) -> &Rc<SkillString>;
    fn field_id(&self) -> usize;

    fn add_chunk(&mut self, chunk: FieldChunk);

    fn compress_chunks(&mut self, total_count: usize);
    fn offset(&self, iter: dynamic_instances::Iter) -> Result<usize, SkillFail>;

    /// This call will also update the offsets of the chunk
    fn write_meta(
        &mut self,
        writer: &mut FileWriter,
        iter: dynamic_instances::Iter,
        offset: usize,
    ) -> Result<usize, SkillFail>;
    fn write_data(
        &self,
        writer: &mut FileWriter,
        iter: dynamic_instances::Iter,
    ) -> Result<(), SkillFail>;
}
