use common::internal::{InstancePool, ObjectReader};
use common::io::{
    Block, BlockIndex, BuildInType, ContinuationFieldChunk, DeclarationFieldChunk, FieldChunk,
    FieldType, FileReader, Offset,
};
use common::PoolMaker;
use common::SkillError;
use common::StringBlock;

use std::cell::RefCell;
use std::rc::Rc;

#[derive(Default)]
pub struct TypeBlock {
    pools: Vec<u64>,
}

impl TypeBlock {
    pub fn new() -> TypeBlock {
        TypeBlock { pools: Vec::new() }
    }
    pub fn reserve(&mut self, size: usize) {
        self.pools.reserve(size);
    }
    pub fn extend(&mut self, size: usize) {
        let reserve = self.pools.len();
        self.reserve(reserve + size);
    }
    pub fn add(&mut self, i: u64) {
        self.pools.push(i);
    }
    pub fn has(&self, i: u64) -> bool {
        self.pools.contains(&i)
    }

    fn read_field_type(
        &self,
        reader: &mut FileReader,
        type_pools: &Vec<Rc<RefCell<InstancePool>>>,
    ) -> Result<FieldType, SkillError> {
        let field_type = reader.read_v64()?; // type of field

        //TODO add from for the enum and use that to match and throw an error?
        Ok(match field_type {
            0x0 => {
                reader.read_i8()?;
                info!(target: "SkillParsing", "~~~~FieldType = const i8");
                FieldType::BuildIn(BuildInType::ConstTi8)
            }
            0x1 => {
                reader.read_i16()?;
                info!(target: "SkillParsing", "~~~~FieldType = const i16");
                FieldType::BuildIn(BuildInType::ConstTi16)
            }
            0x2 => {
                reader.read_i32()?;
                info!(target: "SkillParsing", "~~~~FieldType = const i32");
                FieldType::BuildIn(BuildInType::ConstTi32)
            }
            0x3 => {
                reader.read_i64()?;
                info!(target: "SkillParsing", "~~~~FieldType = const i64");
                FieldType::BuildIn(BuildInType::ConstTi64)
            }
            0x4 => {
                reader.read_v64()?;
                info!(target: "SkillParsing", "~~~~FieldType = const v64");
                FieldType::BuildIn(BuildInType::ConstTv64)
            }
            0x5 => {
                info!(target: "SkillParsing", "~~~~FieldType = annotation");
                FieldType::BuildIn(BuildInType::Tannotation)
            }
            0x6 => {
                info!(target: "SkillParsing", "~~~~FieldType = bool");
                FieldType::BuildIn(BuildInType::Tbool)
            }
            0x7 => {
                info!(target: "SkillParsing", "~~~~FieldType = i8");
                FieldType::BuildIn(BuildInType::Ti8)
            }
            0x8 => {
                info!(target: "SkillParsing", "~~~~FieldType = i16");
                FieldType::BuildIn(BuildInType::Ti16)
            }
            0x9 => {
                info!(target: "SkillParsing", "~~~~FieldType = i32");
                FieldType::BuildIn(BuildInType::Ti32)
            }
            0xA => {
                info!(target: "SkillParsing", "~~~~FieldType = i64");
                FieldType::BuildIn(BuildInType::Ti64)
            }
            0xB => {
                info!(target: "SkillParsing", "~~~~FieldType = v64");
                FieldType::BuildIn(BuildInType::Tv64)
            }
            0xC => {
                info!(target: "SkillParsing", "~~~~FieldType = f32");
                FieldType::BuildIn(BuildInType::Tf32)
            }
            0xD => {
                info!(target: "SkillParsing", "~~~~FieldType = f64");
                FieldType::BuildIn(BuildInType::Tf64)
            }
            0xE => {
                info!(target: "SkillParsing", "~~~~FieldType = string");
                FieldType::BuildIn(BuildInType::Tstring)
            }
            0xF => {
                let length = reader.read_v64()? as u64;
                info!(target: "SkillParsing", "~~~~FieldType = const array length: {:?}", length);
                FieldType::BuildIn(BuildInType::ConstTarray(
                    length,
                    Box::new(self.read_field_type(reader, type_pools)?),
                ))
            }
            0x11 => {
                info!(target: "SkillParsing", "~~~~FieldType = varray");
                FieldType::BuildIn(BuildInType::Tarray(Box::new(
                    self.read_field_type(reader, type_pools)?,
                )))
            }
            0x12 => {
                info!(target: "SkillParsing", "~~~~FieldType = list");
                FieldType::BuildIn(BuildInType::Tlist(Box::new(
                    self.read_field_type(reader, type_pools)?,
                )))
            }
            0x13 => {
                info!(target: "SkillParsing", "~~~~FieldType = set");
                FieldType::BuildIn(BuildInType::Tset(Box::new(
                    self.read_field_type(reader, type_pools)?,
                )))
            }
            0x14 => {
                info!(target: "SkillParsing", "~~~~FieldType = map");
                FieldType::BuildIn(BuildInType::Tmap(
                    Box::new(self.read_field_type(reader, type_pools)?),
                    Box::new(self.read_field_type(reader, type_pools)?),
                ))
            }
            user => {
                if user < 32 {
                    // TODO check the current upper limit of known types
                    panic!("Invalid UserType ID {:?}", user);
                }
                info!(target: "SkillParsing", "~~~~FieldType = User ID {:?}", user);
                // FIXME this is wrong!
                // What we want is to put the pool in here - user - 32 to access the vector that
                // stores all pools -> there has to be a vector that stores all pools ...
                FieldType::User(type_pools[user as usize - 32].clone(), user as usize)
            }
        })
    }

    pub fn read_type_block(
        &mut self,
        block: BlockIndex,
        reader: &mut FileReader,
        pool_maker: &mut PoolMaker,
        string_pool: &Rc<RefCell<StringBlock>>,
        type_pools: &mut Vec<Rc<RefCell<InstancePool>>>,
        field_data: &mut Vec<FileReader>,
    ) -> Result<(), SkillError> {
        let mut block_local_pools = Vec::new();
        let mut block_local_id_limit = 0;
        let mut seen_types = Vec::new();

        info!(target: "SkillParsing", "~Block Start~");
        let type_amount = reader.read_v64()? as usize;
        info!(target: "SkillParsing", "~Types: {:?}", type_amount);
        self.extend(type_amount);

        info!(target: "SkillParsing", "~TypeData~");
        for _ in 0..type_amount {
            let type_name_index = reader.read_v64()? as u64;
            if seen_types.contains(&type_name_index) {
                // TODO This has to use a separate list for each block
                return Err(SkillError::RedefinitionOfType);
            }
            seen_types.push(type_name_index);

            let type_name = string_pool.borrow().get(type_name_index as usize);
            info!(target: "SkillParsing", "~~TypeName: {}", type_name);
            let instances = reader.read_v64()? as usize; // amount of instances
            info!(target: "SkillParsing", "~~TypeInstances: {:?}", instances);

            let mut type_pool = if let Some(pool) = pool_maker.get_pool(type_name_index as usize) {
                pool
            } else {
                let type_id = type_pools.len() + 32;

                info!(target: "SkillParsing", "~~New Type:{:?}", type_pools.len() + 32);
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
                        _ => panic!("Unknown type restriction"),
                    }
                }
                self.add(type_name_index); // FIXME this has to be 'improved'

                let super_type = reader.read_v64()?; // super type index? id?
                let super_pool = if super_type as usize > type_pools.len() {
                    panic!("Unknown Supertype."); // TODO improve message
                } else if super_type != 0 {
                    info!(
                        target: "SkillParsing",
                        "~~Add Super Type:{:?} for:{:?}",
                        type_pools[(super_type - 1) as usize].borrow().get_type_id(),
                        type_id
                    );
                    // TODO check that this is the expected super type
                    Some(type_pools[(super_type - 1) as usize].clone())
                } else {
                    None
                };

                let type_pool = pool_maker.make_pool(
                    type_name_index as usize,
                    &type_name.clone(),
                    type_id as usize,
                    super_pool,
                );
                type_pools.push(type_pool.clone());
                type_pool
            };

            {
                let mut type_pool = type_pool.borrow_mut();

                if block_local_id_limit < type_pool.get_type_id() {
                    block_local_id_limit = type_pool.get_type_id();
                } else {
                    panic!(
                        "Unordered type block; ID:{} < ID:{}",
                        block_local_id_limit,
                        type_pool.get_type_id()
                    );
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
                        panic!(
                            "Found broken base pool offset of:{:?} super lbpo:{:?}",
                            local_bpo,
                            super_pool.get_local_bpo()
                        );
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

                    let dela = pool.get_local_static_count() as i64
                        - (pool.get_local_bpo() as i64 - super_pool.get_local_bpo() as i64);

                    if dela > 0 {
                        let tmp = super_pool.get_global_static_count() - dela as usize;
                        super_pool.set_global_static_count(tmp);
                        let tmp = super_pool.get_local_static_count() - dela as usize;
                        super_pool.set_local_static_count(tmp);
                    }
                }
            }
        }

        info!(target: "SkillParsing", "~TypeFieldMetaData~");
        let mut data_start = Offset::from(0);

        for (pool, field_count) in block_local_pools {
            let mut field_id_limit = 1 + pool.borrow().field_amount();

            let type_name = string_pool
                .borrow()
                .get((pool.borrow().get_type_id() - 31) as usize);
            info!(
                target: "SkillParsing",
                "~~FieldMetaData for type: {} ID:{:?} Fields:{:?} Limit:{:?}",
                type_name,
                pool.borrow().get_type_id(),
                field_count,
                field_id_limit,
            );

            for _ in 0..field_count {
                let field_id = reader.read_v64()?; // field index

                if field_id <= 0 || field_id_limit < field_id as usize {
                    panic!("Illigal field id:{:?}", field_id_limit);
                }

                if field_id as usize == field_id_limit {
                    field_id_limit += 1;
                    let field_name_index = reader.read_v64()? as usize; // field name id

                    info!(target: "SkillParsing", "~~~Field index: {:?}", field_id);
                    info!(target: "SkillParsing", "~~~Field id: {:?}", field_name_index);

                    let field_name = string_pool.borrow().get(field_name_index);
                    info!(target: "SkillParsing", "~~~Field name: {}", field_name);

                    //TODO add from for the enum and use that to match and throw an error?
                    let field_type = self.read_field_type(reader, type_pools)?;

                    let field_restrictions = reader.read_v64()?; // restrictions
                    info!(target: "SkillParsing", "~~~FieldRestrictions: {:?}", field_restrictions);
                    match field_type {
                        FieldType::User(_, _) => (), // NOTE this might be wrong
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
                                            | FieldType::User(_, _) => {
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
                                        _ => panic!("Range restriction on non numeric type"),
                                    },
                                    0x5 => {
                                        //Coding
                                        let coding_str_index = reader.read_v64()?;
                                        string_pool.borrow().get(coding_str_index as usize);
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
                                        panic!("Unknown field restriction")
                                    }
                                }
                            }
                        }
                    }
                    let data_end = Offset::from(reader.read_v64()? as usize);

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
                            &field_name.clone(),
                            field_type,
                            FieldChunk::from(DeclarationFieldChunk {
                                begin: data_start,
                                end: data_end,
                                count: tmp_count,
                                appearance: BlockIndex::from(tmp_blocks),
                            }),
                        );
                    }
                    data_start = data_end;
                } else {
                    let data_end = Offset::from(reader.read_v64()? as usize);

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
                        );
                    }
                    data_start = data_end;
                }
            }
        }
        field_data.push(reader.jump(data_start));
        info!(target: "SkillParsing", "~Block End~");
        Ok(())
    }
}
