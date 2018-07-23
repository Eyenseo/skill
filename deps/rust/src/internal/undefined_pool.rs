use common::internal::{InstancePool, LazyFieldReader, ObjectReader, SkillObject, UndefinedObject};
use common::io::{Block, FieldChunk, FieldReader, FieldType, FileReader};
use common::Ptr;
use common::SkillError;
use common::StringBlock;

use std::cell::RefCell;
use std::rc::Rc;

pub struct UndefinedPool {
    string_block: Rc<RefCell<StringBlock>>,
    instances: Rc<RefCell<Vec<Ptr<SkillObject>>>>,
    book_static: Vec<Ptr<SkillObject>>,
    book_dynamic: Vec<Ptr<SkillObject>>,
    fields: Vec<Box<LazyFieldReader>>,
    type_name_index: usize,
    type_id: usize,
    blocks: Vec<Block>,
    super_pool: Option<Rc<RefCell<InstancePool>>>,
    sub_pools: Vec<Rc<RefCell<InstancePool>>>,
    base_pool: Option<Rc<RefCell<InstancePool>>>,
    static_count: usize,
    dynamic_count: usize,
    cached_count: usize,
    deleted_count: usize,
    invariant: bool,
}

impl UndefinedPool {
    pub fn new(string_block: Rc<RefCell<StringBlock>>) -> UndefinedPool {
        UndefinedPool {
            string_block,
            instances: Rc::default(),
            book_static: Vec::new(),
            book_dynamic: Vec::new(),
            fields: Vec::new(),
            type_name_index: 0,
            type_id: 0,
            blocks: Vec::new(),
            super_pool: None,
            sub_pools: Vec::new(),
            base_pool: None,
            static_count: 0,
            dynamic_count: 0,
            cached_count: 0,
            deleted_count: 0,
            invariant: false,
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
        reader.as_mut().add_chunk(chunk);
        self.fields.push(reader);
    }
    fn has_field(&self, name_id: usize) -> bool {
        for f in &self.fields {
            let f = f.as_ref();
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
            let f = f.as_mut();
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

    fn add_block(&mut self, block: Block) {
        self.blocks.push(block);
    }
    fn blocks(&mut self) -> &mut Vec<Block> {
        &mut self.blocks
    }

    fn set_super(&mut self, pool: Rc<RefCell<InstancePool>>) {
        if pool.borrow().is_base() {
            self.base_pool = Some(pool.clone());
        } else {
            self.base_pool = pool.borrow().get_base(); // TODO check?
        }
        self.instances = self.base_pool.as_ref().unwrap().borrow().get_base_vec();
        self.super_pool = Some(pool);
    }
    fn get_super(&self) -> Option<Rc<RefCell<InstancePool>>> {
        self.super_pool.clone()
    }

    fn add_sub(&mut self, pool: Rc<RefCell<InstancePool>>) {
        self.sub_pools.push(pool);
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

    fn set_invariant(&mut self, invariant: bool) {
        if self.invariant != invariant {
            self.invariant = invariant;
            if invariant {
                self.cached_count = self.static_count - self.deleted_count;
                for s in self.sub_pools.iter() {
                    let mut s = s.borrow_mut();
                    s.set_invariant(true);
                    self.cached_count += s.get_global_cached_count();
                }
            } else if self.super_pool.is_some() {
                self.super_pool
                    .as_ref()
                    .unwrap()
                    .borrow_mut()
                    .set_invariant(false);
            }
        }
    }

    fn size(&self) -> usize {
        if self.invariant {
            self.cached_count
        } else {
            let mut ret = self.static_count;
            for s in self.sub_pools.iter() {
                ret += s.borrow().size();
            }
            ret
        }
    }

    fn get_global_static_count(&self) -> usize {
        self.static_count
    }
    fn set_global_static_count(&mut self, count: usize) {
        self.static_count = count;
    }

    fn get_global_cached_count(&self) -> usize {
        self.cached_count
    }
    fn set_global_cached_count(&mut self, count: usize) {
        self.cached_count = count;
    }

    fn get_base_vec(&self) -> Rc<RefCell<Vec<Ptr<SkillObject>>>> {
        self.instances.clone()
    }
    fn read_object(&self, index: usize) -> Result<Ptr<SkillObject>, SkillError> {
        assert!(index >= 1);
        info!(
            target:"SkillParsing",
            "read user instance:{} from:{}",
            index,
            self.instances.borrow().len(),
        );
        Ok(self.instances.borrow()[index - 1].clone())
    }

    fn initialize(
        &self,
        file_reader: &Vec<FileReader>,
        string_block: &StringBlock,
        type_pools: &Vec<Rc<RefCell<InstancePool>>>,
    ) -> Result<(), SkillError> {
        for f in self.fields.iter() {
            let instances = self.instances.borrow();
            f.read(
                file_reader,
                string_block,
                &self.blocks,
                type_pools,
                &instances,
            )?;
        }
        Ok(())
    }

    fn allocate(&mut self) {
        let mut vec = self.instances.borrow_mut();
        if self.is_base() {
            let tmp = Ptr::new(UndefinedObject::new());
            info!(
                target:"SkillParsing",
                "Allocate space for:UndefinedPool amount:{}",
                self.get_global_cached_count(),
            );
            trace!(
                target:"SkillParsing",
                "Allocate space for:UndefinedPool with:{:?}",
                tmp,
            );
            vec.reserve(self.get_global_cached_count()); // FIXME check if dynamic count is the correct one
                                                         // TODO figure out a better way - set_len doesn't wrk as dtor will be called on garbage data
            for _ in 0..self.get_global_cached_count() {
                vec.push(tmp.clone());
            }
        }
        self.book_static.reserve(self.static_count);

        info!(
            target:"SkillParsing",
            "Initialize UndefinedPool id:{}",
            self.get_type_id(),
        );

        for block in self.blocks.iter() {
            let begin = block.bpo + 1;
            let end = begin + block.static_count;
            for id in begin..end {
                if self.super_pool.is_some() {
                    let pool = self.super_pool.as_ref().unwrap().borrow();
                    trace!(
                        target:"SkillParsing",
                        "UndefinedObject id:{} super:{:?} block:{:?}",
                        id,
                        pool.get_type_id(),
                        block,
                    );
                    self.book_static.push(pool.make_instance());
                } else {
                    trace!(
                        target:"SkillParsing",
                        "UndefinedObject id:{} block:{:?}",
                        id,
                        block,
                    );
                    let tmp = self.make_instance();
                    self.book_static.push(tmp);
                }
                vec[id - 1] = self.book_static.last().unwrap().clone();
            }
        }
    }

    fn make_instance(&self) -> Ptr<SkillObject> {
        if let Some(pool) = self.super_pool.as_ref() {
            return pool.borrow().make_instance();
        }
        trace!(
            target:"SkillParsing",
            "Create new UndefinedObject",
        );
        Ptr::new(UndefinedObject {})
    }
}
