use common::error::*;
use common::internal::io::*;
use common::internal::*;
use common::iterator::dynamic_instances;
use common::*;
use SkillFile;
use SkillFileBuilder;

use std::cell::RefCell;
use std::rc::{Rc, Weak};

/// Used by `TypeBlock` to make the appropriate pools - implemented by `SkillFileBuilder`
pub(crate) trait PoolMaker {
    /// # Arguments
    /// * `type_name` - name of the type / pool
    /// * `type_id` - id of the type / pool
    /// * `super_pool` - super pool of the type / pool
    ///
    /// If no specific `UserTypePool` of the given name exists a `ForeignPool` will be create instead
    fn make_pool(
        &mut self,
        type_name: &Rc<SkillString>,
        type_id: usize,
        super_pool: Option<Rc<RefCell<PoolProxy>>>,
    ) -> Result<Rc<RefCell<PoolProxy>>, SkillFail>;

    /// # Returns
    /// `PoolProxy` the given index
    fn get_pool(&self, type_name_index: usize) -> Option<Rc<RefCell<PoolProxy>>>;
}

/// used as interface for the specific `UserTypePools`
pub(crate) trait PoolProxy {
    /// # Returns
    /// Borrow to the general `Pool`
    fn pool(&self) -> &Pool;
    /// # Returns
    /// Borrow to the mutable general `Pool`
    fn pool_mut(&mut self) -> &mut Pool;
    /// completes the `UserTypePool` - missing fields are created
    ///
    /// # Arguments
    /// * `file` - state to use to create the missing FieldDeclarations
    fn complete(&mut self, file: &SkillFileBuilder);
}

/// used to pass `Pool` specific functions from `UserTypePools`
pub(crate) trait PoolPartsMaker {
    /// makes a FieldDeclaration
    /// # Arguments
    /// * `index` - index of the field declaration to be created
    /// * `name` - name of the field declaration to be created
    /// * `field_type` - type of the field declaration to be created
    /// * `string_pool` - string access
    fn make_field(
        &self,
        index: usize,
        name: Rc<SkillString>,
        field_type: FieldType,
        string_pool: &StringBlock,
    ) -> Result<(bool, Box<RefCell<FieldDeclaration>>), SkillFail>;
    /// creates a `UserType` instance
    ///
    /// # Arguments
    /// * `skill_id` - object id / index into the type hierarchy array
    /// * `type_id` - type id / index into the pool array
    fn make_instance(&self, skill_id: usize, type_id: usize) -> Ptr<SkillObject>;
}

/// General pool - this is specification independent code
pub(crate) struct Pool {
    // NOTE PoolPartsMaker is needed for specification specific functions
    parts_maker: Box<PoolPartsMaker>,
    string_block: Rc<RefCell<StringBlock>>,
    // type hierarchy array of "old" instances
    instances: Rc<RefCell<Vec<Ptr<SkillObject>>>>,
    // only of the specific UserType
    own_new_instances: Vec<Ptr<SkillObject>>,
    fields: Vec<Box<RefCell<FieldDeclaration>>>,
    name: Rc<SkillString>,
    type_id: usize,
    blocks: Vec<Block>,
    super_pool: Option<Weak<RefCell<PoolProxy>>>,
    sub_pools: Vec<Weak<RefCell<PoolProxy>>>,
    // used for iterators
    base_pool: Option<Weak<RefCell<PoolProxy>>>,
    // used for iterators
    next_pool: Option<Weak<RefCell<PoolProxy>>>,
    // instances of the specific UserType
    static_count: usize,
    // instances of type hierarchy
    dynamic_count: usize,
    // cached dynamic count
    cached_count: usize,
    // instances that will be deleted at flush / write
    deleted_count: usize,
    type_hierarchy_height: usize,
    invariant: bool,
    // needed for late initialization of foreign fields
    foreign_fields: bool,
    block_reader: Weak<RefCell<Vec<FileReader>>>,
    type_pools: Weak<Vec<Rc<RefCell<PoolProxy>>>>,
}

impl Pool {
    /// # Arguments
    /// * `name` - name of the type / pool
    /// * `type_id` - type id / index into the pool array
    /// * `pool_proxy` - needed to use specification dependent functions
    pub(crate) fn new(
        string_block: Rc<RefCell<StringBlock>>,
        name: Rc<SkillString>,
        type_id: usize,
        pool_proxy: Box<PoolPartsMaker>,
    ) -> Pool {
        Pool {
            string_block,
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
            block_reader: Weak::new(),
            type_pools: Weak::new(),
        }
    }

    /// # Returns
    /// `true` if the type has already a `FieldDeclaration` with the string id `name_id`
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

    /// adds a field to the type
    ///
    /// # Arguments
    /// * `index` - index of the `FieldDeclaration` to be created
    /// * `field_name` - name of the `FieldDeclaration`
    /// * `field_type` - the `FieldDeclaration` shall have
    /// * `chunk` - the `FieldDeclaration` appeared in
    pub(crate) fn add_field(
        &mut self,
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

        let string_block = self.string_block.borrow();
        let (foreign, field) =
            self.parts_maker
                .make_field(index, field_name, field_type, &string_block)?;
        field.borrow_mut().io.add_chunk(chunk);
        self.fields.push(field);
        if foreign {
            self.foreign_fields = true;
        }
        Ok(())
    }

    /// Initializes dependencies for the initialization of fields
    ///
    /// # Arguments
    /// * `block_reader` - vector where each element is a reader for a specific block
    /// * `type_pools` - all type pools -- usd to read a specific instance
    pub(crate) fn initialize_state(
        &mut self,
        block_reader: &Rc<RefCell<Vec<FileReader>>>,
        type_pools: &Rc<Vec<Rc<RefCell<PoolProxy>>>>,
    ) {
        self.block_reader = Rc::downgrade(block_reader);
        self.type_pools = Rc::downgrade(type_pools);
    }

    /// Initializes the fields after reading the skill binary file - necessary for all non foreign fields
    pub(crate) fn lazy_initialize_fields(&self) -> Result<(), SkillFail> {
        let type_pools = Weak::upgrade(&self.type_pools)
            .ok_or(SkillFail::internal(InternalFail::MissingTypePools))?;
        let block_reader = Weak::upgrade(&self.block_reader)
            .ok_or(SkillFail::internal(InternalFail::MissingBlockReader))?;
        let block_reader = block_reader.borrow();
        let string_block = self.string_block.borrow();

        let instances = self.instances.borrow();
        for f in self.fields.iter() {
            f.borrow().io.lazy_read(
                &block_reader,
                &string_block,
                &self.blocks,
                &type_pools,
                &instances,
            )?;
        }
        Ok(())
    }

    /// Initializes the fields after before writing the skill binary file - necessary for foreign fields
    pub fn initialize_all_fields(&self) -> Result<(), SkillFail> {
        debug!(
            target: "SkillWriting",
            "~~~Deserialize foreign data for {}", self.name.as_str(),
        );
        let type_pools = Weak::upgrade(&self.type_pools)
            .ok_or(SkillFail::internal(InternalFail::MissingTypePools))?;
        let block_reader = Weak::upgrade(&self.block_reader)
            .ok_or(SkillFail::internal(InternalFail::MissingBlockReader))?;
        let block_reader = block_reader.borrow();
        let string_block = self.string_block.borrow();

        let instances = self.instances.borrow();
        for f in self.fields.iter() {
            f.borrow_mut().io.force_read(
                &block_reader,
                &string_block,
                &self.blocks,
                &type_pools,
                &instances,
            )?;
        }
        if let Some(pool) = self.super_pool.as_ref() {
            if let Some(pool) = pool.upgrade() {
                return pool.borrow().pool().initialize_all_fields();
            }
        }
        Ok(())
    }

    /// initializes the fields after before writing the skill binary file - necessary for foreign fields
    ///
    /// # Arguments
    /// * `name` - field to initialize
    pub fn initialize_field(&self, name: &str) -> Result<(), SkillFail> {
        debug!(
            target: "SkillWriting",
            "~~~Deserialize foreign data for {}", self.name.as_str(),
        );
        let type_pools = Weak::upgrade(&self.type_pools)
            .ok_or(SkillFail::internal(InternalFail::MissingTypePools))?;
        let block_reader = Weak::upgrade(&self.block_reader)
            .ok_or(SkillFail::internal(InternalFail::MissingBlockReader))?;
        let block_reader = block_reader.borrow();
        let string_block = self.string_block.borrow();

        let instances = self.instances.borrow();
        for f in self.fields.iter() {
            if f.borrow().name().as_str() == name {
                return f.borrow_mut().io.force_read(
                    &block_reader,
                    &string_block,
                    &self.blocks,
                    &type_pools,
                    &instances,
                );
            }
        }
        Err(SkillFail::user(UserFail::UnknownField {
            name: name.to_owned(),
        }))
    }

    /// allocates all instances without initialization of fields
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
    /// # Arguments
    /// * `field_id` - field to add the chunk for
    /// * `chunk` - chunk to add
    pub(crate) fn add_chunk_to(
        &mut self,
        field_id: usize,
        chunk: FieldChunk,
    ) -> Result<(), SkillFail> {
        for f in &mut self.fields.iter() {
            let mut f = f.borrow_mut();
            if f.io.field_id() == field_id {
                f.io.add_chunk(chunk);
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

    /// # Returns
    /// vector of the type hierarchy
    pub(crate) fn get_base_vec(&self) -> Rc<RefCell<Vec<Ptr<SkillObject>>>> {
        self.instances.clone()
    }

    pub(crate) fn delete(&mut self, instance: Ptr<SkillObject>) {
        instance.borrow_mut().mark_for_deletion();
        self.deleted_count += 1;
    }
    /// "Reads" a object from the type hierarchy vector
    ///
    /// # Arguments
    /// * `index` - Index/ID of the object to retrieve
    ///
    /// # Returns
    /// Instance of given index
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
    /// # Arguments
    /// * `block` - to add to the type to get data and instances from
    pub(crate) fn add_block(&mut self, block: Block) {
        self.blocks.push(block);
    }
    pub(crate) fn blocks(&self) -> &Vec<Block> {
        &self.blocks
    }
    /// # Arguments
    /// * `pool` - to add as sub pool
    pub(crate) fn add_sub(&mut self, pool: &Rc<RefCell<PoolProxy>>) {
        self.sub_pools.push(Rc::downgrade(pool));
    }
    /// # Arguments
    /// * `pool` - to add as super pool to this pool - this will also add this pool as sub pool to the super pool
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
    /// # Returns
    /// Amount of instances of this pools type in the last block
    pub(crate) fn get_local_static_count(&self) -> usize {
        return self.blocks.last().unwrap().static_count;
    }
    /// # Arguments
    /// * `count` - of instances of this pools type in the last block
    pub(crate) fn set_local_static_count(&mut self, count: usize) {
        self.blocks.last_mut().unwrap().static_count = count
    }

    /// # Returns
    /// Amount of instances of the type hierarchy in the last block
    pub(crate) fn get_local_dynamic_count(&self) -> usize {
        return self.blocks.last().unwrap().dynamic_count;
    }

    /// # Returns
    /// base pool offset of the last block
    pub(crate) fn get_local_bpo(&self) -> usize {
        self.blocks.last().unwrap().bpo
    }

    /// # Arguments
    /// * `invariant` - if `true` the dynamic instance amount is calculated and cached, if `false` this will be reversed
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

    /// # Returns
    /// Amount of instances of the type hierarchy
    pub(crate) fn get_global_static_count(&self) -> usize {
        self.static_count
    }
    /// # Arguments
    /// * `count` - of instances of this pools type
    pub(crate) fn set_global_static_count(&mut self, count: usize) {
        self.static_count = count;
    }

    /// # Returns
    /// Amount of cached dynamic instances of the type hierarchy
    pub(crate) fn get_global_cached_count(&self) -> usize {
        self.cached_count
    }
    /// # Arguments
    /// * `count` - of cached dynamic instances of this pools type
    pub(crate) fn set_global_cached_count(&mut self, count: usize) {
        self.cached_count = count;
    }

    /// # Arguments
    /// * `skill_id` - object id / index into the type hierarchy array
    /// * `type_id` - type id / index into the pool array
    pub(crate) fn make_instance(&self, skill_id: usize, type_id: usize) -> Ptr<SkillObject> {
        trace!(
            target: "SkillParsing",
            "Create new foreign::{}",
            self.name.as_str()
        );
        self.parts_maker.make_instance(skill_id, type_id)
    }

    /// # Arguments
    /// * `local_bpo` - base pool offsets to use
    /// * `vec` - new instance vector with all instances of this type hierarchy
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
    /// # Arguments
    /// * `pool` - to be set as next for iterators - is propagated to sub pools
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

    /// Used by iterators
    ///
    /// # Returns
    /// The next pool in the type hierarchy
    pub(crate) fn get_next_pool(&self) -> Option<Weak<RefCell<PoolProxy>>> {
        self.next_pool.clone()
    }

    /// Used by iterators
    ///
    /// # Returns
    /// The height of the hierarchy
    pub(crate) fn type_hierarchy_height(&self) -> usize {
        self.type_hierarchy_height
    }

    /// Collapses the field chunks into one
    pub(crate) fn compress_field_chunks(&mut self) {
        let total_count = self.get_global_cached_count();
        for f in self.fields.iter() {
            f.borrow_mut().io.compress_chunks(total_count);
        }
    }

    /// # Arguments
    /// * `writer` - to write
    /// * `local_bpos` - base pool offsets to use
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

    /// # Arguments
    /// * `writer` - to write
    /// * `iter` - iterator to traverse all instances. Can't be created by this type as a wrapper is needed
    /// * `offset` - into the data section
    ///
    /// # Returns
    /// advanced offset into the data section
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
            offset = f.borrow_mut().io.write_meta(writer, iter.clone(), offset)?;
        }
        Ok(offset)
    }
    /// # Arguments
    /// * `writer` - to write
    /// * `iter` - iterator to traverse all instances. Can't be created by this type as a wrapper is needed
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
            f.borrow().io.write_data(writer, iter.clone())?;
        }
        Ok(())
    }
}
