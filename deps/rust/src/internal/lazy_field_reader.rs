use common::error::*;
use common::internal::*;
use common::io::*;
use common::iterator::dynamic_data;
use common::*;

use std::cell::RefCell;
use std::rc::Rc;

use std::collections::HashMap;
use std::collections::HashSet;

/// Helper Iterator to reduce code duplication
struct SingleItemIter<'a> {
    item: Option<&'a UndefinedFieldData>,
}

impl<'a> SingleItemIter<'a> {
    fn new(item: &'a UndefinedFieldData) -> SingleItemIter {
        SingleItemIter { item: Some(item) }
    }
}

impl<'a> Iterator for SingleItemIter<'a> {
    type Item = &'a UndefinedFieldData;
    fn next(&mut self) -> Option<&'a UndefinedFieldData> {
        if let Some(item) = self.item {
            Some(item)
        } else {
            None
        }
    }
}

// TODO Rename - this is a unknown
pub struct LazyFieldDeclaration {
    name: Rc<SkillString>,
    field_id: usize,
    undefined_vec_index: usize,
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
            undefined_vec_index: std::usize::MAX,
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

    fn offset<'a, T>(field_type: &FieldType, iter: T) -> Result<usize, SkillFail>
    where
        T: Iterator<Item = &'a UndefinedFieldData>,
    {
        let mut offset = 0;
        match field_type {
            FieldType::BuildIn(field) => match field {
                BuildInType::ConstTi8 => offset = iter.count(),
                BuildInType::ConstTi16 => offset = 2 * iter.count(),
                BuildInType::ConstTi32 => offset = 4 * iter.count(),
                BuildInType::ConstTi64 => offset = 8 * iter.count(),
                BuildInType::ConstTv64 => for data in iter {
                    match data {
                        UndefinedFieldData::I64(val) => offset += bytes_v64(*val),
                        _ => Err(SkillFail::internal(InternalFail::WorngUndefinedFieldType))?,
                    };
                },
                BuildInType::Tannotation => for data in iter {
                    match data {
                        UndefinedFieldData::User(obj) => {
                            if let Some(obj) = obj {
                                let obj = obj.nucast::<UndefinedObjectT>().unwrap();
                                let obj = obj.borrow(); // borrowing madness
                                if obj.to_prune() {
                                    offset += 2;
                                } else {
                                    offset += bytes_v64(obj.skill_type_id() as i64)
                                        + bytes_v64(obj.get_skill_id() as i64);
                                }
                            } else {
                                offset += 2;
                            }
                        }
                        _ => Err(SkillFail::internal(InternalFail::WorngUndefinedFieldType))?,
                    }
                },
                BuildInType::Tbool => offset = iter.count(),
                BuildInType::Ti8 => offset = iter.count(),
                BuildInType::Ti16 => offset = 2 * iter.count(),
                BuildInType::Ti32 => offset = 4 * iter.count(),
                BuildInType::Ti64 => offset = 8 * iter.count(),
                BuildInType::Tv64 => for data in iter {
                    match data {
                        UndefinedFieldData::I64(val) => offset += bytes_v64(*val),
                        _ => Err(SkillFail::internal(InternalFail::WorngUndefinedFieldType))?,
                    }
                },
                BuildInType::Tf32 => offset = 4 * iter.count(),
                BuildInType::Tf64 => offset = 8 * iter.count(),
                BuildInType::Tstring => for data in iter {
                    match data {
                        UndefinedFieldData::String(val) => {
                            offset += bytes_v64(val.get_skill_id() as i64)
                        }
                        _ => Err(SkillFail::internal(InternalFail::WorngUndefinedFieldType))?,
                    }
                },
                BuildInType::ConstTarray(length, box_v) => for data in iter {
                    match data {
                        UndefinedFieldData::Array(array) => {
                            offset += LazyFieldDeclaration::offset(&*box_v, array.iter())?
                        }
                        _ => Err(SkillFail::internal(InternalFail::WorngUndefinedFieldType))?,
                    }
                },
                BuildInType::Tarray(box_v) => for data in iter {
                    match data {
                        UndefinedFieldData::Array(array) => {
                            offset += bytes_v64(array.len() as i64)
                                + LazyFieldDeclaration::offset(&*box_v, array.iter())?
                        }
                        _ => Err(SkillFail::internal(InternalFail::WorngUndefinedFieldType))?,
                    }
                },
                BuildInType::Tlist(box_v) => for data in iter {
                    match data {
                        UndefinedFieldData::Array(array) => {
                            offset += bytes_v64(array.len() as i64)
                                + LazyFieldDeclaration::offset(&*box_v, array.iter())?
                        }
                        _ => Err(SkillFail::internal(InternalFail::WorngUndefinedFieldType))?,
                    }
                },
                BuildInType::Tset(box_v) => for data in iter {
                    match data {
                        UndefinedFieldData::Set(set) => {
                            offset += bytes_v64(set.len() as i64)
                                + LazyFieldDeclaration::offset(&*box_v, set.iter())?
                        }
                        _ => Err(SkillFail::internal(InternalFail::WorngUndefinedFieldType))?,
                    }
                },
                BuildInType::Tmap(key_box_v, box_v) => for data in iter {
                    match data {
                        UndefinedFieldData::Map(map) => {
                            offset += bytes_v64(map.len() as i64)
                                + LazyFieldDeclaration::offset(&*key_box_v, map.keys())?
                                + LazyFieldDeclaration::offset(&*box_v, map.values())?
                        }
                        _ => Err(SkillFail::internal(InternalFail::WorngUndefinedFieldType))?,
                    }
                },
            },
            FieldType::User(_pool) => for data in iter {
                match data {
                    UndefinedFieldData::User(obj) => {
                        if let Some(obj) = obj {
                            let obj = obj.nucast::<UndefinedObjectT>().unwrap();
                            let obj = obj.borrow(); // borrowing madness
                            if obj.to_prune() {
                                offset += 1;
                            } else {
                                offset += bytes_v64(obj.get_skill_id() as i64);
                            }
                        } else {
                            offset += 1;
                        }
                    }
                    _ => Err(SkillFail::internal(InternalFail::WorngUndefinedFieldType))?,
                }
            },
        }
        Ok(offset)
    }
    fn write<'a, T>(
        writer: &mut FileWriter,
        field_type: &FieldType,
        iter: T,
    ) -> Result<(), SkillFail>
    where
        T: Iterator<Item = &'a UndefinedFieldData>,
    {
        match field_type {
            FieldType::BuildIn(field) => match field {
                BuildInType::ConstTi8 => for data in iter {
                    match data {
                        UndefinedFieldData::I8(val) => writer.write_i8(*val)?,
                        _ => Err(SkillFail::internal(InternalFail::WorngUndefinedFieldType))?,
                    }
                },
                BuildInType::ConstTi16 => for data in iter {
                    match data {
                        UndefinedFieldData::I16(val) => writer.write_i16(*val)?,
                        _ => Err(SkillFail::internal(InternalFail::WorngUndefinedFieldType))?,
                    }
                },
                BuildInType::ConstTi32 => for data in iter {
                    match data {
                        UndefinedFieldData::I32(val) => writer.write_i32(*val)?,
                        _ => Err(SkillFail::internal(InternalFail::WorngUndefinedFieldType))?,
                    }
                },
                BuildInType::ConstTi64 => for data in iter {
                    match data {
                        UndefinedFieldData::I64(val) => writer.write_i64(*val)?,
                        _ => Err(SkillFail::internal(InternalFail::WorngUndefinedFieldType))?,
                    }
                },
                BuildInType::ConstTv64 => for data in iter {
                    match data {
                        UndefinedFieldData::I64(val) => writer.write_i64(*val)?,
                        _ => Err(SkillFail::internal(InternalFail::WorngUndefinedFieldType))?,
                    };
                },
                BuildInType::Tannotation => for data in iter {
                    match data {
                        UndefinedFieldData::User(obj) => {
                            if let Some(obj) = obj {
                                let obj = obj.nucast::<UndefinedObjectT>().unwrap();
                                let obj = obj.borrow(); // borrowing madness

                                if obj.to_prune() {
                                    writer.write_i8(0)?;
                                    writer.write_i8(0)?;
                                } else {
                                    writer.write_v64((obj.skill_type_id() - 31) as i64)?;
                                    writer.write_v64(obj.get_skill_id() as i64)?;
                                }
                            } else {
                                writer.write_i8(0)?;
                                writer.write_i8(0)?;
                            }
                        }
                        _ => Err(SkillFail::internal(InternalFail::WorngUndefinedFieldType))?,
                    }
                },
                BuildInType::Tbool => for data in iter {
                    match data {
                        UndefinedFieldData::Bool(val) => writer.write_bool(*val)?,
                        _ => Err(SkillFail::internal(InternalFail::WorngUndefinedFieldType))?,
                    };
                },
                BuildInType::Ti8 => for data in iter {
                    match data {
                        UndefinedFieldData::I8(val) => writer.write_i8(*val)?,
                        _ => Err(SkillFail::internal(InternalFail::WorngUndefinedFieldType))?,
                    };
                },
                BuildInType::Ti16 => for data in iter {
                    match data {
                        UndefinedFieldData::I16(val) => writer.write_i16(*val)?,
                        _ => Err(SkillFail::internal(InternalFail::WorngUndefinedFieldType))?,
                    };
                },
                BuildInType::Ti32 => for data in iter {
                    match data {
                        UndefinedFieldData::I32(val) => writer.write_i32(*val)?,
                        _ => Err(SkillFail::internal(InternalFail::WorngUndefinedFieldType))?,
                    };
                },
                BuildInType::Ti64 => for data in iter {
                    match data {
                        UndefinedFieldData::I64(val) => writer.write_i64(*val)?,
                        _ => Err(SkillFail::internal(InternalFail::WorngUndefinedFieldType))?,
                    };
                },
                BuildInType::Tv64 => for data in iter {
                    match data {
                        UndefinedFieldData::I64(val) => writer.write_v64(*val)?,
                        _ => Err(SkillFail::internal(InternalFail::WorngUndefinedFieldType))?,
                    }
                },
                BuildInType::Tf32 => for data in iter {
                    match data {
                        UndefinedFieldData::F32(val) => writer.write_f32(*val)?,
                        _ => Err(SkillFail::internal(InternalFail::WorngUndefinedFieldType))?,
                    }
                },
                BuildInType::Tf64 => for data in iter {
                    match data {
                        UndefinedFieldData::F64(val) => writer.write_f64(*val)?,
                        _ => Err(SkillFail::internal(InternalFail::WorngUndefinedFieldType))?,
                    }
                },

                BuildInType::Tstring => for data in iter {
                    match data {
                        UndefinedFieldData::String(val) => {
                            writer.write_v64(val.get_skill_id() as i64)?
                        }
                        _ => Err(SkillFail::internal(InternalFail::WorngUndefinedFieldType))?,
                    }
                },
                BuildInType::ConstTarray(_length, box_v) => for data in iter {
                    match data {
                        UndefinedFieldData::Array(array) => {
                            LazyFieldDeclaration::write(writer, &*box_v, array.iter())?
                        }
                        _ => Err(SkillFail::internal(InternalFail::WorngUndefinedFieldType))?,
                    }
                },
                BuildInType::Tarray(box_v) => for data in iter {
                    match data {
                        UndefinedFieldData::Array(array) => {
                            writer.write_v64(array.len() as i64)?;
                            LazyFieldDeclaration::write(writer, &*box_v, array.iter())?
                        }
                        _ => Err(SkillFail::internal(InternalFail::WorngUndefinedFieldType))?,
                    }
                },
                BuildInType::Tlist(box_v) => for data in iter {
                    match data {
                        UndefinedFieldData::Array(array) => {
                            writer.write_v64(array.len() as i64)?;
                            LazyFieldDeclaration::write(writer, &*box_v, array.iter())?
                        }
                        _ => Err(SkillFail::internal(InternalFail::WorngUndefinedFieldType))?,
                    }
                },
                BuildInType::Tset(box_v) => for data in iter {
                    match data {
                        UndefinedFieldData::Set(set) => {
                            writer.write_v64(set.len() as i64)?;
                            LazyFieldDeclaration::write(writer, &*box_v, set.iter())?
                        }
                        _ => Err(SkillFail::internal(InternalFail::WorngUndefinedFieldType))?,
                    }
                },
                BuildInType::Tmap(key_box_v, box_v) => for data in iter {
                    match data {
                        UndefinedFieldData::Map(map) => {
                            writer.write_v64(map.len() as i64)?;
                            for (key, val) in map.iter() {
                                LazyFieldDeclaration::write(
                                    writer,
                                    &*key_box_v,
                                    SingleItemIter::new(key),
                                )?;
                                LazyFieldDeclaration::write(
                                    writer,
                                    &*box_v,
                                    SingleItemIter::new(val),
                                )?;
                            }
                        }
                        _ => Err(SkillFail::internal(InternalFail::WorngUndefinedFieldType))?,
                    }
                },
            },
            FieldType::User(_pool) => for data in iter {
                match data {
                    UndefinedFieldData::User(obj) => {
                        if let Some(obj) = obj {
                            let obj = obj.nucast::<UndefinedObjectT>().unwrap();
                            let obj = obj.borrow(); // borrowing madness

                            if obj.to_prune() {
                                writer.write_i8(0)?;
                            } else {
                                writer.write_v64(obj.get_skill_id() as i64)?;
                            }
                        } else {
                            writer.write_i8(0)?;
                        }
                    }
                    _ => Err(SkillFail::internal(InternalFail::WorngUndefinedFieldType))?,
                }
            },
        }
        Ok(())
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
        &mut self,
        block_reader: &Vec<FileReader>,
        string_block: &StringBlock,
        blocks: &Vec<Block>,
        type_pools: &Vec<Rc<RefCell<InstancePool>>>,
        instances: &[Ptr<SkillObject>],
    ) -> Result<(), SkillFail> {
        info!(
            target:"SkillWriting",
            "~~~Deserialize field {}",
            self.name.as_str(),
        );
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

                                        if self.undefined_vec_index == std::usize::MAX {
                                            self.undefined_vec_index =
                                                obj.undefined_fields_mut().len();
                                        } else if self.undefined_vec_index
                                            != obj.undefined_fields_mut().len()
                                        {
                                            return Err(SkillFail::internal(
                                                InternalFail::InconsistentUndefinedIndex {
                                                    old: self.undefined_vec_index,
                                                    new: obj.undefined_fields_mut().len(),
                                                },
                                            ));
                                        }

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

                                    if self.undefined_vec_index == std::usize::MAX {
                                        self.undefined_vec_index = obj.undefined_fields_mut().len();
                                    } else if self.undefined_vec_index
                                        != obj.undefined_fields_mut().len()
                                    {
                                        return Err(SkillFail::internal(
                                            InternalFail::InconsistentUndefinedIndex {
                                                old: self.undefined_vec_index,
                                                new: obj.undefined_fields_mut().len(),
                                            },
                                        ));
                                    }

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
    fn offset(&self, iter: dynamic_data::Iter) -> Result<usize, SkillFail> {
        let mut offset = 0;
        match &self.field_type {
            FieldType::BuildIn(field) => match field {
                BuildInType::ConstTi8 => offset = iter.count(),
                BuildInType::ConstTi16 => offset = 2 * iter.count(),
                BuildInType::ConstTi32 => offset = 4 * iter.count(),
                BuildInType::ConstTi64 => offset = 8 * iter.count(),
                BuildInType::ConstTv64 => for obj in iter {
                    let obj = obj.nucast::<UndefinedObjectT>().unwrap();
                    let obj = obj.borrow(); // borrowing madness

                    match &obj.undefined_fields()[self.undefined_vec_index] {
                        UndefinedFieldData::I64(val) => offset += bytes_v64(*val),
                        _ => Err(SkillFail::internal(InternalFail::WorngUndefinedFieldType))?,
                    };
                },
                BuildInType::Tannotation => for obj in iter {
                    let obj = obj.nucast::<UndefinedObjectT>().unwrap();
                    let obj = obj.borrow(); // borrowing madness

                    match &obj.undefined_fields()[self.undefined_vec_index] {
                        UndefinedFieldData::User(obj) => {
                            if let Some(obj) = obj {
                                if obj.borrow().to_prune() {
                                    offset += 2;
                                } else {
                                    offset += bytes_v64(obj.borrow().skill_type_id() as i64)
                                        + bytes_v64(obj.borrow().get_skill_id() as i64);
                                }
                            } else {
                                offset += 2
                            }
                        }
                        _ => Err(SkillFail::internal(InternalFail::WorngUndefinedFieldType))?,
                    }
                },
                BuildInType::Tbool => offset = iter.count(),
                BuildInType::Ti8 => offset = iter.count(),
                BuildInType::Ti16 => offset = 2 * iter.count(),
                BuildInType::Ti32 => offset = 4 * iter.count(),
                BuildInType::Ti64 => offset = 8 * iter.count(),
                BuildInType::Tv64 => for obj in iter {
                    let obj = obj.nucast::<UndefinedObjectT>().unwrap();
                    let obj = obj.borrow(); // borrowing madness

                    match &obj.undefined_fields()[self.undefined_vec_index] {
                        UndefinedFieldData::I64(val) => offset += bytes_v64(*val),
                        _ => Err(SkillFail::internal(InternalFail::WorngUndefinedFieldType))?,
                    }
                },
                BuildInType::Tf32 => offset = 4 * iter.count(),
                BuildInType::Tf64 => offset = 8 * iter.count(),
                BuildInType::Tstring => for obj in iter {
                    let obj = obj.nucast::<UndefinedObjectT>().unwrap();
                    let obj = obj.borrow(); // borrowing madness

                    match &obj.undefined_fields()[self.undefined_vec_index] {
                        UndefinedFieldData::String(val) => {
                            offset += bytes_v64(val.get_skill_id() as i64)
                        }
                        _ => Err(SkillFail::internal(InternalFail::WorngUndefinedFieldType))?,
                    }
                },
                BuildInType::ConstTarray(length, box_v) => for obj in iter {
                    let obj = obj.nucast::<UndefinedObjectT>().unwrap();
                    let obj = obj.borrow(); // borrowing madness

                    match &obj.undefined_fields()[self.undefined_vec_index] {
                        UndefinedFieldData::Array(array) => {
                            offset += LazyFieldDeclaration::offset(&*box_v, array.iter())?;
                        }
                        _ => Err(SkillFail::internal(InternalFail::WorngUndefinedFieldType))?,
                    }
                },
                BuildInType::Tarray(box_v) => for obj in iter {
                    let obj = obj.nucast::<UndefinedObjectT>().unwrap();
                    let obj = obj.borrow(); // borrowing madness

                    match &obj.undefined_fields()[self.undefined_vec_index] {
                        UndefinedFieldData::Array(array) => {
                            offset += bytes_v64(array.len() as i64)
                                + LazyFieldDeclaration::offset(&*box_v, array.iter())?;
                        }
                        _ => Err(SkillFail::internal(InternalFail::WorngUndefinedFieldType))?,
                    }
                },
                BuildInType::Tlist(box_v) => for obj in iter {
                    let obj = obj.nucast::<UndefinedObjectT>().unwrap();
                    let obj = obj.borrow(); // borrowing madness

                    match &obj.undefined_fields()[self.undefined_vec_index] {
                        UndefinedFieldData::Array(array) => {
                            offset += bytes_v64(array.len() as i64)
                                + LazyFieldDeclaration::offset(&*box_v, array.iter())?;
                        }
                        _ => Err(SkillFail::internal(InternalFail::WorngUndefinedFieldType))?,
                    }
                },
                BuildInType::Tset(box_v) => for obj in iter {
                    let obj = obj.nucast::<UndefinedObjectT>().unwrap();
                    let obj = obj.borrow(); // borrowing madness

                    match &obj.undefined_fields()[self.undefined_vec_index] {
                        UndefinedFieldData::Set(set) => {
                            offset += bytes_v64(set.len() as i64)
                                + LazyFieldDeclaration::offset(&*box_v, set.iter())?;
                        }
                        _ => Err(SkillFail::internal(InternalFail::WorngUndefinedFieldType))?,
                    }
                },
                BuildInType::Tmap(key_box_v, box_v) => for obj in iter {
                    let obj = obj.nucast::<UndefinedObjectT>().unwrap();
                    let obj = obj.borrow(); // borrowing madness

                    match &obj.undefined_fields()[self.undefined_vec_index] {
                        UndefinedFieldData::Map(map) => {
                            offset += bytes_v64(map.len() as i64)
                                + LazyFieldDeclaration::offset(&*key_box_v, map.keys())?
                                + LazyFieldDeclaration::offset(&*box_v, map.values())?;
                        }
                        _ => Err(SkillFail::internal(InternalFail::WorngUndefinedFieldType))?,
                    }
                },
            },
            FieldType::User(_pool) => for obj in iter {
                let obj = obj.nucast::<UndefinedObjectT>().unwrap();
                let obj = obj.borrow(); // borrowing madness

                match &obj.undefined_fields()[self.undefined_vec_index] {
                    UndefinedFieldData::User(obj) => {
                        if let Some(obj) = obj {
                            if obj.borrow().to_prune() {
                                offset += 1;
                            } else {
                                offset += bytes_v64(obj.borrow().get_skill_id() as i64);
                            }
                        } else {
                            offset += 1;
                        }
                    }
                    _ => Err(SkillFail::internal(InternalFail::WorngUndefinedFieldType))?,
                }
            },
        }
        Ok(offset)
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
        let end_offset = offset + self.offset(iter)?;
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
        let mut writer = match self.chunks.first().unwrap() {
            FieldChunk::Declaration(ref chunk) => writer.rel_view(chunk.begin, chunk.end)?,
            FieldChunk::Continuation(_) => Err(SkillFail::internal(InternalFail::OnlyOneChunk))?,
        };
        match &self.field_type {
            FieldType::BuildIn(field) => match field {
                BuildInType::ConstTi8 => for obj in iter {
                    let obj = obj.nucast::<UndefinedObjectT>().unwrap();
                    let obj = obj.borrow(); // borrowing madness

                    match &obj.undefined_fields()[self.undefined_vec_index] {
                        UndefinedFieldData::I8(val) => writer.write_i8(*val)?,
                        _ => Err(SkillFail::internal(InternalFail::WorngUndefinedFieldType))?,
                    }
                },
                BuildInType::ConstTi16 => for obj in iter {
                    let obj = obj.nucast::<UndefinedObjectT>().unwrap();
                    let obj = obj.borrow(); // borrowing madness

                    match &obj.undefined_fields()[self.undefined_vec_index] {
                        UndefinedFieldData::I16(val) => writer.write_i16(*val)?,
                        _ => Err(SkillFail::internal(InternalFail::WorngUndefinedFieldType))?,
                    }
                },
                BuildInType::ConstTi32 => for obj in iter {
                    let obj = obj.nucast::<UndefinedObjectT>().unwrap();
                    let obj = obj.borrow(); // borrowing madness

                    match &obj.undefined_fields()[self.undefined_vec_index] {
                        UndefinedFieldData::I32(val) => writer.write_i32(*val)?,
                        _ => Err(SkillFail::internal(InternalFail::WorngUndefinedFieldType))?,
                    }
                },
                BuildInType::ConstTi64 => for obj in iter {
                    let obj = obj.nucast::<UndefinedObjectT>().unwrap();
                    let obj = obj.borrow(); // borrowing madness

                    match &obj.undefined_fields()[self.undefined_vec_index] {
                        UndefinedFieldData::I64(val) => writer.write_i64(*val)?,
                        _ => Err(SkillFail::internal(InternalFail::WorngUndefinedFieldType))?,
                    }
                },
                BuildInType::ConstTv64 => for obj in iter {
                    let obj = obj.nucast::<UndefinedObjectT>().unwrap();
                    let obj = obj.borrow(); // borrowing madness

                    match &obj.undefined_fields()[self.undefined_vec_index] {
                        UndefinedFieldData::I64(val) => writer.write_i64(*val)?,
                        _ => Err(SkillFail::internal(InternalFail::WorngUndefinedFieldType))?,
                    };
                },
                BuildInType::Tannotation => for obj in iter {
                    let obj = obj.nucast::<UndefinedObjectT>().unwrap();
                    let obj = obj.borrow(); // borrowing madness

                    match &obj.undefined_fields()[self.undefined_vec_index] {
                        UndefinedFieldData::User(obj) => {
                            if let Some(obj) = obj {
                                let obj = obj.nucast::<UndefinedObjectT>().unwrap();
                                let obj = obj.borrow(); // borrowing madness
                                if obj.to_prune() {
                                    writer.write_i8(0)?;
                                    writer.write_i8(0)?;
                                } else {
                                    writer.write_v64((obj.skill_type_id() - 31) as i64)?;
                                    writer.write_v64(obj.get_skill_id() as i64)?;
                                }
                            } else {
                                writer.write_i8(0)?;
                                writer.write_i8(0)?;
                            }
                        }
                        _ => Err(SkillFail::internal(InternalFail::WorngUndefinedFieldType))?,
                    }
                },
                BuildInType::Tbool => for obj in iter {
                    let obj = obj.nucast::<UndefinedObjectT>().unwrap();
                    let obj = obj.borrow(); // borrowing madness

                    match &obj.undefined_fields()[self.undefined_vec_index] {
                        UndefinedFieldData::Bool(val) => writer.write_bool(*val)?,
                        _ => Err(SkillFail::internal(InternalFail::WorngUndefinedFieldType))?,
                    };
                },
                BuildInType::Ti8 => for obj in iter {
                    let obj = obj.nucast::<UndefinedObjectT>().unwrap();
                    let obj = obj.borrow(); // borrowing madness

                    match &obj.undefined_fields()[self.undefined_vec_index] {
                        UndefinedFieldData::I8(val) => writer.write_i8(*val)?,
                        _ => Err(SkillFail::internal(InternalFail::WorngUndefinedFieldType))?,
                    };
                },
                BuildInType::Ti16 => for obj in iter {
                    let obj = obj.nucast::<UndefinedObjectT>().unwrap();
                    let obj = obj.borrow(); // borrowing madness

                    match &obj.undefined_fields()[self.undefined_vec_index] {
                        UndefinedFieldData::I16(val) => writer.write_i16(*val)?,
                        _ => Err(SkillFail::internal(InternalFail::WorngUndefinedFieldType))?,
                    };
                },
                BuildInType::Ti32 => for obj in iter {
                    let obj = obj.nucast::<UndefinedObjectT>().unwrap();
                    let obj = obj.borrow(); // borrowing madness

                    match &obj.undefined_fields()[self.undefined_vec_index] {
                        UndefinedFieldData::I32(val) => writer.write_i32(*val)?,
                        _ => Err(SkillFail::internal(InternalFail::WorngUndefinedFieldType))?,
                    };
                },
                BuildInType::Ti64 => for obj in iter {
                    let obj = obj.nucast::<UndefinedObjectT>().unwrap();
                    let obj = obj.borrow(); // borrowing madness

                    match &obj.undefined_fields()[self.undefined_vec_index] {
                        UndefinedFieldData::I64(val) => writer.write_i64(*val)?,
                        _ => Err(SkillFail::internal(InternalFail::WorngUndefinedFieldType))?,
                    };
                },
                BuildInType::Tv64 => for obj in iter {
                    let obj = obj.nucast::<UndefinedObjectT>().unwrap();
                    let obj = obj.borrow(); // borrowing madness

                    match &obj.undefined_fields()[self.undefined_vec_index] {
                        UndefinedFieldData::I64(val) => writer.write_v64(*val)?,
                        _ => Err(SkillFail::internal(InternalFail::WorngUndefinedFieldType))?,
                    }
                },
                BuildInType::Tf32 => for obj in iter {
                    let obj = obj.nucast::<UndefinedObjectT>().unwrap();
                    let obj = obj.borrow(); // borrowing madness

                    match &obj.undefined_fields()[self.undefined_vec_index] {
                        UndefinedFieldData::F32(val) => writer.write_f32(*val)?,
                        _ => Err(SkillFail::internal(InternalFail::WorngUndefinedFieldType))?,
                    }
                },
                BuildInType::Tf64 => for obj in iter {
                    let obj = obj.nucast::<UndefinedObjectT>().unwrap();
                    let obj = obj.borrow(); // borrowing madness

                    match &obj.undefined_fields()[self.undefined_vec_index] {
                        UndefinedFieldData::F64(val) => writer.write_f64(*val)?,
                        _ => Err(SkillFail::internal(InternalFail::WorngUndefinedFieldType))?,
                    }
                },
                BuildInType::Tstring => for obj in iter {
                    let obj = obj.nucast::<UndefinedObjectT>().unwrap();
                    let obj = obj.borrow(); // borrowing madness

                    match &obj.undefined_fields()[self.undefined_vec_index] {
                        UndefinedFieldData::String(val) => {
                            writer.write_v64(val.get_skill_id() as i64)?
                        }
                        _ => Err(SkillFail::internal(InternalFail::WorngUndefinedFieldType))?,
                    }
                },
                BuildInType::ConstTarray(_length, box_v) => for obj in iter {
                    let obj = obj.nucast::<UndefinedObjectT>().unwrap();
                    let obj = obj.borrow(); // borrowing madness

                    match &obj.undefined_fields()[self.undefined_vec_index] {
                        UndefinedFieldData::Array(array) => {
                            LazyFieldDeclaration::write(&mut writer, &*box_v, array.iter())?
                        }
                        _ => Err(SkillFail::internal(InternalFail::WorngUndefinedFieldType))?,
                    }
                },
                BuildInType::Tarray(box_v) => for obj in iter {
                    let obj = obj.nucast::<UndefinedObjectT>().unwrap();
                    let obj = obj.borrow(); // borrowing madness

                    match &obj.undefined_fields()[self.undefined_vec_index] {
                        UndefinedFieldData::Array(array) => {
                            writer.write_v64(array.len() as i64)?;
                            LazyFieldDeclaration::write(&mut writer, &*box_v, array.iter())?
                        }
                        _ => Err(SkillFail::internal(InternalFail::WorngUndefinedFieldType))?,
                    }
                },
                BuildInType::Tlist(box_v) => for obj in iter {
                    let obj = obj.nucast::<UndefinedObjectT>().unwrap();
                    let obj = obj.borrow(); // borrowing madness

                    match &obj.undefined_fields()[self.undefined_vec_index] {
                        UndefinedFieldData::Array(array) => {
                            writer.write_v64(array.len() as i64)?;
                            LazyFieldDeclaration::write(&mut writer, &*box_v, array.iter())?
                        }
                        _ => Err(SkillFail::internal(InternalFail::WorngUndefinedFieldType))?,
                    }
                },
                BuildInType::Tset(box_v) => for obj in iter {
                    let obj = obj.nucast::<UndefinedObjectT>().unwrap();
                    let obj = obj.borrow(); // borrowing madness

                    match &obj.undefined_fields()[self.undefined_vec_index] {
                        UndefinedFieldData::Set(set) => {
                            writer.write_v64(set.len() as i64)?;
                            LazyFieldDeclaration::write(&mut writer, &*box_v, set.iter())?
                        }
                        _ => Err(SkillFail::internal(InternalFail::WorngUndefinedFieldType))?,
                    }
                },
                BuildInType::Tmap(key_box_v, box_v) => for obj in iter {
                    let obj = obj.nucast::<UndefinedObjectT>().unwrap();
                    let obj = obj.borrow(); // borrowing madness

                    match &obj.undefined_fields()[self.undefined_vec_index] {
                        UndefinedFieldData::Map(map) => {
                            writer.write_v64(map.len() as i64)?;
                            for (key, val) in map.iter() {
                                LazyFieldDeclaration::write(
                                    &mut writer,
                                    &*key_box_v,
                                    SingleItemIter::new(key),
                                )?;
                                LazyFieldDeclaration::write(
                                    &mut writer,
                                    &*box_v,
                                    SingleItemIter::new(val),
                                )?;
                            }
                        }
                        _ => Err(SkillFail::internal(InternalFail::WorngUndefinedFieldType))?,
                    }
                },
            },
            FieldType::User(_pool) => for obj in iter {
                let obj = obj.nucast::<UndefinedObjectT>().unwrap();
                let obj = obj.borrow(); // borrowing madness

                match &obj.undefined_fields()[self.undefined_vec_index] {
                    UndefinedFieldData::User(obj) => {
                        if let Some(obj) = obj {
                            let obj = obj.nucast::<UndefinedObjectT>().unwrap();
                            let obj = obj.borrow(); // borrowing madness

                            if obj.to_prune() {
                                writer.write_i8(0)?;
                            } else {
                                writer.write_v64(obj.get_skill_id() as i64)?;
                            }
                        } else {
                            writer.write_i8(0)?;
                        }
                    }
                    _ => Err(SkillFail::internal(InternalFail::WorngUndefinedFieldType))?,
                }
            },
        }
        Ok(())
    }
}
