use common::error::*;
use common::internal::{ObjectReader, SkillObject};
use common::io::{Block, FieldChunk, FieldDeclaration, FieldType, FileReader, FileWriter};
use common::iterator::dynamic_data;
use common::Ptr;
use common::SkillString;
use common::StringBlock;
use SkillFile;

use std::cell::RefCell;
use std::rc::Rc;

// TODO rename
// TODO reorder
pub trait InstancePool {
    fn has_field(&self, name_id: usize) -> bool;
    fn add_chunk_to(&mut self, name_id: usize, chunk: FieldChunk) -> Result<(), SkillFail>;
    fn add_field(
        &mut self,
        field_id: usize,
        field_name: Rc<SkillString>,
        field_type: FieldType,
        chunk: FieldChunk,
    ) -> Result<(), SkillFail>;

    fn field_amount(&self) -> usize;

    fn add_block(&mut self, block: Block);
    fn blocks_mut(&mut self) -> &mut Vec<Block>;
    fn blocks(&self) -> &Vec<Block>;

    fn set_super(&mut self, pool: Rc<RefCell<InstancePool>>);
    fn get_super(&self) -> Option<Rc<RefCell<InstancePool>>>;

    fn get_base(&self) -> Option<Rc<RefCell<InstancePool>>>;
    fn is_base(&self) -> bool;

    fn set_type_id(&mut self, id: usize);
    fn get_type_id(&self) -> usize;

    fn name(&self) -> &Rc<SkillString>;

    fn get_local_static_count(&self) -> usize;
    fn set_local_static_count(&mut self, count: usize);

    fn get_local_dynamic_count(&self) -> usize;
    fn get_local_bpo(&self) -> usize;

    fn get_global_static_count(&self) -> usize;
    fn set_global_static_count(&mut self, count: usize);

    fn set_invariant(&mut self, invariant: bool);

    fn get_global_cached_count(&self) -> usize;
    fn set_global_cached_count(&mut self, count: usize);

    fn get_base_vec(&self) -> Rc<RefCell<Vec<Ptr<SkillObject>>>>;
    fn read_object(&self, index: usize) -> Result<Ptr<SkillObject>, SkillFail>;

    fn add_sub(&mut self, pool: Rc<RefCell<InstancePool>>);

    fn allocate(&mut self);
    fn initialize(
        &self,
        block_reader: &Vec<FileReader>,
        string_block: &StringBlock,
        type_pools: &Vec<Rc<RefCell<InstancePool>>>,
    ) -> Result<(), SkillFail>;
    fn deserialize(&self, skill_file: &SkillFile) -> Result<(), SkillFail>;

    fn make_foreign(&self, skill_id: usize, skill_type_id: usize) -> Ptr<SkillObject>;

    fn set_next_pool(&mut self, pool: Option<Rc<RefCell<InstancePool>>>);
    fn get_next_pool(&self) -> Option<Rc<RefCell<InstancePool>>>;
    fn type_hierarchy_height(&self) -> usize;

    /// Returns the raw vector that stores the newly created instances of type T
    ///
    /// # Important
    /// There can be deleted instances in the vector, `len()` is not the amount of valid instances
    fn new_instances(&self) -> &Vec<Ptr<SkillObject>>;
    /// Returns the vector that stores the deserialized instances of the type hierarchy - the base
    /// vector
    ///
    /// # Important
    /// There can be deleted instances in the vector, `len()` is not the amount of valid instances
    fn static_instances(&self) -> &Vec<Ptr<SkillObject>>;

    /// Is the value of new_objects + static_objects = All instances of type T - also the ones
    /// marked for deletion
    fn static_size(&self) -> usize;

    /// Amount of instances of T (static and new) and the number of instances in all subpools
    ///
    /// # NOTE named size in C++
    fn dynamic_size(&self) -> usize;

    // TODO add prune method that removes non-held references
    fn deleted(&self) -> usize;

    fn update_after_compress(
        &mut self,
        local_bpo: &Vec<usize>,
        vec: Rc<RefCell<Vec<Ptr<SkillObject>>>>,
    );
    fn compress_field_chunks(&mut self, local_bpo: &Vec<usize>);

    fn write_type_meta(
        &self,
        writer: &mut FileWriter,
        local_bpos: &Vec<usize>,
    ) -> Result<(), SkillFail>;
    /// * `iter` is needed as self cant create a dynamic_data::Iter instance
    fn write_field_meta(
        &self,
        writer: &mut FileWriter,
        iter: dynamic_data::Iter,
        offset: usize,
    ) -> Result<usize, SkillFail>;
    /// * `iter` is needed as self cant create a dynamic_data::Iter instance
    fn write_field_data(
        &self,
        writer: &mut FileWriter,
        iter: dynamic_data::Iter,
    ) -> Result<(), SkillFail>;
}
