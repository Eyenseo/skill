use common::error::*;
use common::internal::foreign;
use common::internal::*;
use common::io::*;
use common::iterator::dynamic_data;
use common::*;
use SkillFile;

use std::cell::Cell;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

pub struct Pool {
    instances: Rc<RefCell<Vec<Ptr<SkillObject>>>>,
    own_static_instances: Vec<Ptr<SkillObject>>,
    own_new_instances: Vec<Ptr<SkillObject>>,
    fields: Vec<Box<RefCell<foreign::FieldDeclaration>>>,
    name: Rc<SkillString>,
    type_id: usize,
    blocks: Vec<Block>,
    super_pool: Option<Rc<RefCell<InstancePool>>>,
    sub_pools: Vec<Rc<RefCell<InstancePool>>>,
    base_pool: Option<Rc<RefCell<InstancePool>>>,
    next_pool: Option<Rc<RefCell<InstancePool>>>,
    static_count: usize,
    dynamic_count: usize,
    cached_count: usize,
    deleted_count: usize,
    invariant: bool,
    type_hierarchy_height: usize,
}

impl Pool {
    pub fn new(name: Rc<SkillString>, type_id: usize) -> Pool {
        Pool {
            instances: Rc::default(),
            own_static_instances: Vec::new(),
            own_new_instances: Vec::new(),
            fields: Vec::new(),
            name,
            type_id,
            blocks: Vec::new(),
            super_pool: None,
            sub_pools: Vec::new(),
            base_pool: None,
            next_pool: None,
            static_count: 0,
            dynamic_count: 0,
            cached_count: 0,
            deleted_count: 0,
            invariant: false,
            type_hierarchy_height: 0,
        }
    }
}

impl InstancePool for Pool {
    fn add_field(
        &mut self,
        field_id: usize,
        field_name: Rc<SkillString>,
        field_type: FieldType,
        chunk: FieldChunk,
    ) -> Result<(), SkillFail> {
        let mut reader = Box::new(RefCell::new(foreign::FieldDeclaration::new(
            field_name, field_id, field_type,
        )));
        reader.borrow_mut().add_chunk(chunk);
        self.fields.push(reader);
        Ok(())
    }
    fn has_field(&self, name_id: usize) -> bool {
        for f in &self.fields {
            let f = f.as_ref();
            if f.borrow().name().get_skill_id() == name_id {
                return true;
            }
        }
        false
    }
    fn field_amount(&self) -> usize {
        self.fields.len()
    }
    fn add_chunk_to(&mut self, field_id: usize, chunk: FieldChunk) -> Result<(), SkillFail> {
        for f in self.fields.iter() {
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

    fn set_type_id(&mut self, id: usize) {
        self.type_id = id;
    }
    fn get_type_id(&self) -> usize {
        self.type_id
    }

    fn name(&self) -> &Rc<SkillString> {
        &self.name
    }

    fn add_block(&mut self, block: Block) {
        self.blocks.push(block);
    }
    fn blocks(&self) -> &Vec<Block> {
        &self.blocks
    }
    fn blocks_mut(&mut self) -> &mut Vec<Block> {
        &mut self.blocks
    }

    fn set_super(&mut self, pool: Rc<RefCell<InstancePool>>) {
        if pool.borrow().is_base() {
            self.base_pool = Some(pool.clone());
        } else {
            self.base_pool = pool.borrow().get_base(); // TODO check?
        }
        self.instances = self.base_pool.as_ref().unwrap().borrow().get_base_vec();
        self.type_hierarchy_height = pool.borrow().type_hierarchy_height() + 1;
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
        return self.blocks.last().unwrap().static_count;
    }
    fn set_local_static_count(&mut self, count: usize) {
        self.blocks.last_mut().unwrap().static_count = count
    }

    fn get_local_dynamic_count(&self) -> usize {
        return self.blocks.last().unwrap().dynamic_count;
    }

    fn get_local_bpo(&self) -> usize {
        self.blocks.last().unwrap().bpo
    }

    fn set_invariant(&mut self, invariant: bool) {
        if self.invariant != invariant {
            self.invariant = invariant;
            if invariant {
                self.cached_count = self.static_size() - self.deleted_count;
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
    fn read_object(&self, index: usize) -> Result<Ptr<SkillObject>, SkillFail> {
        if index == 0 {
            return Err(SkillFail::internal(InternalFail::ReservedID { id: 0 }));
        }
        info!(
            target: "SkillParsing",
            "read user instance:{} from:{}",
            index,
            self.instances.borrow().len(),
        );
        Ok(self.instances.borrow()[index - 1].clone())
    }

    fn initialize(
        &self,
        _block_reader: &Vec<FileReader>,
        _string_pool: &StringBlock,
        _type_pools: &Vec<Rc<RefCell<InstancePool>>>,
    ) -> Result<(), SkillFail> {
        // NOTE this is defered until writing - see the deserialize method
        Ok(())
    }
    fn deserialize(&self, skill_file: &SkillFile) -> Result<(), SkillFail> {
        info!(
            target:"SkillWriting",
            "~~~Deserialize undefind Data for Pool aka {}",
            self.name.as_str(),
        );

        let block_reader = skill_file.block_reader.borrow();
        let string_pool = skill_file.strings.borrow();

        for f in self.fields.iter() {
            let instances = self.instances.borrow();
            f.borrow_mut().deserialize(
                &block_reader,
                &string_pool,
                &self.blocks,
                skill_file.type_pool.pools(),
                &instances,
            )?;
        }
        Ok(())
    }

    fn allocate(&mut self) {
        let mut vec = self.instances.borrow_mut();
        if self.is_base() {
            // TODO add extra Garbage / placeholder object
            let tmp = Ptr::new(foreign::Object::new(0, 0));
            info!(
                target: "SkillParsing",
                "Allocate space for:Pool amount:{}",
                self.get_global_cached_count(),
            );
            trace!(
                target: "SkillParsing",
                "Allocate space for:Pool with:{:?}",
                tmp,
            );
            vec.reserve(self.get_global_cached_count());
            // TODO figure out a better way - set_len doesn't wrk as dtor will be called on garbage data
            for _ in 0..self.get_global_cached_count() {
                vec.push(tmp.clone());
            }
        }
        self.own_static_instances.reserve(self.static_count);

        info!(
            target: "SkillParsing",
            "Initialize Pool id:{}",
            self.get_type_id(),
        );

        for block in self.blocks.iter() {
            let begin = block.bpo + 1;
            let end = begin + block.static_count;
            for id in begin..end {
                if self.super_pool.is_some() {
                    let pool = self.super_pool.as_ref().unwrap().borrow();
                    trace!(
                        target: "SkillParsing",
                        "Object id:{} super:{:?} block:{:?}",
                        id,
                        pool.get_type_id(),
                        block,
                    );
                    self.own_static_instances
                        .push(pool.make_foreign(id, self.type_id));
                } else {
                    trace!(
                        target: "SkillParsing",
                        "Object id:{} block:{:?}",
                        id,
                        block,
                    );
                    let tmp = self.make_foreign(id, self.type_id);
                    self.own_static_instances.push(tmp);
                }
                vec[id - 1] = self.own_static_instances.last().unwrap().clone();
            }
        }
    }

    fn make_foreign(&self, skill_id: usize, skill_type_id: usize) -> Ptr<SkillObject> {
        if let Some(pool) = self.super_pool.as_ref() {
            return pool.borrow().make_foreign(skill_id, skill_type_id);
        }
        trace!(
            target: "SkillParsing",
            "Create new Object",
        );
        Ptr::new(foreign::Object::new(skill_id, skill_type_id))
    }
    fn update_after_compress(
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
            block: BlockIndex::from(0),
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

    fn static_instances(&self) -> &Vec<Ptr<SkillObject>> {
        &self.own_static_instances
    }
    fn new_instances(&self) -> &Vec<Ptr<SkillObject>> {
        &self.own_new_instances
    }

    fn static_size(&self) -> usize {
        self.static_count + self.own_new_instances.len()
    }
    fn dynamic_size(&self) -> usize {
        if self.invariant {
            self.cached_count
        } else {
            let mut ret = self.static_size();
            for s in self.sub_pools.iter() {
                ret += s.borrow().static_size();
            }
            ret
        }
    }
    fn deleted(&self) -> usize {
        self.deleted_count
    }

    fn set_next_pool(&mut self, pool: Option<Rc<RefCell<InstancePool>>>) {
        if self.sub_pools.len() > 0 {
            self.next_pool = Some(self.sub_pools.first().unwrap().clone());
            for i in 0..self.sub_pools.len() - 1 {
                self.sub_pools[i]
                    .borrow_mut()
                    .set_next_pool(Some(self.sub_pools[i + 1].clone()));
            }
            self.sub_pools
                .last()
                .unwrap()
                .borrow_mut()
                .set_next_pool(pool);
        } else {
            self.next_pool = pool;
        }
    }
    fn get_next_pool(&self) -> Option<Rc<RefCell<InstancePool>>> {
        self.next_pool.clone()
    }
    fn type_hierarchy_height(&self) -> usize {
        self.type_hierarchy_height
    }
    fn compress_field_chunks(&mut self, local_bpo: &Vec<usize>) {
        let total_count = self.get_global_cached_count();
        for f in self.fields.iter() {
            f.borrow_mut().compress_chunks(total_count);
        }
    }
    fn write_type_meta(
        &self,
        writer: &mut FileWriter,
        local_bpos: &Vec<usize>,
    ) -> Result<(), SkillFail> {
        info!(
            target: "SkillWriting",
            "~~~Write Meta Data for Pool:{} Instances; Static:{} Dynamic:{}",
            self.name.as_ref(),
            self.get_local_static_count(),
            self.get_local_dynamic_count(),
        );

        writer.write_v64(self.name().get_skill_id() as i64)?;
        writer.write_v64(self.get_local_dynamic_count() as i64)?;
        // FIXME restrictions
        writer.write_v64(0)?;
        if let Some(s) = self.get_super() {
            writer.write_v64((s.borrow().get_type_id() - 31) as i64)?;
            if self.get_local_dynamic_count() != 0 {
                writer.write_v64(local_bpos[self.get_type_id() - 32] as i64)?;
            }
        } else {
            // tiny optimisation
            writer.write_i8(0)?;
        }
        writer.write_v64(self.field_amount() as i64)?;
        Ok(())
    }
    fn write_field_meta(
        &self,
        writer: &mut FileWriter,
        iter: dynamic_data::Iter,
        mut offset: usize,
    ) -> Result<usize, SkillFail> {
        info!(
            target: "SkillWriting",
            "~~~Write Field Meta Data for Pool:{} Fields:{}",
            self.name.as_ref(),
            self.fields.len(),
        );
        for f in self.fields.iter() {
            offset = f.borrow_mut().write_meta(writer, iter.clone(), offset)?;
        }
        Ok(offset)
    }
    fn write_field_data(
        &self,
        writer: &mut FileWriter,
        iter: dynamic_data::Iter,
    ) -> Result<(), SkillFail> {
        info!(
            target: "SkillWriting",
            "~~~Write Field Data for Pool:{} Fields:{}",
            self.name.as_ref(),
            self.fields.len(),
        );
        for f in self.fields.iter() {
            f.borrow().write_data(writer, iter.clone())?;
        }
        Ok(())
    }
}
