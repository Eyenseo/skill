/*
 * @author Roland Jaeger
 */

use common::error::*;
use common::internal::io::*;
use common::internal::*;
use common::iterator::dynamic_instances;
use common::*;
use SkillFile;
use SkillFileBuilder;

use std::cell::RefCell;
use std::rc::{Rc, Weak};

pub(crate) trait PoolMaker {
    fn make_pool(
        &mut self,
        type_name: &Rc<SkillString>,
        type_id: usize,
        super_pool: Option<Rc<RefCell<PoolProxy>>>,
    ) -> Result<Rc<RefCell<PoolProxy>>, SkillFail>;

    fn get_pool(&self, type_name_index: usize) -> Option<Rc<RefCell<PoolProxy>>>;
}

pub(crate) trait PoolProxy {
    fn pool(&self) -> &Pool;
    fn pool_mut(&mut self) -> &mut Pool;
    fn complete(&mut self, file: &SkillFileBuilder);
}

pub(crate) trait PoolPartsMaker {
    fn make_field(
        &self,
        index: usize,
        name: Rc<SkillString>,
        field_type: FieldType,
        string_pool: &StringBlock,
    ) -> Result<(bool, Box<RefCell<FieldDeclaration>>), SkillFail>;
    fn make_instance(&self, skill_id: usize, type_id: usize) -> Ptr<SkillObject>;
}

pub(crate) struct Pool {
    parts_maker: Box<PoolPartsMaker>,
    instances: Rc<RefCell<Vec<Ptr<SkillObject>>>>,
    own_new_instances: Vec<Ptr<SkillObject>>,
    fields: Vec<Box<RefCell<FieldDeclaration>>>,
    name: Rc<SkillString>,
    type_id: usize,
    blocks: Vec<Block>,
    super_pool: Option<Weak<RefCell<PoolProxy>>>,
    sub_pools: Vec<Weak<RefCell<PoolProxy>>>,
    base_pool: Option<Weak<RefCell<PoolProxy>>>,
    next_pool: Option<Weak<RefCell<PoolProxy>>>,
    static_count: usize,
    dynamic_count: usize,
    cached_count: usize,
    deleted_count: usize,
    type_hierarchy_height: usize,
    invariant: bool,
    foreign_fields: bool,
}

impl Pool {
    pub(crate) fn new(
        name: Rc<SkillString>,
        type_id: usize,
        pool_proxy: Box<PoolPartsMaker>,
    ) -> Pool {
        Pool {
            parts_maker: pool_proxy,
            instances: Rc::default(),
            own_new_instances: Vec::new(),
            fields: Vec::new(),
            name,
            type_id,
            blocks: Vec::new(),
            super_pool: None,
            base_pool: None,
            next_pool: None,
            sub_pools: Vec::new(),
            static_count: 0,
            dynamic_count: 0,
            cached_count: 0,
            deleted_count: 0,
            type_hierarchy_height: 0,
            invariant: false,
            foreign_fields: false,
        }
    }

    fn has_field(&self, name_id: usize) -> bool {
        for f in self.fields.iter() {
            if f.borrow().name().get_id() == name_id {
                return true;
            }
        }
        false
    }

    pub(crate) fn fields_mut(&mut self) -> &mut Vec<Box<RefCell<FieldDeclaration>>> {
        &mut self.fields
    }
    pub(crate) fn fields(&self) -> &Vec<Box<RefCell<FieldDeclaration>>> {
        &self.fields
    }
    pub(crate) fn add_field(
        &mut self,
        string_pool: &StringBlock,
        index: usize,
        field_name: Rc<SkillString>,
        mut field_type: FieldType,
        chunk: FieldChunk,
    ) -> Result<(), SkillFail> {
        // TODO could be speed up with a set
        for f in self.fields.iter() {
            if f.borrow().name().as_str() == field_name.as_str() {
                Err(SkillFail::internal(InternalFail::SameField {
                    field: field_name.string().clone(),
                }))?;
            }
        }
        let (foreign, field) =
            self.parts_maker
                .make_field(index, field_name, field_type, string_pool)?;
        field.borrow_mut().add_chunk(chunk);
        self.fields.push(field);
        if foreign {
            self.foreign_fields = true;
        }
        Ok(())
    }

    pub(crate) fn initialize(
        &self,
        block_reader: &Vec<FileReader>,
        string_pool: &StringBlock,
        type_pools: &Vec<Rc<RefCell<PoolProxy>>>,
    ) -> Result<(), SkillFail> {
        for f in self.fields.iter() {
            let instances = self.instances.borrow();
            f.borrow().read(
                block_reader,
                string_pool,
                &self.blocks,
                type_pools,
                &instances,
            )?;
        }
        Ok(())
    }

    pub(crate) fn deserialize(&self, skill_file: &SkillFile) -> Result<(), SkillFail> {
        debug!(
            target: "SkillWriting",
            "~~~Deserialize foreign data for {}", self.name.as_str(),
        );

        let block_reader = skill_file.block_reader();
        let string_pool = skill_file.strings().string_block();
        let string_pool = string_pool.borrow();

        for f in self.fields.iter() {
            let instances = self.instances.borrow();
            f.borrow_mut().deserialize(
                &block_reader,
                &string_pool,
                &self.blocks,
                skill_file.type_pool().pools(),
                &instances,
            )?;
        }
        Ok(())
    }

    pub(crate) fn allocate(&mut self) {
        let mut vec = self.instances.borrow_mut();
        if self.is_base() {
            // TODO add garbage type
            let tmp = Ptr::new(foreign::Foreign::new(0, 0));
            trace!(
                target: "SkillParsing",
                "Allocate space for:{} amount:{} with:{:?}",
                self.name.as_str(),
                self.get_global_cached_count(),
                tmp,
            );
            vec.reserve(self.get_global_cached_count());
            // TODO figure out a better way - set_len doesn't wrk as dtor will be called on garbage data
            for _ in 0..self.get_global_cached_count() {
                vec.push(tmp.clone());
            }
        }

        debug!(
            target: "SkillParsing",
            "Initialize:{} id:{}",
            self.name.as_str(),
            self.get_type_id(),
        );

        for block in self.blocks.iter() {
            let begin = block.bpo + 1;
            let end = begin + block.static_count;
            for id in begin..end {
                trace!(
                    target: "SkillParsing",
                    "{} id:{} block:{:?}",
                    self.name.as_str(),
                    id,
                    block,
                );
                vec[id - 1] = self.parts_maker.make_instance(id, self.type_id);
            }
        }
    }

    pub(crate) fn add_chunk_to(
        &mut self,
        field_id: usize,
        chunk: FieldChunk,
    ) -> Result<(), SkillFail> {
        for f in &mut self.fields.iter() {
            let mut f = f.borrow_mut();
            if f.field_id() == field_id {
                f.add_chunk(chunk);
                return Ok(());
            }
        }
        Err(SkillFail::internal(InternalFail::UnknownField {
            id: field_id,
        }))
    }

    pub(crate) fn get_type_id(&self) -> usize {
        self.type_id
    }

    pub(crate) fn name(&self) -> &Rc<SkillString> {
        &self.name
    }

    pub(crate) fn get_base_vec(&self) -> Rc<RefCell<Vec<Ptr<SkillObject>>>> {
        self.instances.clone()
    }

    pub(crate) fn delete(&mut self, instance: Ptr<SkillObject>) {
        instance.borrow_mut().mark_for_deletion();
        self.deleted_count += 1;
    }
    pub(crate) fn read_object(&self, index: usize) -> Result<Ptr<SkillObject>, SkillFail> {
        if index == 0 {
            return Err(SkillFail::internal(InternalFail::ReservedID { id: 0 }));
        }
        debug!(
            target: "SkillParsing",
            "read user instance:{} from:{}",
            index,
            self.instances.borrow().len(),
        );
        Ok(self.instances.borrow()[index - 1].clone())
    }

    pub(crate) fn add_block(&mut self, block: Block) {
        self.blocks.push(block);
    }
    pub(crate) fn blocks(&self) -> &Vec<Block> {
        &self.blocks
    }

    pub(crate) fn add_sub(&mut self, pool: &Rc<RefCell<PoolProxy>>) {
        self.sub_pools.push(Rc::downgrade(pool));
    }

    pub(crate) fn set_super(&mut self, pool: &Rc<RefCell<PoolProxy>>) {
        if pool.borrow().pool().is_base() {
            self.base_pool = Some(Rc::downgrade(pool));
        } else {
            self.base_pool = pool.borrow().pool().get_base(); // TODO check?
        }
        self.instances = self
            .base_pool
            .as_ref()
            .unwrap()
            .upgrade()
            .unwrap()
            .borrow()
            .pool()
            .get_base_vec();
        self.type_hierarchy_height = pool.borrow().pool().type_hierarchy_height() + 1;
        self.super_pool = Some(Rc::downgrade(pool));
    }
    pub(crate) fn get_super(&self) -> Option<Weak<RefCell<PoolProxy>>> {
        self.super_pool.clone()
    }

    pub(crate) fn get_base(&self) -> Option<Weak<RefCell<PoolProxy>>> {
        self.base_pool.clone()
    }
    pub(crate) fn is_base(&self) -> bool {
        self.super_pool.is_none()
    }

    pub(crate) fn get_local_static_count(&self) -> usize {
        return self.blocks.last().unwrap().static_count;
    }
    pub(crate) fn set_local_static_count(&mut self, count: usize) {
        self.blocks.last_mut().unwrap().static_count = count
    }

    pub(crate) fn get_local_dynamic_count(&self) -> usize {
        return self.blocks.last().unwrap().dynamic_count;
    }

    pub(crate) fn get_local_bpo(&self) -> usize {
        self.blocks.last().unwrap().bpo
    }

    pub(crate) fn set_invariant(&mut self, invariant: bool) {
        if self.invariant != invariant {
            self.invariant = invariant;
            if invariant {
                self.cached_count = self.static_size() - self.deleted_count;
                for s in self.sub_pools.iter() {
                    let mut s = s.upgrade().unwrap();
                    let mut s = s.borrow_mut();
                    let mut s = s.pool_mut();
                    s.set_invariant(true);
                    self.cached_count += s.get_global_cached_count();
                }
            } else if self.super_pool.is_some() {
                self.super_pool
                    .as_ref()
                    .unwrap()
                    .upgrade()
                    .unwrap()
                    .borrow_mut()
                    .pool_mut()
                    .set_invariant(false);
            }
        }
    }

    pub(crate) fn get_global_static_count(&self) -> usize {
        self.static_count
    }
    pub(crate) fn set_global_static_count(&mut self, count: usize) {
        self.static_count = count;
    }

    pub(crate) fn get_global_cached_count(&self) -> usize {
        self.cached_count
    }
    pub(crate) fn set_global_cached_count(&mut self, count: usize) {
        self.cached_count = count;
    }

    pub(crate) fn make_instance(&self, skill_id: usize, skill_type_id: usize) -> Ptr<SkillObject> {
        trace!(
            target: "SkillParsing",
            "Create new foreign::{}",
            self.name.as_str()
        );
        self.parts_maker.make_instance(skill_id, skill_type_id)
    }

    pub(crate) fn update_after_compress(
        &mut self,
        local_bpo: &Vec<usize>,
        vec: Rc<RefCell<Vec<Ptr<SkillObject>>>>,
    ) {
        self.instances = vec;
        self.static_count += self.own_new_instances.len();
        self.own_new_instances = Vec::new();
        self.blocks = Vec::with_capacity(1);
        let static_size = self.static_size();
        self.blocks.push(Block {
            block: 0,
            bpo: local_bpo[self.type_id - 32],
            static_count: static_size,
            dynamic_count: self.cached_count,
        });
        trace!(
            target: "SkillWriting",
            "Updated Block:{:?}",
            self.blocks.last().unwrap(),
        );
    }

    pub(crate) fn new_instances(&self) -> &Vec<Ptr<SkillObject>> {
        &self.own_new_instances
    }

    pub(crate) fn static_size(&self) -> usize {
        self.static_count + self.own_new_instances.len()
    }

    pub(crate) fn deleted_instances(&self) -> usize {
        self.deleted_count
    }

    pub(crate) fn add(&mut self, instance: Ptr<SkillObject>) {
        self.own_new_instances.push(instance);
    }

    pub(crate) fn set_next_pool(&mut self, pool: Option<Rc<RefCell<PoolProxy>>>) {
        if self.sub_pools.len() > 0 {
            self.next_pool = Some(self.sub_pools.first().unwrap().clone());
            for i in 0..self.sub_pools.len() - 1 {
                self.sub_pools[i]
                    .upgrade()
                    .unwrap()
                    .borrow_mut()
                    .pool_mut()
                    .set_next_pool(Some(self.sub_pools[i + 1].upgrade().unwrap()));
            }
            self.sub_pools
                .last()
                .unwrap()
                .upgrade()
                .unwrap()
                .borrow_mut()
                .pool_mut()
                .set_next_pool(pool);
        } else {
            if let Some(pool) = pool {
                self.next_pool = Some(Rc::downgrade(&pool));
            } else {
                self.next_pool = None;
            }
        }
    }
    pub(crate) fn get_next_pool(&self) -> Option<Weak<RefCell<PoolProxy>>> {
        self.next_pool.clone()
    }
    pub(crate) fn type_hierarchy_height(&self) -> usize {
        self.type_hierarchy_height
    }

    pub(crate) fn compress_field_chunks(&mut self, local_bpo: &Vec<usize>) {
        let total_count = self.get_global_cached_count();
        for f in self.fields.iter() {
            f.borrow_mut().compress_chunks(total_count);
        }
    }
    pub(crate) fn write_type_meta(
        &self,
        writer: &mut FileWriter,
        local_bpos: &Vec<usize>,
    ) -> Result<(), SkillFail> {
        debug!(
            target: "SkillWriting",
            "~~~Write Meta Data for:{} Instances; Static:{} Dynamic:{}",
            self.name.as_str(),
            self.get_local_static_count(),
            self.get_local_dynamic_count(),
        );

        writer.write_v64(self.name().get_id() as i64)?;
        writer.write_v64(self.get_local_dynamic_count() as i64)?;
        // FIXME restrictions
        writer.write_v64(0)?;
        if let Some(s) = self.get_super() {
            writer.write_v64((s.upgrade().unwrap().borrow().pool().get_type_id() - 31) as i64)?;
            if self.get_local_dynamic_count() != 0 {
                writer.write_v64(local_bpos[self.get_type_id() - 32] as i64)?;
            }
        } else {
            // tiny optimisation
            writer.write_i8(0)?;
        }
        writer.write_v64(self.fields().len() as i64)?;
        Ok(())
    }
    pub(crate) fn write_field_meta(
        &self,
        writer: &mut FileWriter,
        iter: dynamic_instances::Iter,
        mut offset: usize,
    ) -> Result<usize, SkillFail> {
        debug!(
            target: "SkillWriting",
            "~~~Write Field Meta Data for:{} Fields:{}",
            self.name.as_str(),
            self.fields.len(),
        );
        for f in self.fields.iter() {
            offset = f.borrow_mut().write_meta(writer, iter.clone(), offset)?;
        }
        Ok(offset)
    }
    pub(crate) fn write_field_data(
        &self,
        writer: &mut FileWriter,
        iter: dynamic_instances::Iter,
    ) -> Result<(), SkillFail> {
        debug!(
            target: "SkillWriting",
            "~~~Write Field Data for:{} Fields:{}",
            self.name.as_str(),
            self.fields.len(),
        );
        for f in self.fields.iter() {
            f.borrow().write_data(writer, iter.clone())?;
        }
        Ok(())
    }
}
