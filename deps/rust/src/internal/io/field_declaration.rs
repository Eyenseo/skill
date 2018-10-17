use common::error::*;
use common::internal::io::*;
use common::internal::*;
use common::iterator::dynamic_instances;
use common::*;

use std::cell::RefCell;
use std::ops::{Add, AddAssign, Sub, SubAssign};
use std::rc::Rc;

/// Contains information about a Block of a Skill binary file
#[derive(Default, Debug, Clone)]
pub(crate) struct Block {
    pub(crate) block: usize,
    pub(crate) bpo: usize,
    pub(crate) static_count: usize,
    pub(crate) dynamic_count: usize,
}

/// Contains information about a chunk of data that appears in
/// the same block as the field definition
#[derive(Default, Debug, Clone)]
pub(crate) struct DeclarationFieldChunk {
    pub(crate) begin: usize,
    pub(crate) end: usize,
    pub(crate) count: usize,
    pub(crate) appearance: usize,
}

/// Contains information about a chunk of data that appears in
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

pub(crate) trait FieldIO {
    /// Deserialize the field data at "opening" time.
    /// This function should implement the deserialization logic for all non
    /// foreign types
    ///
    /// # Arguments
    /// * `block_reader` - vector where each element is a reader for a specific block
    /// * `string_pool` - to "read" strings
    /// * `blocks` - in the binary file
    /// * `type_pools` - all type pools -- usd to read a specific instance
    /// * `instances` - instances that have this field
    ///
    /// see force_read()
    fn lazy_read(
        &self,
        block_reader: &Vec<FileReader>,
        string_pool: &StringBlock,
        blocks: &Vec<Block>,
        type_pools: &Vec<Rc<RefCell<PoolProxy>>>,
        instances: &[Ptr<SkillObject>],
    ) -> Result<(), SkillFail>;

    /// deserialize the field data at "writing" time
    /// This function should implement the deserialization logic for all
    /// foreign types
    ///
    /// # NOTE
    /// If full reflection has to be implemented this behaviour has
    /// to change as deserialization has to happen right before the first
    /// access
    ///
    /// # Arguments
    /// * `block_reader` - vector where each element is a reader for a specific block
    /// * `string_pool` - to "read" strings
    /// * `blocks` - in the binary file
    /// * `type_pools` - all type pools -- usd to read a specific instance
    /// * `instances` - instances that have this field
    ///
    /// see lazy_read()
    fn force_read(
        &mut self,
        block_reader: &Vec<FileReader>,
        string_pool: &StringBlock,
        blocks: &Vec<Block>,
        type_pools: &Vec<Rc<RefCell<PoolProxy>>>,
        instances: &[Ptr<SkillObject>],
    ) -> Result<(), SkillFail>;

    /// # Returns
    /// name of the field
    fn name(&self) -> &Rc<SkillString>;
    /// # Returns
    /// id of the field -- this is the index into the Pool vector this field
    /// is stored in
    fn field_id(&self) -> usize;
    /// # Returns
    /// type information of the field
    fn field_type(&self) -> &FieldType;

    /// # Arguments
    /// * `chunk` - to be deserialized
    fn add_chunk(&mut self, chunk: FieldChunk);
    /// Compresses saved chunks into one contiguous one
    ///
    /// # Arguments
    /// * `total_count` - total count of instances
    fn compress_chunks(&mut self, total_count: usize);

    /// Calculates the offset / length for the field data of all instances
    /// this declaration manages
    ///
    /// # Arguments
    /// * `iter` - iterator to traverse all instances. Can't be created by this type as a wrapper is needed
    ///
    /// # Returns
    /// Size needed to serialize this field for all instances
    fn offset(&self, iter: dynamic_instances::Iter) -> Result<usize, SkillFail>;

    /// Writes the metadata part of this declaration
    ///
    /// # Arguments
    /// * `writer` - used for writing
    /// * `iter` - iterator to traverse all instances. Can't be created by this type as a wrapper is needed
    /// * `offset` - current data section offset
    ///
    /// # Returns
    /// `offset` + the needed size to serialize this field for all instances
    fn write_meta(
        &mut self,
        writer: &mut FileWriter,
        iter: dynamic_instances::Iter,
        offset: usize,
    ) -> Result<usize, SkillFail>;
    /// Writes the field data of all instances managed by this declaration
    ///
    /// # Arguments
    /// * `writer` - used for writing
    /// * `iter` - iterator to traverse all instances. Can't be created by this type as a wrapper is needed
    fn write_data(
        &self,
        writer: &mut FileWriter,
        iter: dynamic_instances::Iter,
    ) -> Result<(), SkillFail>;
}

/// Bulls*** struct that is used to hide trait functions ...
///
/// Attributes can't be extracted because of borrowing rules
pub struct FieldDeclaration {
    pub(crate) io: Box<FieldIO>,
}

impl FieldDeclaration {
    pub(crate) fn new(io: Box<FieldIO>) -> FieldDeclaration {
        FieldDeclaration { io }
    }

    /// # Returns
    /// name of the field
    pub fn name(&self) -> &Rc<SkillString> {
        self.io.name()
    }
}
