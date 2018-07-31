use common::error::*;
use common::internal::{InstancePool, ObjectReader, SkillObject, UndefinedObject};
use common::io::{
    Block, BlockIndex, BuildInType, ContinuationFieldChunk, DeclarationFieldChunk, FieldChunk,
    FieldType, FileReader, FileWriter,
};
use common::iterator::*;
use common::PoolMaker;
use common::Ptr;
use common::StringBlock;

use std::cell::RefCell;
use std::rc::Rc;

#[derive(Default)]
// TODO rename
pub struct TypeBlock {
    pools: Vec<Rc<RefCell<InstancePool>>>,
}

impl TypeBlock {
    pub fn new() -> TypeBlock {
        TypeBlock { pools: Vec::new() }
    }

    pub fn read_type_block(
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

        info!(target: "SkillParsing", "~Block Start~");
        let type_amount = reader.read_v64()? as usize;
        info!(target: "SkillParsing", "~Types: {:?}", type_amount);
        self.pools.reserve(type_amount);

        info!(target: "SkillParsing", "~TypeData~");
        for _ in 0..type_amount {
            let type_name_index = reader.read_v64()? as u64;
            let type_name = string_pool.borrow().get(type_name_index as usize)?;
            info!(target: "SkillParsing", "~~TypeName: {}", type_name);

            if seen_types.contains(&type_name_index) {
                return Err(SkillFail::internal(InternalFail::RedefinitionOfType {
                    name: type_name.as_str().to_owned(),
                }));
            }
            seen_types.push(type_name_index);

            let instances = reader.read_v64()? as usize; // amount of instances
            info!(target: "SkillParsing", "~~TypeInstances: {:?}", instances);

            let mut type_pool = if let Some(pool) = pool_maker.get_pool(type_name_index as usize) {
                pool
            } else {
                let type_id = self.pools.len() + 32;

                info!(target: "SkillParsing", "~~New Type:{:?}", type_id);
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
                            }))
                        }
                    }
                }

                let super_type = reader.read_v64()?; // super type index? id?
                let super_pool = if super_type as usize > self.pools.len() {
                    return Err(SkillFail::internal(InternalFail::UnknownType {
                        id: super_type as usize,
                    }));
                } else if super_type != 0 {
                    info!(
                        target: "SkillParsing",
                        "~~Add Super Type:{:?} for:{:?}",
                        self.pools[(super_type - 1) as usize].borrow().get_type_id(),
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
                let mut type_pool = type_pool.borrow_mut();

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
                let mut local_bpo = if let Some(base_pool) = type_pool.borrow().get_base() {
                    base_pool.borrow().get_global_cached_count()
                } else {
                    type_pool.borrow().get_global_cached_count()
                };
                // NOTE order prevents borrow panic
                let mut type_pool = type_pool.borrow_mut();

                if let Some(super_pool) = type_pool.get_super() {
                    let super_pool = super_pool.borrow();

                    if instances != 0 {
                        local_bpo += reader.read_v64()? as usize;
                    } else {
                        local_bpo += super_pool.get_local_bpo();
                    }

                    if local_bpo < super_pool.get_local_bpo()
                        || super_pool.get_local_bpo() + super_pool.get_local_dynamic_count()
                            < local_bpo
                    {
                        return Err(SkillFail::internal(InternalFail::BadBasePoolOffset {
                            local_bpo,
                            super_local_bpo: super_pool.get_local_bpo(),
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
        info!(target: "SkillParsing", "~Resize Pools~");
        for (ref pool, ref _field_count) in block_local_pools.iter() {
            let mut pool = pool.borrow_mut();

            if pool.get_local_dynamic_count() != 0 {
                let tmp = pool.get_global_cached_count() + pool.get_local_dynamic_count();
                pool.set_global_cached_count(tmp);

                if let Some(super_pool) = pool.get_super() {
                    let mut super_pool = super_pool.borrow_mut();

                    let delta = super_pool.get_local_static_count() as i64
                        - (pool.get_local_bpo() as i64 - super_pool.get_local_bpo() as i64);

                    info!(target: "SkillParsing", "~~Resize delta:{}", delta);
                    if delta > 0 {
                        let tmp = super_pool.get_global_static_count() - delta as usize;
                        super_pool.set_global_static_count(tmp);
                        let tmp = super_pool.get_local_static_count() - delta as usize;
                        super_pool.set_local_static_count(tmp);
                    }
                }
            }
        }

        info!(target: "SkillParsing", "~TypeFieldMetaData~");
        let mut data_start = 0;

        for (pool, field_count) in block_local_pools {
            let mut field_id_limit = 1 + pool.borrow().field_amount();

            info!(
                target: "SkillParsing",
                "~~FieldMetaData for type: {} ID:{:?} Fields:{:?} Limit:{:?}",
                pool.borrow().name().as_str(),
                pool.borrow().get_type_id(),
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

                    info!(target: "SkillParsing", "~~~Field id: {:?}", field_id);
                    info!(target: "SkillParsing", "~~~Field name id: {:?}", field_name_id);

                    let field_name = string_pool.borrow().get(field_name_id)?;
                    info!(target: "SkillParsing", "~~~Field name: {}", field_name);

                    //TODO add from for the enum and use that to match and throw an error?
                    let field_type = reader.read_field_type(&self.pools)?;

                    let field_restrictions = reader.read_v64()?; // restrictions
                    info!(target: "SkillParsing", "~~~FieldRestrictions: {:?}", field_restrictions);
                    match field_type {
                        FieldType::User(_) => (), // NOTE this might be wrong
                        _ => {
                            for restriction in 0..field_restrictions {
                                // TODO call real function / match
                                let restriction_type = reader.read_v64()?; // restriction type

                                info!(
                                    target: "SkillParsing",
                                    "~~~~FieldRestriction: #{:?} as {:?}",
                                    restriction,
                                    restriction_type
                                );

                                match restriction_type {
                                    0x0 => (), // Non Null
                                    0x1 => {
                                        //Default
                                        match field_type {
                                            FieldType::BuildIn(BuildInType::Tannotation)
                                            | FieldType::User(_) => {
                                                // TODO This is the id of the user object that is used
                                                // as defualt value / initialization
                                                reader.read_v64()?;
                                            }
                                            _ => {
                                                // TODO this reads a v64 that is also a id to a object,
                                                // that is strored in a Pool ... and is then thrown
                                                // away in the c++ implementatino
                                                reader.read_v64()?;
                                            }
                                        }
                                    }
                                    0x3 => match field_type {
                                        // Range
                                        FieldType::BuildIn(BuildInType::Ti8) => {
                                            let min = reader.read_i8();
                                            let max = reader.read_i8();
                                        }
                                        FieldType::BuildIn(BuildInType::Ti16) => {
                                            let min = reader.read_i16();
                                            let max = reader.read_i16();
                                        }
                                        FieldType::BuildIn(BuildInType::Ti32) => {
                                            let min = reader.read_i32();
                                            let max = reader.read_i32();
                                        }
                                        FieldType::BuildIn(BuildInType::Ti64) => {
                                            let min = reader.read_i64();
                                            let max = reader.read_i64();
                                        }
                                        FieldType::BuildIn(BuildInType::Tv64) => {
                                            let min = reader.read_v64();
                                            let max = reader.read_v64();
                                        }
                                        FieldType::BuildIn(BuildInType::Tf32) => {
                                            let min = reader.read_f32();
                                            let max = reader.read_f32();
                                        }
                                        FieldType::BuildIn(BuildInType::Tf64) => {
                                            let min = reader.read_f64();
                                            let max = reader.read_f64();
                                        }
                                        _ => {
                                            return Err(SkillFail::internal(
                                                InternalFail::BadRangeRestriction,
                                            ))
                                        }
                                    },
                                    0x5 => {
                                        //Coding
                                        let coding_str_index = reader.read_v64()?;
                                        string_pool.borrow().get(coding_str_index as usize)?;
                                    }
                                    0x7 => (), // Constant LengthPointer
                                    0x9 => {
                                        // One of
                                        for _ in 0..(reader.read_v64()? as u64) {
                                            let one_of_type_index = reader.read_v64()?;
                                        }
                                    }
                                    i => {
                                        error!(target: "SkillParsing", "Unknown field restiction: {:?}", i);
                                        return Err(SkillFail::internal(
                                            InternalFail::UnknownFieldRestriction {
                                                id: i as usize,
                                            },
                                        ));
                                    }
                                }
                            }
                        }
                    }
                    let data_end = reader.read_v64()? as usize;

                    info!(
                        target: "SkillParsing", "~~~Add Field:{} start:{:?} end:{:?}",
                        field_name.clone(),
                        data_start,
                        data_end
                    );
                    {
                        let mut pool = pool.borrow_mut();
                        let tmp_count = pool.get_global_cached_count();
                        let tmp_blocks = pool.blocks().len();
                        pool.add_field(
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

                    info!(
                        target: "SkillParsing", "~~~Add Field Chunk:{} start:{:?} end:{:?}",
                        field_id,
                        data_start,
                        data_end
                    );
                    {
                        let mut pool = pool.borrow_mut();
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
        info!(target: "SkillParsing", "~Block End~");
        Ok(())
    }

    pub fn add(&mut self, pool: Rc<RefCell<InstancePool>>) {
        self.pools.push(pool);
    }

    pub fn len(&self) -> usize {
        self.pools.len()
    }

    pub fn initialize(
        &self,
        strings: &StringBlock,
        reader: &Vec<FileReader>,
    ) -> Result<(), SkillFail> {
        for pool in self.pools.iter() {
            pool.borrow().initialize(reader, strings, &self.pools)?;
        }
        Ok(())
    }

    pub fn compress(&mut self) -> Result<Vec<usize>, SkillFail> {
        let mut local_bpos = Vec::new();
        local_bpos.reserve(self.pools.len());
        for _ in 0..self.pools.len() {
            local_bpos.push(0);
        }

        for p in self.pools.iter() {
            if p.borrow().is_base() {
                let vec: Rc<RefCell<Vec<Ptr<SkillObject>>>> = Rc::new(RefCell::new(Vec::new()));
                {
                    let mut vec = vec.borrow_mut();
                    vec.reserve(p.borrow().static_size());
                    for _ in 0..p.borrow().static_size() {
                        // TODO replace with garbage object
                        vec.push(Ptr::new(UndefinedObject::new(0)));
                    }

                    let mut id = 1;
                    for i in type_order::Iter::new(p.clone())? {
                        i.borrow().set_skill_id(id)?;
                        vec[id - 1] = i;
                        id += 1;
                    }
                }

                // TODO does the reorder work?
                let mut next = 0;
                for p in type_hierarchy::Iter::new(p.clone())? {
                    local_bpos[p.borrow().get_type_id() - 32] = next;
                    next += p.borrow().static_size() - p.borrow().deleted();
                    p.borrow_mut().compress_field_chunks(&local_bpos);
                }

                for p in type_hierarchy::Iter::new(p.clone())? {
                    p.borrow_mut()
                        .update_after_compress(&local_bpos, vec.clone());
                }
            }
        }
        Ok(local_bpos)
    }

    pub fn write_block(
        &self,
        writer: &mut FileWriter,
        local_bpos: &Vec<usize>,
    ) -> Result<(), SkillFail> {
        // How many types
        info!(
            target: "SkillWriting",
            "~Type Block Start~"
        );
        info!(
            target: "SkillWriting",
            "~Write {} types",
            self.pools.len(),
        );
        writer.write_v64(self.pools.len() as i64)?;

        info!(
            target: "SkillWriting",
            "~~Write Type Meta Data",
        );
        // Write Type meta data
        for p in self.pools.iter() {
            p.borrow().write_type_meta(writer, &local_bpos)?;
        }

        info!(
            target: "SkillWriting",
            "~~Write Type Field Meta Data",
        );
        // Write Field meta data
        let mut offset = 0;
        for p in self.pools.iter() {
            offset =
                p.borrow()
                    .write_field_meta(writer, static_data::Iter::new(p.clone()), offset)?;
        }

        info!(
            target: "SkillWriting",
            "~~Write Type Field Data",
        );
        // Write Field data
        for p in self.pools.iter() {
            p.borrow()
                .write_field_data(writer, static_data::Iter::new(p.clone()))?;
        }

        info!(
            target: "SkillWriting",
            "~Type Block End~"
        );

        Ok(())
    }

    pub fn set_invariant(&self, invariant: bool) {
        for p in self.pools.iter() {
            p.borrow_mut().set_invariant(invariant);
        }
    }
}
