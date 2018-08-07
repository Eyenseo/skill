use common::error::*;
use common::internal::*;
use common::io::*;
use common::iterator::dynamic_data;
use common::*;

use std::cell::RefCell;
use std::rc::Rc;

use std::collections::HashMap;
use std::collections::HashSet;

// TODO Rename - this is a unknown
pub struct LazyFieldDeclaration {
    name: Rc<SkillString>,
    field_id: usize,
    chunks: Vec<FieldChunk>,
    field_type: FieldType,
}

impl LazyFieldDeclaration {
    pub fn new(
        name: Rc<SkillString>,
        field_id: usize,
        field_type: FieldType,
    ) -> LazyFieldDeclaration {
        LazyFieldDeclaration {
            name,
            field_id,
            chunks: Vec::new(),
            field_type,
        }
    }

    // TODO this should also produce an offset for fields that do not contain user types / strings
    fn read_undefined_field(
        field: &FieldType,
        reader: &mut FileReader,
        string_block: &StringBlock,
        type_pools: &Vec<Rc<RefCell<InstancePool>>>,
    ) -> Result<UndefinedFieldData, SkillFail> {
        Ok(match field {
            FieldType::BuildIn(ref field) => match field {
                BuildInType::ConstTi8 => UndefinedFieldData::I8(reader.read_i8()?),
                BuildInType::ConstTi16 => UndefinedFieldData::I16(reader.read_i16()?),
                BuildInType::ConstTi32 => UndefinedFieldData::I32(reader.read_i32()?),
                BuildInType::ConstTi64 => UndefinedFieldData::I64(reader.read_i64()?),
                BuildInType::ConstTv64 => UndefinedFieldData::I64(reader.read_v64()?),
                BuildInType::Tbool => UndefinedFieldData::Bool(reader.read_bool()?),
                BuildInType::Ti8 => UndefinedFieldData::I8(reader.read_i8()?),
                BuildInType::Ti16 => UndefinedFieldData::I16(reader.read_i16()?),
                BuildInType::Ti32 => UndefinedFieldData::I32(reader.read_i32()?),
                BuildInType::Ti64 => UndefinedFieldData::I64(reader.read_i64()?),
                BuildInType::Tv64 => UndefinedFieldData::I64(reader.read_v64()?),
                BuildInType::Tf32 => UndefinedFieldData::F32(reader.read_f32()?),
                BuildInType::Tf64 => UndefinedFieldData::F64(reader.read_f64()?),
                BuildInType::Tannotation => UndefinedFieldData::User({
                    let pool = reader.read_v64()? as usize;
                    let object = reader.read_v64()? as usize;
                    if pool != 0 && object != 0 {
                        Some(type_pools[pool - 1].borrow().read_object(object)?)
                    } else {
                        None
                    }
                }),
                BuildInType::Tstring => {
                    UndefinedFieldData::String(string_block.get(reader.read_v64()? as usize)?)
                }
                BuildInType::ConstTarray(length, box_v) => UndefinedFieldData::Array({
                    let mut arr = Vec::with_capacity(*length as usize);
                    for i in 0..*length as usize {
                        arr[i] = LazyFieldDeclaration::read_undefined_field(
                            &*box_v,
                            reader,
                            string_block,
                            type_pools,
                        )?;
                    }
                    arr
                }),
                BuildInType::Tarray(box_v) => UndefinedFieldData::Array({
                    let elements = reader.read_v64()? as usize;
                    let mut vec = Vec::with_capacity(elements);
                    for _ in 0..elements {
                        vec.push(LazyFieldDeclaration::read_undefined_field(
                            &*box_v,
                            reader,
                            string_block,
                            type_pools,
                        )?);
                    }
                    vec
                }),
                BuildInType::Tlist(box_v) => UndefinedFieldData::Array({
                    let elements = reader.read_v64()? as usize;
                    let mut vec = Vec::with_capacity(elements);
                    for _ in 0..elements {
                        vec.push(LazyFieldDeclaration::read_undefined_field(
                            &*box_v,
                            reader,
                            string_block,
                            type_pools,
                        )?);
                    }
                    vec
                }),
                BuildInType::Tset(box_v) => UndefinedFieldData::Set({
                    let elements = reader.read_v64()? as usize;
                    let mut set = HashSet::new();
                    set.reserve(elements);
                    for _ in 0..elements {
                        set.insert(LazyFieldDeclaration::read_undefined_field(
                            &*box_v,
                            reader,
                            string_block,
                            type_pools,
                        )?);
                    }
                    set
                }),
                BuildInType::Tmap(key_box_v, box_v) => UndefinedFieldData::Map({
                    let elements = reader.read_v64()? as usize;
                    let mut map = HashMap::new();
                    map.reserve(elements);
                    for _ in 0..elements {
                        map.insert(
                            LazyFieldDeclaration::read_undefined_field(
                                &*key_box_v,
                                reader,
                                string_block,
                                type_pools,
                            )?,
                            LazyFieldDeclaration::read_undefined_field(
                                &*box_v,
                                reader,
                                string_block,
                                type_pools,
                            )?,
                        );
                    }
                    map
                }),
            },
            FieldType::User(ref pool) => UndefinedFieldData::User({
                let object = reader.read_v64()? as usize;
                if object != 0 {
                    Some(pool.borrow().read_object(object)?)
                } else {
                    None
                }
            }),
        })
    }
}

impl FieldDeclaration for LazyFieldDeclaration {
    fn read(
        &self,
        block_reader: &Vec<FileReader>,
        string_block: &StringBlock,
        blocks: &Vec<Block>,
        type_pools: &Vec<Rc<RefCell<InstancePool>>>,
        instances: &[Ptr<SkillObject>],
    ) -> Result<(), SkillFail> {
        Ok(())
    }

    fn deserialize(
        &self,
        block_reader: &Vec<FileReader>,
        string_block: &StringBlock,
        blocks: &Vec<Block>,
        type_pools: &Vec<Rc<RefCell<InstancePool>>>,
        instances: &[Ptr<SkillObject>],
    ) -> Result<(), SkillFail> {
        let mut block_index = BlockIndex::from(0);

        for chunk in self.chunks.iter() {
            match chunk {
                FieldChunk::Declaration(chunk) => {
                    block_index += chunk.appearance - 1;

                    let block = &blocks[block_index.block];
                    let mut reader =
                        block_reader[block.block.block].rel_view(chunk.begin, chunk.end);
                    block_index += 1;

                    if chunk.count > 0 {
                        for block in blocks.iter().take(chunk.appearance.block) {
                            let mut o = 0;

                            for obj in instances.iter().skip(block.bpo).take(block.dynamic_count) {
                                info!(
                                        target: "SkillParsing",
                                        "Block:{:?} Object:{}",
                                        block,
                                        o + block.bpo,
                                    );
                                o += 1;
                                match obj.nucast::<UndefinedObjectT>() {
                                    Some(obj) => {
                                        let mut obj = obj.borrow_mut();
                                        obj.undefined_fields_mut().push(
                                            LazyFieldDeclaration::read_undefined_field(
                                                &self.field_type,
                                                &mut reader,
                                                string_block,
                                                type_pools,
                                            )?,
                                        )
                                    }
                                    None => return Err(SkillFail::internal(InternalFail::BadCast)),
                                }
                            }
                        }
                    }
                }
                FieldChunk::Continuation(chunk) => {
                    let block = &blocks[block_index.block];
                    let mut reader =
                        block_reader[block.block.block].rel_view(chunk.begin, chunk.end);
                    block_index += 1;

                    if chunk.count > 0 {
                        let mut o = 0;
                        for obj in instances.iter().skip(chunk.bpo).take(chunk.count) {
                            info!(
                                    target: "SkillParsing",
                                    "Block:{:?} Object:{}",
                                    block,
                                    o + chunk.bpo,
                                );
                            o += 1;

                            match obj.nucast::<UndefinedObjectT>() {
                                Some(obj) => {
                                    let mut obj = obj.borrow_mut();
                                    obj.undefined_fields_mut().push(
                                        LazyFieldDeclaration::read_undefined_field(
                                            &self.field_type,
                                            &mut reader,
                                            string_block,
                                            type_pools,
                                        )?,
                                    )
                                }
                                None => return Err(SkillFail::internal(InternalFail::BadCast)),
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
    fn add_chunk(&mut self, chunk: FieldChunk) {
        self.chunks.push(chunk);
    }
    fn name(&self) -> &Rc<SkillString> {
        &self.name
    }
    fn field_id(&self) -> usize {
        self.field_id
    }
    fn compress_chunks(&mut self, total_count: usize) {
        self.chunks = Vec::with_capacity(1);
        self.chunks
            .push(FieldChunk::Declaration(DeclarationFieldChunk {
                begin: 0,
                end: 0,
                count: total_count,
                appearance: BlockIndex::from(1),
            }));
    }
    fn offset(&self, iter: dynamic_data::Iter) -> usize {
        unimplemented!();
    }
    fn write_meta(
        &mut self,
        writer: &mut FileWriter,
        iter: dynamic_data::Iter,
        offset: usize,
    ) -> Result<usize, SkillFail> {
        writer.write_v64(self.field_id as i64)?;
        writer.write_v64(self.name.get_skill_id() as i64)?;
        writer.write_field_type(&self.field_type)?;
        writer.write_i8(0)?; // TODO write restrictions
        let end_offset = offset + self.offset(iter);
        writer.write_v64(end_offset as i64)?;

        match self.chunks.first_mut().unwrap() {
            FieldChunk::Declaration(ref mut dec) => {
                dec.begin = offset;
                dec.end = end_offset;
                Ok(())
            }
            _ => Err(SkillFail::internal(InternalFail::BadChunk)),
        }?;

        Ok(end_offset)
    }
    fn write_data(
        &self,
        writer: &mut FileWriter,
        iter: dynamic_data::Iter,
    ) -> Result<(), SkillFail> {
        unimplemented!();
    }
}
