use common::error::*;
use common::internal::io::*;
use common::internal::*;
use common::iterator::dynamic_data;
use common::*;

use std::cell::RefCell;
use std::rc::Rc;

use std::collections::HashMap;
use std::collections::HashSet;

/// Helper Iterator to reduce code duplication
struct SingleItemIter<'a> {
    item: Option<&'a foreign::FieldData>,
}

impl<'a> SingleItemIter<'a> {
    fn new(item: &'a foreign::FieldData) -> SingleItemIter {
        SingleItemIter { item: Some(item) }
    }
}

impl<'a> Iterator for SingleItemIter<'a> {
    type Item = &'a foreign::FieldData;
    fn next(&mut self) -> Option<&'a foreign::FieldData> {
        if let Some(item) = self.item {
            self.item = None;
            Some(item)
        } else {
            None
        }
    }
}

pub(crate) struct FieldDeclaration {
    name: Rc<SkillString>,
    field_id: usize,
    foreign_vec_index: usize,
    chunks: Vec<FieldChunk>,
    field_type: FieldType,
}

impl FieldDeclaration {
    pub(crate) fn new(
        name: Rc<SkillString>,
        field_id: usize,
        field_type: FieldType,
    ) -> FieldDeclaration {
        FieldDeclaration {
            name,
            field_id,
            chunks: Vec::new(),
            field_type,
            foreign_vec_index: std::usize::MAX,
        }
    }

    // TODO this should also produce an offset for fields that do not contain user types / strings
    fn read_foreign_field(
        field: &FieldType,
        reader: &mut FileReader,
        string_pool: &StringBlock,
        type_pools: &Vec<Rc<RefCell<PoolProxy>>>,
    ) -> Result<foreign::FieldData, SkillFail> {
        Ok(match field {
            FieldType::BuildIn(ref field) => match field {
                BuildInType::ConstTi8 => foreign::FieldData::I8(reader.read_i8()?),
                BuildInType::ConstTi16 => foreign::FieldData::I16(reader.read_i16()?),
                BuildInType::ConstTi32 => foreign::FieldData::I32(reader.read_i32()?),
                BuildInType::ConstTi64 => foreign::FieldData::I64(reader.read_i64()?),
                BuildInType::ConstTv64 => foreign::FieldData::I64(reader.read_v64()?),
                BuildInType::Tbool => foreign::FieldData::Bool(reader.read_bool()?),
                BuildInType::Ti8 => foreign::FieldData::I8(reader.read_i8()?),
                BuildInType::Ti16 => foreign::FieldData::I16(reader.read_i16()?),
                BuildInType::Ti32 => foreign::FieldData::I32(reader.read_i32()?),
                BuildInType::Ti64 => foreign::FieldData::I64(reader.read_i64()?),
                BuildInType::Tv64 => foreign::FieldData::I64(reader.read_v64()?),
                BuildInType::Tf32 => foreign::FieldData::F32(reader.read_f32()?),
                BuildInType::Tf64 => foreign::FieldData::F64(reader.read_f64()?),
                BuildInType::Tannotation => foreign::FieldData::User({
                    let pool = reader.read_v64()? as usize;
                    let object = reader.read_v64()? as usize;
                    if pool != 0 && object != 0 {
                        Some(type_pools[pool - 1].borrow().pool().read_object(object)?)
                    } else {
                        None
                    }
                }),
                BuildInType::Tstring => {
                    foreign::FieldData::String(string_pool.get(reader.read_v64()? as usize)?)
                }
                BuildInType::ConstTarray(length, box_v) => foreign::FieldData::Array({
                    let mut arr = Vec::with_capacity(*length as usize);
                    for i in 0..*length as usize {
                        arr[i] = FieldDeclaration::read_foreign_field(
                            &*box_v,
                            reader,
                            string_pool,
                            type_pools,
                        )?;
                    }
                    arr
                }),
                BuildInType::Tarray(box_v) => foreign::FieldData::Array({
                    let elements = reader.read_v64()? as usize;
                    let mut vec = Vec::with_capacity(elements);
                    for _ in 0..elements {
                        vec.push(FieldDeclaration::read_foreign_field(
                            &*box_v,
                            reader,
                            string_pool,
                            type_pools,
                        )?);
                    }
                    vec
                }),
                BuildInType::Tlist(box_v) => foreign::FieldData::Array({
                    let elements = reader.read_v64()? as usize;
                    let mut vec = Vec::with_capacity(elements);
                    for _ in 0..elements {
                        vec.push(FieldDeclaration::read_foreign_field(
                            &*box_v,
                            reader,
                            string_pool,
                            type_pools,
                        )?);
                    }
                    vec
                }),
                BuildInType::Tset(box_v) => foreign::FieldData::Set({
                    let elements = reader.read_v64()? as usize;
                    let mut set = HashSet::new();
                    set.reserve(elements);
                    for _ in 0..elements {
                        set.insert(FieldDeclaration::read_foreign_field(
                            &*box_v,
                            reader,
                            string_pool,
                            type_pools,
                        )?);
                    }
                    set
                }),
                BuildInType::Tmap(key_box_v, box_v) => foreign::FieldData::Map({
                    let elements = reader.read_v64()? as usize;
                    let mut map = HashMap::new();
                    map.reserve(elements);
                    for _ in 0..elements {
                        map.insert(
                            FieldDeclaration::read_foreign_field(
                                &*key_box_v,
                                reader,
                                string_pool,
                                type_pools,
                            )?,
                            FieldDeclaration::read_foreign_field(
                                &*box_v,
                                reader,
                                string_pool,
                                type_pools,
                            )?,
                        );
                    }
                    map
                }),
            },
            FieldType::User(ref pool) => foreign::FieldData::User({
                let object = reader.read_v64()? as usize;
                if object != 0 {
                    Some(pool.borrow().pool().read_object(object)?)
                } else {
                    None
                }
            }),
        })
    }

    fn offset<'a, T>(field_type: &FieldType, iter: T) -> Result<usize, SkillFail>
    where
        T: Iterator<Item = &'a foreign::FieldData>,
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
                        foreign::FieldData::I64(val) => offset += bytes_v64(*val),
                        _ => Err(SkillFail::internal(InternalFail::WrongForeignField))?,
                    };
                },
                BuildInType::Tannotation => for data in iter {
                    match data {
                        foreign::FieldData::User(obj) => {
                            if let Some(obj) = obj {
                                let obj = obj.nucast::<foreign::Object>().unwrap();
                                let obj = obj.borrow(); // borrowing madness
                                if obj.to_delete() {
                                    offset += 2;
                                } else {
                                    offset += bytes_v64(obj.skill_type_id() as i64)
                                        + bytes_v64(obj.get_skill_id() as i64);
                                }
                            } else {
                                offset += 2;
                            }
                        }
                        _ => Err(SkillFail::internal(InternalFail::WrongForeignField))?,
                    }
                },
                BuildInType::Tbool => offset = iter.count(),
                BuildInType::Ti8 => offset = iter.count(),
                BuildInType::Ti16 => offset = 2 * iter.count(),
                BuildInType::Ti32 => offset = 4 * iter.count(),
                BuildInType::Ti64 => offset = 8 * iter.count(),
                BuildInType::Tv64 => for data in iter {
                    match data {
                        foreign::FieldData::I64(val) => offset += bytes_v64(*val),
                        _ => Err(SkillFail::internal(InternalFail::WrongForeignField))?,
                    }
                },
                BuildInType::Tf32 => offset = 4 * iter.count(),
                BuildInType::Tf64 => offset = 8 * iter.count(),
                BuildInType::Tstring => for data in iter {
                    match data {
                        foreign::FieldData::String(val) => {
                            offset += bytes_v64(val.get_skill_id() as i64)
                        }
                        _ => Err(SkillFail::internal(InternalFail::WrongForeignField))?,
                    }
                },
                BuildInType::ConstTarray(length, box_v) => for data in iter {
                    match data {
                        foreign::FieldData::Array(array) => {
                            offset += FieldDeclaration::offset(&*box_v, array.iter())?
                        }
                        _ => Err(SkillFail::internal(InternalFail::WrongForeignField))?,
                    }
                },
                BuildInType::Tarray(box_v) => for data in iter {
                    match data {
                        foreign::FieldData::Array(array) => {
                            offset += bytes_v64(array.len() as i64)
                                + FieldDeclaration::offset(&*box_v, array.iter())?
                        }
                        _ => Err(SkillFail::internal(InternalFail::WrongForeignField))?,
                    }
                },
                BuildInType::Tlist(box_v) => for data in iter {
                    match data {
                        foreign::FieldData::Array(array) => {
                            offset += bytes_v64(array.len() as i64)
                                + FieldDeclaration::offset(&*box_v, array.iter())?
                        }
                        _ => Err(SkillFail::internal(InternalFail::WrongForeignField))?,
                    }
                },
                BuildInType::Tset(box_v) => for data in iter {
                    match data {
                        foreign::FieldData::Set(set) => {
                            offset += bytes_v64(set.len() as i64)
                                + FieldDeclaration::offset(&*box_v, set.iter())?
                        }
                        _ => Err(SkillFail::internal(InternalFail::WrongForeignField))?,
                    }
                },
                BuildInType::Tmap(key_box_v, box_v) => for data in iter {
                    match data {
                        foreign::FieldData::Map(map) => {
                            offset += bytes_v64(map.len() as i64)
                                + FieldDeclaration::offset(&*key_box_v, map.keys())?
                                + FieldDeclaration::offset(&*box_v, map.values())?
                        }
                        _ => Err(SkillFail::internal(InternalFail::WrongForeignField))?,
                    }
                },
            },
            FieldType::User(_pool) => for data in iter {
                match data {
                    foreign::FieldData::User(obj) => {
                        if let Some(obj) = obj {
                            let obj = obj.nucast::<foreign::Object>().unwrap();
                            let obj = obj.borrow(); // borrowing madness
                            if obj.to_delete() {
                                offset += 1;
                            } else {
                                offset += bytes_v64(obj.get_skill_id() as i64);
                            }
                        } else {
                            offset += 1;
                        }
                    }
                    _ => Err(SkillFail::internal(InternalFail::WrongForeignField))?,
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
        T: Iterator<Item = &'a foreign::FieldData>,
    {
        match field_type {
            FieldType::BuildIn(field) => match field {
                BuildInType::ConstTi8 => for data in iter {
                    match data {
                        foreign::FieldData::I8(val) => writer.write_i8(*val)?,
                        _ => Err(SkillFail::internal(InternalFail::WrongForeignField))?,
                    }
                },
                BuildInType::ConstTi16 => for data in iter {
                    match data {
                        foreign::FieldData::I16(val) => writer.write_i16(*val)?,
                        _ => Err(SkillFail::internal(InternalFail::WrongForeignField))?,
                    }
                },
                BuildInType::ConstTi32 => for data in iter {
                    match data {
                        foreign::FieldData::I32(val) => writer.write_i32(*val)?,
                        _ => Err(SkillFail::internal(InternalFail::WrongForeignField))?,
                    }
                },
                BuildInType::ConstTi64 => for data in iter {
                    match data {
                        foreign::FieldData::I64(val) => writer.write_i64(*val)?,
                        _ => Err(SkillFail::internal(InternalFail::WrongForeignField))?,
                    }
                },
                BuildInType::ConstTv64 => for data in iter {
                    match data {
                        foreign::FieldData::I64(val) => writer.write_i64(*val)?,
                        _ => Err(SkillFail::internal(InternalFail::WrongForeignField))?,
                    };
                },
                BuildInType::Tannotation => for data in iter {
                    match data {
                        foreign::FieldData::User(obj) => {
                            if let Some(obj) = obj {
                                let obj = obj.nucast::<foreign::Object>().unwrap();
                                let obj = obj.borrow(); // borrowing madness

                                if obj.to_delete() {
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
                        _ => Err(SkillFail::internal(InternalFail::WrongForeignField))?,
                    }
                },
                BuildInType::Tbool => for data in iter {
                    match data {
                        foreign::FieldData::Bool(val) => writer.write_bool(*val)?,
                        _ => Err(SkillFail::internal(InternalFail::WrongForeignField))?,
                    };
                },
                BuildInType::Ti8 => for data in iter {
                    match data {
                        foreign::FieldData::I8(val) => writer.write_i8(*val)?,
                        _ => Err(SkillFail::internal(InternalFail::WrongForeignField))?,
                    };
                },
                BuildInType::Ti16 => for data in iter {
                    match data {
                        foreign::FieldData::I16(val) => writer.write_i16(*val)?,
                        _ => Err(SkillFail::internal(InternalFail::WrongForeignField))?,
                    };
                },
                BuildInType::Ti32 => for data in iter {
                    match data {
                        foreign::FieldData::I32(val) => writer.write_i32(*val)?,
                        _ => Err(SkillFail::internal(InternalFail::WrongForeignField))?,
                    };
                },
                BuildInType::Ti64 => for data in iter {
                    match data {
                        foreign::FieldData::I64(val) => writer.write_i64(*val)?,
                        _ => Err(SkillFail::internal(InternalFail::WrongForeignField))?,
                    };
                },
                BuildInType::Tv64 => for data in iter {
                    match data {
                        foreign::FieldData::I64(val) => writer.write_v64(*val)?,
                        _ => Err(SkillFail::internal(InternalFail::WrongForeignField))?,
                    }
                },
                BuildInType::Tf32 => for data in iter {
                    match data {
                        foreign::FieldData::F32(val) => writer.write_f32(*val)?,
                        _ => Err(SkillFail::internal(InternalFail::WrongForeignField))?,
                    }
                },
                BuildInType::Tf64 => for data in iter {
                    match data {
                        foreign::FieldData::F64(val) => writer.write_f64(*val)?,
                        _ => Err(SkillFail::internal(InternalFail::WrongForeignField))?,
                    }
                },

                BuildInType::Tstring => for data in iter {
                    match data {
                        foreign::FieldData::String(val) => {
                            writer.write_v64(val.get_skill_id() as i64)?
                        }
                        _ => Err(SkillFail::internal(InternalFail::WrongForeignField))?,
                    }
                },
                BuildInType::ConstTarray(_length, box_v) => for data in iter {
                    match data {
                        foreign::FieldData::Array(array) => {
                            FieldDeclaration::write(writer, &*box_v, array.iter())?
                        }
                        _ => Err(SkillFail::internal(InternalFail::WrongForeignField))?,
                    }
                },
                BuildInType::Tarray(box_v) => for data in iter {
                    match data {
                        foreign::FieldData::Array(array) => {
                            writer.write_v64(array.len() as i64)?;
                            FieldDeclaration::write(writer, &*box_v, array.iter())?
                        }
                        _ => Err(SkillFail::internal(InternalFail::WrongForeignField))?,
                    }
                },
                BuildInType::Tlist(box_v) => for data in iter {
                    match data {
                        foreign::FieldData::Array(array) => {
                            writer.write_v64(array.len() as i64)?;
                            FieldDeclaration::write(writer, &*box_v, array.iter())?
                        }
                        _ => Err(SkillFail::internal(InternalFail::WrongForeignField))?,
                    }
                },
                BuildInType::Tset(box_v) => for data in iter {
                    match data {
                        foreign::FieldData::Set(set) => {
                            writer.write_v64(set.len() as i64)?;
                            FieldDeclaration::write(writer, &*box_v, set.iter())?
                        }
                        _ => Err(SkillFail::internal(InternalFail::WrongForeignField))?,
                    }
                },
                BuildInType::Tmap(key_box_v, box_v) => for data in iter {
                    match data {
                        foreign::FieldData::Map(map) => {
                            writer.write_v64(map.len() as i64)?;
                            for (key, val) in map.iter() {
                                FieldDeclaration::write(
                                    writer,
                                    &*key_box_v,
                                    SingleItemIter::new(key),
                                )?;
                                FieldDeclaration::write(writer, &*box_v, SingleItemIter::new(val))?;
                            }
                        }
                        _ => Err(SkillFail::internal(InternalFail::WrongForeignField))?,
                    }
                },
            },
            FieldType::User(_pool) => for data in iter {
                match data {
                    foreign::FieldData::User(obj) => {
                        if let Some(obj) = obj {
                            let obj = obj.nucast::<foreign::Object>().unwrap();
                            let obj = obj.borrow(); // borrowing madness

                            if obj.to_delete() {
                                writer.write_i8(0)?;
                            } else {
                                writer.write_v64(obj.get_skill_id() as i64)?;
                            }
                        } else {
                            writer.write_i8(0)?;
                        }
                    }
                    _ => Err(SkillFail::internal(InternalFail::WrongForeignField))?,
                }
            },
        }
        Ok(())
    }
}

impl io::FieldDeclaration for FieldDeclaration {
    fn read(
        &self,
        block_reader: &Vec<FileReader>,
        string_pool: &StringBlock,
        blocks: &Vec<Block>,
        type_pools: &Vec<Rc<RefCell<PoolProxy>>>,
        instances: &[Ptr<SkillObject>],
    ) -> Result<(), SkillFail> {
        Ok(())
    }

    fn deserialize(
        &mut self,
        block_reader: &Vec<FileReader>,
        string_pool: &StringBlock,
        blocks: &Vec<Block>,
        type_pools: &Vec<Rc<RefCell<PoolProxy>>>,
        instances: &[Ptr<SkillObject>],
    ) -> Result<(), SkillFail> {
        info!(
            target: "SkillWriting",
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
                                    "Block:{:?} ObjectProper:{}",
                                    block,
                                    o + block.bpo,
                                );
                                o += 1;
                                match obj.nucast::<foreign::Object>() {
                                    Some(obj) => {
                                        let mut obj = obj.borrow_mut();

                                        if self.foreign_vec_index == std::usize::MAX {
                                            self.foreign_vec_index = obj.foreign_fields_mut().len();
                                        } else if self.foreign_vec_index
                                            != obj.foreign_fields_mut().len()
                                        {
                                            return Err(SkillFail::internal(
                                                InternalFail::InconsistentForeignIndex {
                                                    old: self.foreign_vec_index,
                                                    new: obj.foreign_fields_mut().len(),
                                                },
                                            ));
                                        }

                                        obj.foreign_fields_mut().push(
                                            FieldDeclaration::read_foreign_field(
                                                &self.field_type,
                                                &mut reader,
                                                string_pool,
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
                                "Block:{:?} ObjectProper:{}",
                                block,
                                o + chunk.bpo,
                            );
                            o += 1;

                            match obj.nucast::<foreign::Object>() {
                                Some(obj) => {
                                    let mut obj = obj.borrow_mut();

                                    if self.foreign_vec_index == std::usize::MAX {
                                        self.foreign_vec_index = obj.foreign_fields_mut().len();
                                    } else if self.foreign_vec_index
                                        != obj.foreign_fields_mut().len()
                                    {
                                        return Err(SkillFail::internal(
                                            InternalFail::InconsistentForeignIndex {
                                                old: self.foreign_vec_index,
                                                new: obj.foreign_fields_mut().len(),
                                            },
                                        ));
                                    }

                                    obj.foreign_fields_mut().push(
                                        FieldDeclaration::read_foreign_field(
                                            &self.field_type,
                                            &mut reader,
                                            string_pool,
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
                    let obj = obj.nucast::<foreign::Object>().unwrap();
                    let obj = obj.borrow(); // borrowing madness

                    match &obj.foreign_fields()[self.foreign_vec_index] {
                        foreign::FieldData::I64(val) => offset += bytes_v64(*val),
                        _ => Err(SkillFail::internal(InternalFail::WrongForeignField))?,
                    };
                },
                BuildInType::Tannotation => for obj in iter {
                    let obj = obj.nucast::<foreign::Object>().unwrap();
                    let obj = obj.borrow(); // borrowing madness

                    match &obj.foreign_fields()[self.foreign_vec_index] {
                        foreign::FieldData::User(obj) => {
                            if let Some(obj) = obj {
                                if obj.borrow().to_delete() {
                                    offset += 2;
                                } else {
                                    offset += bytes_v64(obj.borrow().skill_type_id() as i64)
                                        + bytes_v64(obj.borrow().get_skill_id() as i64);
                                }
                            } else {
                                offset += 2
                            }
                        }
                        _ => Err(SkillFail::internal(InternalFail::WrongForeignField))?,
                    }
                },
                BuildInType::Tbool => offset = iter.count(),
                BuildInType::Ti8 => offset = iter.count(),
                BuildInType::Ti16 => offset = 2 * iter.count(),
                BuildInType::Ti32 => offset = 4 * iter.count(),
                BuildInType::Ti64 => offset = 8 * iter.count(),
                BuildInType::Tv64 => for obj in iter {
                    let obj = obj.nucast::<foreign::Object>().unwrap();
                    let obj = obj.borrow(); // borrowing madness

                    match &obj.foreign_fields()[self.foreign_vec_index] {
                        foreign::FieldData::I64(val) => offset += bytes_v64(*val),
                        _ => Err(SkillFail::internal(InternalFail::WrongForeignField))?,
                    }
                },
                BuildInType::Tf32 => offset = 4 * iter.count(),
                BuildInType::Tf64 => offset = 8 * iter.count(),
                BuildInType::Tstring => for obj in iter {
                    let obj = obj.nucast::<foreign::Object>().unwrap();
                    let obj = obj.borrow(); // borrowing madness

                    match &obj.foreign_fields()[self.foreign_vec_index] {
                        foreign::FieldData::String(val) => {
                            offset += bytes_v64(val.get_skill_id() as i64)
                        }
                        _ => Err(SkillFail::internal(InternalFail::WrongForeignField))?,
                    }
                },
                BuildInType::ConstTarray(length, box_v) => for obj in iter {
                    let obj = obj.nucast::<foreign::Object>().unwrap();
                    let obj = obj.borrow(); // borrowing madness

                    match &obj.foreign_fields()[self.foreign_vec_index] {
                        foreign::FieldData::Array(array) => {
                            offset += FieldDeclaration::offset(&*box_v, array.iter())?;
                        }
                        _ => Err(SkillFail::internal(InternalFail::WrongForeignField))?,
                    }
                },
                BuildInType::Tarray(box_v) => for obj in iter {
                    let obj = obj.nucast::<foreign::Object>().unwrap();
                    let obj = obj.borrow(); // borrowing madness

                    match &obj.foreign_fields()[self.foreign_vec_index] {
                        foreign::FieldData::Array(array) => {
                            offset += bytes_v64(array.len() as i64)
                                + FieldDeclaration::offset(&*box_v, array.iter())?;
                        }
                        _ => Err(SkillFail::internal(InternalFail::WrongForeignField))?,
                    }
                },
                BuildInType::Tlist(box_v) => for obj in iter {
                    let obj = obj.nucast::<foreign::Object>().unwrap();
                    let obj = obj.borrow(); // borrowing madness

                    match &obj.foreign_fields()[self.foreign_vec_index] {
                        foreign::FieldData::Array(array) => {
                            offset += bytes_v64(array.len() as i64)
                                + FieldDeclaration::offset(&*box_v, array.iter())?;
                        }
                        _ => Err(SkillFail::internal(InternalFail::WrongForeignField))?,
                    }
                },
                BuildInType::Tset(box_v) => for obj in iter {
                    let obj = obj.nucast::<foreign::Object>().unwrap();
                    let obj = obj.borrow(); // borrowing madness

                    match &obj.foreign_fields()[self.foreign_vec_index] {
                        foreign::FieldData::Set(set) => {
                            offset += bytes_v64(set.len() as i64)
                                + FieldDeclaration::offset(&*box_v, set.iter())?;
                        }
                        _ => Err(SkillFail::internal(InternalFail::WrongForeignField))?,
                    }
                },
                BuildInType::Tmap(key_box_v, box_v) => for obj in iter {
                    let obj = obj.nucast::<foreign::Object>().unwrap();
                    let obj = obj.borrow(); // borrowing madness

                    match &obj.foreign_fields()[self.foreign_vec_index] {
                        foreign::FieldData::Map(map) => {
                            offset += bytes_v64(map.len() as i64)
                                + FieldDeclaration::offset(&*key_box_v, map.keys())?
                                + FieldDeclaration::offset(&*box_v, map.values())?;
                        }
                        _ => Err(SkillFail::internal(InternalFail::WrongForeignField))?,
                    }
                },
            },
            FieldType::User(_pool) => for obj in iter {
                let obj = obj.nucast::<foreign::Object>().unwrap();
                let obj = obj.borrow(); // borrowing madness

                match &obj.foreign_fields()[self.foreign_vec_index] {
                    foreign::FieldData::User(obj) => {
                        if let Some(obj) = obj {
                            if obj.borrow().to_delete() {
                                offset += 1;
                            } else {
                                offset += bytes_v64(obj.borrow().get_skill_id() as i64);
                            }
                        } else {
                            offset += 1;
                        }
                    }
                    _ => Err(SkillFail::internal(InternalFail::WrongForeignField))?,
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
                    let obj = obj.nucast::<foreign::Object>().unwrap();
                    let obj = obj.borrow(); // borrowing madness

                    match &obj.foreign_fields()[self.foreign_vec_index] {
                        foreign::FieldData::I8(val) => writer.write_i8(*val)?,
                        _ => Err(SkillFail::internal(InternalFail::WrongForeignField))?,
                    }
                },
                BuildInType::ConstTi16 => for obj in iter {
                    let obj = obj.nucast::<foreign::Object>().unwrap();
                    let obj = obj.borrow(); // borrowing madness

                    match &obj.foreign_fields()[self.foreign_vec_index] {
                        foreign::FieldData::I16(val) => writer.write_i16(*val)?,
                        _ => Err(SkillFail::internal(InternalFail::WrongForeignField))?,
                    }
                },
                BuildInType::ConstTi32 => for obj in iter {
                    let obj = obj.nucast::<foreign::Object>().unwrap();
                    let obj = obj.borrow(); // borrowing madness

                    match &obj.foreign_fields()[self.foreign_vec_index] {
                        foreign::FieldData::I32(val) => writer.write_i32(*val)?,
                        _ => Err(SkillFail::internal(InternalFail::WrongForeignField))?,
                    }
                },
                BuildInType::ConstTi64 => for obj in iter {
                    let obj = obj.nucast::<foreign::Object>().unwrap();
                    let obj = obj.borrow(); // borrowing madness

                    match &obj.foreign_fields()[self.foreign_vec_index] {
                        foreign::FieldData::I64(val) => writer.write_i64(*val)?,
                        _ => Err(SkillFail::internal(InternalFail::WrongForeignField))?,
                    }
                },
                BuildInType::ConstTv64 => for obj in iter {
                    let obj = obj.nucast::<foreign::Object>().unwrap();
                    let obj = obj.borrow(); // borrowing madness

                    match &obj.foreign_fields()[self.foreign_vec_index] {
                        foreign::FieldData::I64(val) => writer.write_i64(*val)?,
                        _ => Err(SkillFail::internal(InternalFail::WrongForeignField))?,
                    };
                },
                BuildInType::Tannotation => for obj in iter {
                    let obj = obj.nucast::<foreign::Object>().unwrap();
                    let obj = obj.borrow(); // borrowing madness

                    match &obj.foreign_fields()[self.foreign_vec_index] {
                        foreign::FieldData::User(obj) => {
                            if let Some(obj) = obj {
                                let obj = obj.nucast::<foreign::Object>().unwrap();
                                let obj = obj.borrow(); // borrowing madness
                                if obj.to_delete() {
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
                        _ => Err(SkillFail::internal(InternalFail::WrongForeignField))?,
                    }
                },
                BuildInType::Tbool => for obj in iter {
                    let obj = obj.nucast::<foreign::Object>().unwrap();
                    let obj = obj.borrow(); // borrowing madness

                    match &obj.foreign_fields()[self.foreign_vec_index] {
                        foreign::FieldData::Bool(val) => writer.write_bool(*val)?,
                        _ => Err(SkillFail::internal(InternalFail::WrongForeignField))?,
                    };
                },
                BuildInType::Ti8 => for obj in iter {
                    let obj = obj.nucast::<foreign::Object>().unwrap();
                    let obj = obj.borrow(); // borrowing madness

                    match &obj.foreign_fields()[self.foreign_vec_index] {
                        foreign::FieldData::I8(val) => writer.write_i8(*val)?,
                        _ => Err(SkillFail::internal(InternalFail::WrongForeignField))?,
                    };
                },
                BuildInType::Ti16 => for obj in iter {
                    let obj = obj.nucast::<foreign::Object>().unwrap();
                    let obj = obj.borrow(); // borrowing madness

                    match &obj.foreign_fields()[self.foreign_vec_index] {
                        foreign::FieldData::I16(val) => writer.write_i16(*val)?,
                        _ => Err(SkillFail::internal(InternalFail::WrongForeignField))?,
                    };
                },
                BuildInType::Ti32 => for obj in iter {
                    let obj = obj.nucast::<foreign::Object>().unwrap();
                    let obj = obj.borrow(); // borrowing madness

                    match &obj.foreign_fields()[self.foreign_vec_index] {
                        foreign::FieldData::I32(val) => writer.write_i32(*val)?,
                        _ => Err(SkillFail::internal(InternalFail::WrongForeignField))?,
                    };
                },
                BuildInType::Ti64 => for obj in iter {
                    let obj = obj.nucast::<foreign::Object>().unwrap();
                    let obj = obj.borrow(); // borrowing madness

                    match &obj.foreign_fields()[self.foreign_vec_index] {
                        foreign::FieldData::I64(val) => writer.write_i64(*val)?,
                        _ => Err(SkillFail::internal(InternalFail::WrongForeignField))?,
                    };
                },
                BuildInType::Tv64 => for obj in iter {
                    let obj = obj.nucast::<foreign::Object>().unwrap();
                    let obj = obj.borrow(); // borrowing madness

                    match &obj.foreign_fields()[self.foreign_vec_index] {
                        foreign::FieldData::I64(val) => writer.write_v64(*val)?,
                        _ => Err(SkillFail::internal(InternalFail::WrongForeignField))?,
                    }
                },
                BuildInType::Tf32 => for obj in iter {
                    let obj = obj.nucast::<foreign::Object>().unwrap();
                    let obj = obj.borrow(); // borrowing madness

                    match &obj.foreign_fields()[self.foreign_vec_index] {
                        foreign::FieldData::F32(val) => writer.write_f32(*val)?,
                        _ => Err(SkillFail::internal(InternalFail::WrongForeignField))?,
                    }
                },
                BuildInType::Tf64 => for obj in iter {
                    let obj = obj.nucast::<foreign::Object>().unwrap();
                    let obj = obj.borrow(); // borrowing madness

                    match &obj.foreign_fields()[self.foreign_vec_index] {
                        foreign::FieldData::F64(val) => writer.write_f64(*val)?,
                        _ => Err(SkillFail::internal(InternalFail::WrongForeignField))?,
                    }
                },
                BuildInType::Tstring => for obj in iter {
                    let obj = obj.nucast::<foreign::Object>().unwrap();
                    let obj = obj.borrow(); // borrowing madness

                    match &obj.foreign_fields()[self.foreign_vec_index] {
                        foreign::FieldData::String(val) => {
                            writer.write_v64(val.get_skill_id() as i64)?
                        }
                        _ => Err(SkillFail::internal(InternalFail::WrongForeignField))?,
                    }
                },
                BuildInType::ConstTarray(_length, box_v) => for obj in iter {
                    let obj = obj.nucast::<foreign::Object>().unwrap();
                    let obj = obj.borrow(); // borrowing madness

                    match &obj.foreign_fields()[self.foreign_vec_index] {
                        foreign::FieldData::Array(array) => {
                            FieldDeclaration::write(&mut writer, &*box_v, array.iter())?
                        }
                        _ => Err(SkillFail::internal(InternalFail::WrongForeignField))?,
                    }
                },
                BuildInType::Tarray(box_v) => for obj in iter {
                    let obj = obj.nucast::<foreign::Object>().unwrap();
                    let obj = obj.borrow(); // borrowing madness

                    match &obj.foreign_fields()[self.foreign_vec_index] {
                        foreign::FieldData::Array(array) => {
                            writer.write_v64(array.len() as i64)?;
                            FieldDeclaration::write(&mut writer, &*box_v, array.iter())?
                        }
                        _ => Err(SkillFail::internal(InternalFail::WrongForeignField))?,
                    }
                },
                BuildInType::Tlist(box_v) => for obj in iter {
                    let obj = obj.nucast::<foreign::Object>().unwrap();
                    let obj = obj.borrow(); // borrowing madness

                    match &obj.foreign_fields()[self.foreign_vec_index] {
                        foreign::FieldData::Array(array) => {
                            writer.write_v64(array.len() as i64)?;
                            FieldDeclaration::write(&mut writer, &*box_v, array.iter())?
                        }
                        _ => Err(SkillFail::internal(InternalFail::WrongForeignField))?,
                    }
                },
                BuildInType::Tset(box_v) => for obj in iter {
                    let obj = obj.nucast::<foreign::Object>().unwrap();
                    let obj = obj.borrow(); // borrowing madness

                    match &obj.foreign_fields()[self.foreign_vec_index] {
                        foreign::FieldData::Set(set) => {
                            writer.write_v64(set.len() as i64)?;
                            FieldDeclaration::write(&mut writer, &*box_v, set.iter())?
                        }
                        _ => Err(SkillFail::internal(InternalFail::WrongForeignField))?,
                    }
                },
                BuildInType::Tmap(key_box_v, box_v) => for obj in iter {
                    let obj = obj.nucast::<foreign::Object>().unwrap();
                    let obj = obj.borrow(); // borrowing madness

                    match &obj.foreign_fields()[self.foreign_vec_index] {
                        foreign::FieldData::Map(map) => {
                            writer.write_v64(map.len() as i64)?;
                            for (key, val) in map.iter() {
                                FieldDeclaration::write(
                                    &mut writer,
                                    &*key_box_v,
                                    SingleItemIter::new(key),
                                )?;
                                FieldDeclaration::write(
                                    &mut writer,
                                    &*box_v,
                                    SingleItemIter::new(val),
                                )?;
                            }
                        }
                        _ => Err(SkillFail::internal(InternalFail::WrongForeignField))?,
                    }
                },
            },
            FieldType::User(_pool) => for obj in iter {
                let obj = obj.nucast::<foreign::Object>().unwrap();
                let obj = obj.borrow(); // borrowing madness

                match &obj.foreign_fields()[self.foreign_vec_index] {
                    foreign::FieldData::User(obj) => {
                        if let Some(obj) = obj {
                            let obj = obj.nucast::<foreign::Object>().unwrap();
                            let obj = obj.borrow(); // borrowing madness

                            if obj.to_delete() {
                                writer.write_i8(0)?;
                            } else {
                                writer.write_v64(obj.get_skill_id() as i64)?;
                            }
                        } else {
                            writer.write_i8(0)?;
                        }
                    }
                    _ => Err(SkillFail::internal(InternalFail::WrongForeignField))?,
                }
            },
        }
        Ok(())
    }
}
