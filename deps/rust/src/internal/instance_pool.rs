use common::internal::{ObjectReader, SkillObject};
use common::io::{Block, FieldChunk, FieldReader, FieldType, FileReader};
use common::Ptr;
use common::SkillError;
use common::StringBlock;

use std::cell::RefCell;
use std::rc::Rc;

// TODO rename
pub trait InstancePool {
    fn has_field(&self, name_id: usize) -> bool;
    fn add_chunk_to(&mut self, name_id: usize, chunk: FieldChunk);
    fn add_field(
        &mut self,
        name_id: usize,
        field_name: &str,
        field_type: FieldType,
        chunk: FieldChunk,
    );

    fn field_amount(&self) -> usize;

    fn add_block(&mut self, block: Block);
    fn blocks(&mut self) -> &mut Vec<Block>;

    fn set_super(&mut self, pool: Rc<RefCell<InstancePool>>);
    fn get_super(&self) -> Option<Rc<RefCell<InstancePool>>>;

    fn get_base(&self) -> Option<Rc<RefCell<InstancePool>>>;
    fn is_base(&self) -> bool;

    fn set_type_id(&mut self, id: usize);
    fn get_type_id(&self) -> usize;

    fn set_type_name_index(&mut self, id: usize);
    fn get_type_name_index(&self) -> usize;

    fn get_local_static_count(&self) -> usize;
    fn set_local_static_count(&mut self, count: usize);

    fn get_local_dynamic_count(&self) -> usize;
    fn get_local_bpo(&self) -> usize;

    fn get_global_static_count(&self) -> usize;
    fn set_global_static_count(&mut self, count: usize);

    fn set_invariant(&mut self, invariant: bool);
    fn get_global_cached_count(&self) -> usize;
    fn set_global_cached_count(&mut self, count: usize);

    fn size(&self) -> usize;

    fn get_base_vec(&self) -> Rc<RefCell<Vec<Ptr<SkillObject>>>>;
    fn read_object(&self, index: usize) -> Result<Ptr<SkillObject>, SkillError>;

    fn add_sub(&mut self, pool: Rc<RefCell<InstancePool>>);

    fn allocate(&mut self, type_pools: &Vec<Rc<RefCell<InstancePool>>>);
    fn initialize(
        &self,
        file_reader: &Vec<FileReader>,
        string_block: &StringBlock,
        type_pools: &Vec<Rc<RefCell<InstancePool>>>,
    ) -> Result<(), SkillError>;

    fn make_instance(&self) -> Ptr<SkillObject>;
}
