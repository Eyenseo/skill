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

/// Struct that contains information about a Block of a Skill binary file
#[derive(Default, Debug, Clone)]
pub(crate) struct Block {
    pub(crate) block: usize,
    pub(crate) bpo: usize,
    pub(crate) static_count: usize,
    pub(crate) dynamic_count: usize,
}

/// Struct that contains information about a chunk of data that appears in
/// the same block as the field definition
#[derive(Default, Debug, Clone)]
pub(crate) struct DeclarationFieldChunk {
    pub(crate) begin: usize,
    pub(crate) end: usize,
    pub(crate) count: usize,
    pub(crate) appearance: usize,
}

/// Struct that contains information about a chunk of data that appears in
/// another, later block as the field definition does
#[derive(Default, Debug, Clone)]
pub(crate) struct ContinuationFieldChunk {
    pub(crate) begin: usize,
    pub(crate) end: usize,
    pub(crate) count: usize,
    pub(crate) bpo: usize,
}

/// Enum that contains field chunk information
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
    /// Deserialize the field data at "opening" time.
    /// This function should implement the deserialization logic for all non
    /// foreign types
    ///
    /// see deserialize()
    fn read(
        &self,
        file_reader: &Vec<FileReader>,
        string_pool: &StringBlock,
        blocks: &Vec<Block>,
        type_pools: &Vec<Rc<RefCell<PoolProxy>>>,
        instances: &[Ptr<SkillObject>],
    ) -> Result<(), SkillFail>;

    /// deserialize the field data at "writing" time
    /// This function should implement the deserialization logic for all
    /// foreign types
    ///
    /// NOTE if full reflection has to be implemented this behaviour has
    /// to change as deserialization has to happen right before the first
    /// access
    ///
    /// see read()
    fn deserialize(
        &mut self,
        block_reader: &Vec<FileReader>,
        string_pool: &StringBlock,
        blocks: &Vec<Block>,
        type_pools: &Vec<Rc<RefCell<PoolProxy>>>,
        instances: &[Ptr<SkillObject>],
    ) -> Result<(), SkillFail>;

    /// name of the field
    fn name(&self) -> &Rc<SkillString>;
    /// id of the field -- this is the index into the Pool vector this field
    /// is stored in
    fn field_id(&self) -> usize;

    /// Adds another field chunk to be deserialized
    fn add_chunk(&mut self, chunk: FieldChunk);
    /// Compresses saved chunks into one contiguous one
    fn compress_chunks(&mut self, total_count: usize);

    /// Calculates the offset / length for the field data of all instances
    /// this declaration manages
    fn offset(&self, iter: dynamic_instances::Iter) -> Result<usize, SkillFail>;

    /// Writes the metadata part of this declaration
    fn write_meta(
        &mut self,
        writer: &mut FileWriter,
        iter: dynamic_instances::Iter,
        offset: usize,
    ) -> Result<usize, SkillFail>;
    /// Writes the field data of all instances managed by this declaration
    fn write_data(
        &self,
        writer: &mut FileWriter,
        iter: dynamic_instances::Iter,
    ) -> Result<(), SkillFail>;
}
