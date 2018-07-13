use common::internal::{InstancePool, LazyFieldReader, ObjectReader, SkillObject, UndefinedObject};
use common::io::{Block, FieldChunk, FieldReader, FieldType, FileReader};
use common::Ptr;
use common::SkillError;
use common::StringBlock;

use std::cell::RefCell;
use std::rc::Rc;

pub struct UndefinedPool {
    string_block: Rc<RefCell<StringBlock>>,
    instances: Vec<Rc<RefCell<UndefinedObject>>>,
    fields: Vec<Box<LazyFieldReader>>,
    type_name_index: usize,
    type_id: usize,
    blocks: Vec<Block>,
    super_pool: Option<Rc<RefCell<InstancePool>>>,
    base_pool: Option<Rc<RefCell<InstancePool>>>,
    static_count: usize,
    dynamic_count: usize,
}

impl UndefinedPool {
    pub fn new(string_block: Rc<RefCell<StringBlock>>) -> UndefinedPool {
        UndefinedPool {
            string_block,
            instances: Vec::new(),
            fields: Vec::new(),
            type_name_index: 0,
            type_id: 0,
            blocks: Vec::new(),
            super_pool: None,
            base_pool: None,
            static_count: 0,
            dynamic_count: 0,
        }
    }
}

impl InstancePool for UndefinedPool {
    fn add_field(
        &mut self,
        name_id: usize,
        field_name: &str,
        field_type: FieldType,
        chunk: FieldChunk,
    ) {
        let mut reader = Box::new(LazyFieldReader::new(name_id));
        (reader.as_mut() as &mut FieldReader<i8>).add_chunk(chunk);
        self.fields.push(reader);
    }
    fn has_field(&self, name_id: usize) -> bool {
        for f in &self.fields {
            let f = f.as_ref() as &FieldReader<i8>;
            if f.name_id() == name_id {
                return true;
            }
        }
        false
    }
    fn field_amount(&self) -> usize {
        self.fields.len()
    }
    fn add_chunk_to(&mut self, name_id: usize, chunk: FieldChunk) {
        for f in &mut self.fields {
            let f = f.as_mut() as &mut FieldReader<i8>;
            if f.name_id() == name_id {
                f.add_chunk(chunk);
                return;
            }
        }
        panic!("No field of id:{}", name_id);
    }

    fn set_type_id(&mut self, id: usize) {
        self.type_id = id;
    }
    fn get_type_id(&self) -> usize {
        self.type_id
    }

    fn set_type_name_index(&mut self, id: usize) {
        self.type_name_index = id;
    }
    fn get_type_name_index(&self) -> usize {
        self.type_name_index
    }

    fn read_object(&self, _index: usize) -> Result<Ptr<SkillObject>, SkillError> {
        unimplemented!();
    }

    fn add_block(&mut self, block: Block) {
        self.blocks.push(block);
    }
    fn blocks(&mut self, block: Block) -> &mut Vec<Block> {
        &mut self.blocks
    }

    fn set_super(&mut self, pool: Rc<RefCell<InstancePool>>) {
        if pool.borrow().is_base() {
            self.base_pool = Some(pool.clone());
        } else {
            self.base_pool = pool.borrow().get_base(); // TODO check?
        }
        self.super_pool = Some(pool);
    }
    fn get_super(&self) -> Option<Rc<RefCell<InstancePool>>> {
        self.super_pool.clone()
    }

    fn get_base(&self) -> Option<Rc<RefCell<InstancePool>>> {
        self.base_pool.clone()
    }
    fn is_base(&self) -> bool {
        self.super_pool.is_none()
    }

    fn get_local_static_count(&self) -> usize {
        if let Some(block) = self.blocks.last() {
            return block.static_count;
        }
        panic!();
    }
    fn set_local_static_count(&mut self, count: usize) {
        if let Some(block) = self.blocks.last_mut() {
            block.static_count = count
        } else {
            panic!();
        }
    }

    fn get_local_dynamic_count(&self) -> usize {
        if let Some(block) = self.blocks.last() {
            return block.dynamic_count;
        }
        panic!();
    }

    fn get_local_bpo(&self) -> usize {
        if let Some(block) = self.blocks.last() {
            return block.bpo;
        }
        panic!();
    }

    fn get_global_static_count(&self) -> usize {
        self.static_count
    }
    fn set_global_static_count(&mut self, count: usize) {
        self.static_count = count;
    }

    fn get_global_cached_count(&self) -> usize {
        self.dynamic_count
    }
    fn set_global_cached_count(&mut self, count: usize) {
        self.dynamic_count = count;
    }

    fn make_state(
        &mut self,
        file_reader: &Vec<FileReader>,
        string_block: &StringBlock,
        type_pools: &Vec<Rc<RefCell<InstancePool>>>,
    ) -> Result<(), SkillError> {
        for mut f in &mut self.fields {
            // f.read(&mut self.instances)?;
        }
        Ok(())
    }

    fn initialize(&mut self) {
        if self.is_base() {
            self.instances.reserve(self.dynamic_count); // FIXME check if dynamic count is the correct one
        }

        for _ in 0..self.static_count {
            self.instances
                .push(Rc::new(RefCell::new(UndefinedObject::new())));
        }
    }
}
