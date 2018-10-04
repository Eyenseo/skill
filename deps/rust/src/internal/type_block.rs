/*
 * @author Roland Jaeger
 */

use common::error::*;
use common::internal::foreign;
use common::internal::io::*;
use common::internal::*;
use common::iterator::*;
use common::*;

use std::cell::RefCell;
use std::rc::Rc;

#[derive(Default)]
pub(crate) struct TypeBlock {
    pools: Vec<Rc<RefCell<PoolProxy>>>,
}

impl TypeBlock {
    pub(crate) fn new() -> TypeBlock {
        TypeBlock { pools: Vec::new() }
    }

    pub(crate) fn pools(&self) -> &Vec<Rc<RefCell<PoolProxy>>> {
        &self.pools
    }

    pub(crate) fn read_type_pool(
        &mut self,
        block: BlockIndex,
        reader: &mut FileReader,
        pool_maker: &mut PoolMaker,
        string_pool: &Rc<RefCell<StringBlock>>,
        field_data: &mut Vec<FileReader>,
    ) -> Result<(), SkillFail> {
        let mut block_local_pools = Vec::new();
        let mut previous_type_id = 0;
        let mut seen_types = Vec::new();

        debug!(target: "SkillParsing", "~Block Start~");
        let type_amount = reader.read_v64()? as usize;
        debug!(target: "SkillParsing", "~Types: {:?}", type_amount);
        self.pools.reserve(type_amount);

        debug!(target: "SkillParsing", "~TypeData~");
        for _ in 0..type_amount {
            let type_name_index = reader.read_v64()? as u64;
            let type_name =
                if let Some(type_name) = string_pool.borrow().get(type_name_index as usize)? {
                    type_name
                } else {
                    return Err(SkillFail::internal(InternalFail::TypeOrFieldNameNull));
                };

            debug!(target: "SkillParsing", "~~TypeName: {:?}", type_name);

            if seen_types.contains(&type_name_index) {
                return Err(SkillFail::internal(InternalFail::RedefinitionOfType {
                    name: type_name.as_str().to_owned(),
                }));
            }
            seen_types.push(type_name_index);

            let instances = reader.read_v64()? as usize; // amount of instances
            debug!(target: "SkillParsing", "~~TypeInstances: {:?}", instances);

            let mut type_pool = if let Some(pool) = pool_maker.get_pool(type_name_index as usize) {
                pool
            } else {
                let type_id = self.pools.len() + 32;

                debug!(target: "SkillParsing", "~~New Type:{:?}", type_id);
                let type_restrictions = reader.read_v64()?; // restrictions ?
                for _ in 0..type_restrictions {
                    let restriction = reader.read_v64()?;
                    match restriction {
                        0x0 => (),
                        0x1 => (),
                        0x2 => (),
                        0x3 => (),
                        0x5 => {
                            let default_value = reader.read_v64();
                        }
                        id => {
                            return Err(SkillFail::internal(InternalFail::UnknownTypeRestriction {
                                id: id as usize,
                            }));
                        }
                    }
                }

                let super_type = reader.read_v64()?; // super type index? id?
                let super_pool = if super_type as usize > self.pools.len() {
                    return Err(SkillFail::internal(InternalFail::UnknownType {
                        id: super_type as usize,
                    }));
                } else if super_type != 0 {
                    debug!(
                        target: "SkillParsing",
                        "~~Add Super Type:{:?} for:{:?}",
                        self.pools[(super_type - 1) as usize].borrow().pool().get_type_id(),
                        type_id
                    );
                    // TODO check that this is the expected super type?
                    Some(self.pools[(super_type - 1) as usize].clone())
                } else {
                    None
                };

                let type_pool = pool_maker.make_pool(&type_name, type_id as usize, super_pool)?;
                self.pools.push(type_pool.clone());
                type_pool
            };

            {
                let mut type_pool = type_pool.borrow();
                let mut type_pool = type_pool.pool();

                if previous_type_id < type_pool.get_type_id() {
                    previous_type_id = type_pool.get_type_id();
                } else {
                    return Err(SkillFail::internal(InternalFail::UnorderedTypeBlock {
                        previous: previous_type_id,
                        current: type_pool.get_type_id(),
                        name: type_pool.name().as_str().to_owned(),
                    }));
                }
            }
            {
                let mut local_bpo = if let Some(base_pool) = type_pool.borrow().pool().get_base() {
                    base_pool
                        .upgrade()
                        .unwrap()
                        .borrow()
                        .pool()
                        .get_global_cached_count()
                } else {
                    type_pool.borrow().pool().get_global_cached_count()
                };
                // NOTE order prevents borrow panic
                let mut type_pool = type_pool.borrow_mut();
                let mut type_pool = type_pool.pool_mut();

                if let Some(super_pool) = type_pool.get_super() {
                    let super_pool = super_pool.upgrade().unwrap();
                    let super_pool = super_pool.borrow();

                    if instances != 0 {
                        local_bpo += reader.read_v64()? as usize;
                    } else {
                        local_bpo += super_pool.pool().get_local_bpo();
                    }

                    if local_bpo < super_pool.pool().get_local_bpo()
                        || super_pool.pool().get_local_bpo()
                            + super_pool.pool().get_local_dynamic_count()
                            < local_bpo
                    {
                        return Err(SkillFail::internal(InternalFail::BadBasePoolOffset {
                            local_bpo,
                            super_local_bpo: super_pool.pool().get_local_bpo(),
                        }));
                    }
                }

                type_pool.add_block(Block {
                    block,
                    bpo: local_bpo,
                    static_count: instances,
                    dynamic_count: instances,
                });
                let tmp = type_pool.get_global_static_count() + instances;
                type_pool.set_global_static_count(tmp);
            }
            let field_declarations = reader.read_v64()? as usize;
            block_local_pools.push((type_pool, field_declarations));
        }

        // TODO resize stuff ...
        debug!(target: "SkillParsing", "~Resize Pools~");
        for (ref pool, ref _field_count) in block_local_pools.iter() {
            let mut pool = pool.borrow_mut();
            let mut pool = pool.pool_mut();

            if pool.get_local_dynamic_count() != 0 {
                let tmp = pool.get_global_cached_count() + pool.get_local_dynamic_count();
                pool.set_global_cached_count(tmp);

                if let Some(super_pool) = pool.get_super() {
                    let mut super_pool = super_pool.upgrade().unwrap();
                    let mut super_pool = super_pool.borrow_mut();
                    let mut super_pool = super_pool.pool_mut();

                    let delta = super_pool.get_local_static_count() as i64
                        - (pool.get_local_bpo() as i64 - super_pool.get_local_bpo() as i64);

                    debug!(target: "SkillParsing", "~~Resize delta:{}", delta);
                    if delta > 0 {
                        let tmp = super_pool.get_global_static_count() - delta as usize;
                        super_pool.set_global_static_count(tmp);
                        let tmp = super_pool.get_local_static_count() - delta as usize;
                        super_pool.set_local_static_count(tmp);
                    }
                }
            }
        }

        debug!(target: "SkillParsing", "~TypeFieldMetaData~");
        let mut data_start = 0;

        for (pool, field_count) in block_local_pools {
            let mut field_id_limit = 1 + pool.borrow().pool().fields().len();

            debug!(
                target: "SkillParsing",
                "~~FieldMetaData for type: {} ID:{:?} Fields:{:?} Limit:{:?}",
                pool.borrow().pool().name().as_str(),
                pool.borrow().pool().get_type_id(),
                field_count,
                field_id_limit,
            );

            for _ in 0..field_count {
                let field_id = reader.read_v64()?; // field index

                if field_id <= 0 || field_id_limit < field_id as usize {
                    return Err(SkillFail::internal(InternalFail::BadFieldID {
                        previous: field_id_limit,
                        current: field_id as usize,
                    }));
                }

                if field_id as usize == field_id_limit {
                    field_id_limit += 1;
                    let field_name_id = reader.read_v64()? as usize; // field name id

                    debug!(target: "SkillParsing", "~~~Field id: {:?}", field_id);
                    debug!(target: "SkillParsing", "~~~Field name id: {:?}", field_name_id);

                    let field_name =
                        if let Some(field_name) = string_pool.borrow().get(field_name_id)? {
                            field_name
                        } else {
                            return Err(SkillFail::internal(InternalFail::TypeOrFieldNameNull));
                        };

                    debug!(target: "SkillParsing", "~~~Field name: {}", field_name);

                    //TODO add from for the enum and use that to match and throw an error?
                    let field_type = reader.read_field_type(&self.pools)?;

                    let field_restrictions = reader.read_v64()?; // restrictions
                    debug!(target: "SkillParsing", "~~~FieldRestrictions: {:?}", field_restrictions);

                    for restriction in 0..field_restrictions {
                        // TODO call real function / match
                        let restriction_type = reader.read_v64()?; // restriction type

                        debug!(
                            target: "SkillParsing",
                            "~~~~FieldRestriction: #{:?} as {:?}",
                            restriction,
                            restriction_type
                        );

                        match restriction_type {
                            0x0 => Ok(()), // Non Null
                            0x1 => {
                                //Default
                                field_type.read(reader)
                            }
                            0x3 => match field_type {
                                // Range
                                FieldType::BuildIn(BuildInType::Ti8) => {
                                    let min = reader.read_i8()?;
                                    let max = reader.read_i8()?;
                                    Ok(())
                                }
                                FieldType::BuildIn(BuildInType::Ti16) => {
                                    let min = reader.read_i16()?;
                                    let max = reader.read_i16()?;
                                    Ok(())
                                }
                                FieldType::BuildIn(BuildInType::Ti32) => {
                                    let min = reader.read_i32()?;
                                    let max = reader.read_i32()?;
                                    Ok(())
                                }
                                FieldType::BuildIn(BuildInType::Ti64) => {
                                    let min = reader.read_i64()?;
                                    let max = reader.read_i64()?;
                                    Ok(())
                                }
                                FieldType::BuildIn(BuildInType::Tv64) => {
                                    let min = reader.read_v64()?;
                                    let max = reader.read_v64()?;
                                    Ok(())
                                }
                                FieldType::BuildIn(BuildInType::Tf32) => {
                                    let min = reader.read_f32()?;
                                    let max = reader.read_f32()?;
                                    Ok(())
                                }
                                FieldType::BuildIn(BuildInType::Tf64) => {
                                    let min = reader.read_f64()?;
                                    let max = reader.read_f64()?;
                                    Ok(())
                                }
                                _ => Err(SkillFail::internal(InternalFail::BadRangeRestriction)),
                            },
                            0x5 => {
                                //Coding
                                let coding_str_index = reader.read_v64()?;
                                string_pool.borrow().get(coding_str_index as usize)?;
                                Ok(())
                            }
                            0x7 => Ok(()), // Constant LengthPointer
                            0x9 => {
                                // One of
                                for _ in 0..(reader.read_v64()? as u64) {
                                    let one_of_type_index = reader.read_v64()?;
                                }
                                Ok(())
                            }
                            i => {
                                error!(target: "SkillParsing", "Unknown field restiction: {:?}", i);
                                Err(SkillFail::internal(InternalFail::UnknownFieldRestriction {
                                    id: i as usize,
                                }))
                            }
                        }?;
                    }
                    let data_end = reader.read_v64()? as usize;

                    debug!(
                        target: "SkillParsing", "~~~Add Field:{} start:{:?} end:{:?}",
                        field_name.clone(),
                        data_start,
                        data_end
                    );
                    {
                        let string_pool = string_pool.borrow();
                        let mut pool = pool.borrow_mut();
                        let mut pool = pool.pool_mut();
                        let tmp_count = pool.get_global_cached_count();
                        let tmp_blocks = pool.blocks().len();
                        pool.add_field(
                            &string_pool,
                            field_id as usize,
                            field_name,
                            field_type,
                            FieldChunk::from(DeclarationFieldChunk {
                                begin: data_start,
                                end: data_end,
                                count: tmp_count,
                                appearance: BlockIndex::from(tmp_blocks),
                            }),
                        )?;
                    }
                    data_start = data_end;
                } else {
                    let data_end = reader.read_v64()? as usize;

                    debug!(
                        target: "SkillParsing", "~~~Add Field Chunk:{} start:{:?} end:{:?}",
                        field_id,
                        data_start,
                        data_end
                    );
                    {
                        let mut pool = pool.borrow_mut();
                        let mut pool = pool.pool_mut();
                        let tmp_count = pool.get_local_dynamic_count();
                        let tmp_bpo = pool.get_local_bpo();
                        pool.add_chunk_to(
                            field_id as usize,
                            FieldChunk::from(ContinuationFieldChunk {
                                begin: data_start,
                                end: data_end,
                                count: tmp_count,
                                bpo: tmp_bpo,
                            }),
                        )?;
                    }
                    data_start = data_end;
                }
            }
        }
        field_data.push(reader.jump(data_start));
        debug!(target: "SkillParsing", "~Block End~");
        Ok(())
    }

    pub(crate) fn add(&mut self, pool: Rc<RefCell<PoolProxy>>) {
        self.pools.push(pool);
    }

    pub(crate) fn len(&self) -> usize {
        self.pools.len()
    }

    pub(crate) fn initialize(
        &self,
        strings: &StringBlock,
        reader: &Vec<FileReader>,
    ) -> Result<(), SkillFail> {
        for pool in self.pools.iter() {
            debug!(
                target: "SkillParsing",
                "Initializing Pool {}",
                pool.borrow().pool().name().as_str(),
            );
            pool.borrow()
                .pool()
                .initialize(reader, strings, &self.pools)?;
        }
        Ok(())
    }

    pub(crate) fn compress(&mut self) -> Result<Vec<usize>, SkillFail> {
        let mut local_bpos = Vec::new();
        local_bpos.reserve(self.pools.len());
        for _ in 0..self.pools.len() {
            local_bpos.push(0);
        }

        for p in self.pools.iter() {
            if p.borrow().pool().is_base() {
                let vec: Rc<RefCell<Vec<Ptr<SkillObject>>>> = Rc::new(RefCell::new(Vec::new()));
                {
                    let mut vec = vec.borrow_mut();
                    vec.reserve(p.borrow().pool().get_global_cached_count());
                    for _ in 0..p.borrow().pool().get_global_cached_count() {
                        // TODO replace with garbage object
                        vec.push(Ptr::new(foreign::Foreign::new(0, 0)));
                    }

                    let mut id = 1;
                    for i in type_order_instances::Iter::new(p.clone())? {
                        if !i.borrow().to_delete() {
                            i.borrow().set_skill_id(id)?;
                            vec[id - 1] = i;
                            id += 1;
                        }
                    }
                }

                // TODO does the reorder work?
                let mut next = 0;
                for p in type_hierarchy::Iter::new(p.clone())? {
                    local_bpos[p.borrow().pool().get_type_id() - 32] = next;
                    next += p.borrow().pool().static_size() - p.borrow().pool().deleted_instances();
                    p.borrow_mut().pool_mut().compress_field_chunks(&local_bpos);
                }

                for p in type_hierarchy::Iter::new(p.clone())? {
                    p.borrow_mut()
                        .pool_mut()
                        .update_after_compress(&local_bpos, vec.clone());
                }
            }
        }
        Ok(local_bpos)
    }

    pub(crate) fn write_block(
        &self,
        writer: &mut FileWriter,
        local_bpos: &Vec<usize>,
    ) -> Result<(), SkillFail> {
        // How many types
        debug!(
            target: "SkillWriting",
            "~Type Block Start~"
        );
        debug!(
            target: "SkillWriting",
            "~Write {} types",
            self.pools.len(),
        );
        writer.write_v64(self.pools.len() as i64)?;

        debug!(
            target: "SkillWriting",
            "~~Write Type Meta Data",
        );
        for p in self.pools.iter() {
            p.borrow().pool().write_type_meta(writer, &local_bpos)?;
        }

        debug!(
            target: "SkillWriting",
            "~~Write Type Field Meta Data",
        );
        let mut offset = 0;
        for p in self.pools.iter() {
            offset = p.borrow().pool().write_field_meta(
                writer,
                dynamic_instances::Iter::new(p.clone())?,
                offset,
            )?;
        }

        debug!(
            target: "SkillWriting",
            "~~Write Type Field Data for #{} pools",
            self.pools.len()
        );
        let mut writer = writer.jump(offset)?;
        for p in self.pools.iter() {
            p.borrow()
                .pool()
                .write_field_data(&mut writer, dynamic_instances::Iter::new(p.clone())?)?;
        }

        debug!(
            target: "SkillWriting",
            "~Type Block End~"
        );
        Ok(())
    }

    pub(crate) fn set_invariant(&self, invariant: bool) {
        for p in self.pools.iter() {
            p.borrow_mut().pool_mut().set_invariant(invariant);
        }
    }
}
