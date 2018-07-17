/*  ___ _  ___ _ _                                                            *\
** / __| |/ (_) | |       The SKilL Generator                                 **
** \__ \ ' <| | | |__     (c) 2013-18 University of Stuttgart                 **
** |___/_|\_\_|_|____|    see LICENSE                                         **
\*                                                                            */
package de.ust.skill.generator.rust

import de.ust.skill.generator.common.Indenter._
import de.ust.skill.ir._

import scala.collection.JavaConverters._

trait PoolsMaker extends GeneralOutputMaker {
  abstract override def make {
    super.make

    makeSource()
  }

  private final def makeSource() {
    // one file per base type
    for (base ← IR) {
      val out = files.open(s"src/${snakeCase(storagePool(base))}.rs")

      out.write(
                 e"""${genUsage(base)}
                    |
                    |${genType(base)}
                    |
                    |${genPool(base)}
                    |
                    |${genFieldReader(base)}
                    |""".stripMargin)
      out.close()
    }
  }

  //----------------------------------------
  // Usage
  //----------------------------------------
  // TODO turn off warning for unused
  private final def genUsage(base: UserType): String = {
    e"""use common::internal::InstancePool;
       |use common::internal::LazyFieldReader;
       |use common::internal::ObjectReader;
       |use common::internal::SkillObject;
       |use common::internal::UndefinedObject;
       |use common::io::{Block, BlockIndex, FileReader, FieldReader, FieldChunk, BuildInType, FieldType};
       |use common::StringBlock;
       |use common::SkillError;
       |use common::{Ptr, WeakPtr};
       |
       |use skill_file::SkillFileBuilder;
       |
       |${getUsageStd()}
       |
       |${getUsageUser(base)}
       |""".stripMargin
  }

  private final def getUsageUser(base: UserType): String = {
    IR
    .filterNot(t ⇒ t.equals(base))
    .toArray
    .map(t ⇒ s"use ${snakeCase(storagePool(t))}::*;\n")
    .sorted
    .mkString
  }

  private final def getUsageStd(): String = {
    e"""
       |use std::cell::RefCell;
       |use std::collections::HashMap;
       |use std::collections::HashSet;
       |use std::collections::LinkedList;
       |use std::ops::DerefMut;
       |use std::rc::Rc;
       |""".stripMargin
  }

  //----------------------------------------
  // Type
  //----------------------------------------
  private final def genType(base: UserType): String = {
    e"""//----------------------------------------
       |// ${base.getName} aka ${name(base)}
       |//----------------------------------------
       |${genTypeStruct(base)}
       |
       |${genTypeTrait(base)}
       |
       |${genTypeImpl(base)}
       |""".stripMargin.trim
  }

  private final def genTypeStruct(base: UserType): String = {
    e"""#[derive(Default, Debug,  PartialEq)]
       |pub struct ${name(base)} {
       |    ${
      (for (f ← base.getAllFields.asScala) yield {
        e"""${name(f)}: ${mapType(f.getType)},
           |""".stripMargin
      }).mkString.trim
    }
       |}""".stripMargin
  }

  private final def genTypeTrait(base: UserType): String = {
    e"""pub trait ${traitName(base)} : ${
      if (base.getSuperType != null) {
        traitName(base.getSuperType)
      } else {
        "SkillObject"
      }
    } {
       |    ${
      (for (f ← base.getFields.asScala) yield {
        e"""fn get_${name(f)}(&self) -> ${
          if (returnByRef(f.getType)) {
            "&" + mapType(f.getType)
          } else {
            mapType(f.getType)
          }
        };
fn set_${name(f)}(&mut self, ${name(f)}: ${mapType(f.getType)});
""".stripMargin
      }).mkString.trim
    }
       |}""".stripMargin
  }

  private final def genGetSetImpl(field: Field): String = {
    e"""fn get_${name(field)}(&self) -> ${
      if (returnByRef(field.getType)) {
        "&" + mapType(field.getType)
      } else {
        mapType(field.getType)
      }
    } {
       |    ${
      if (returnByRef(field.getType)) {
        e"&self.${name(field)}"
      } else {
        e"self.${name(field)}"
      }
    }
       |}
       |
       |fn set_${name(field)}(&mut self, value: ${mapType(field.getType)}) {
       |    self.${name(field)} = value;
       |}""".stripMargin
  }

  private final def genTypeImpl(base: UserType): String = {
    // gen New
    e"""impl ${name(base)} {
       |    pub fn new() -> ${name(base)} {
       |        // TODO optimize for small / single field objects?
       |        ${name(base)} {
       |            ${
      (for (f ← base.getAllFields.asScala) yield {
        e"""${name(f)}: ${defaultValue(f)},
           |""".stripMargin
      }).mkString.trim
    }
       |        }
       |    }
       |}
       |
       |impl ${traitName(base)} for ${name(base)} {
       |    ${ // Impl base
      (for (f ← base.getFields.asScala) yield {
        e"""${genGetSetImpl(f)}
           |
           |""".stripMargin
      }).mkString.trim
    }
       |}
       |
       |${
      // Impl super
      var parent = base.getSuperType
      val ret = new StringBuilder()

      while (parent != null) {
        ret.append(
                    e"""${genTypeImplSuper(base, parent)}
                       |
                       |""".stripMargin
                  )
        parent = parent.getSuperType
      }
      ret.mkString.trim
    }
       |
       |impl SkillObject for ${name(base)} {}
       |""".stripMargin.trim
  }

  private final def genTypeImplSuper(base: UserType,
                                     parent: UserType): String = {
    e"""impl ${traitName(parent)} for ${name(base)} {
       |    ${
      (for (f ← parent.getFields.asScala) yield {
        e"""${genGetSetImpl(f)}
           |
           |""".stripMargin
      }).mkString.trim
    }
       |}""".stripMargin
  }


  //----------------------------------------
  // TypePool
  //----------------------------------------
  private final def genPool(base: UserType): String = {
    e"""//----------------------------------------
       |// ${base.getName}Pool aka ${storagePool(base)}
       |//----------------------------------------
       |${genPoolStruct(base)}
       |
       |${genPoolImpl(base)}
       |
       |${genPoolImplInstancePool(base)}
       |""".stripMargin.trim
  }

  private final def genPoolStruct(base: UserType): String = {
    e"""#[derive(Default)]
       |pub struct ${storagePool(base)} {
       |    string_block: Rc<RefCell<StringBlock>>,
       |    instances: Rc<RefCell<Vec<Ptr<SkillObject>>>>,
       |    book_static: Vec<Ptr<SkillObject>>,
       |    book_dynamic: Vec<Ptr<SkillObject>>,${
      "" // TODO Implement the rest of book / freelist for the one "page"
    }
       |    // TODO needed after construction?
       |    fields: Vec<Box<FieldReader>>,
       |    type_name_index: usize,
       |    type_id: usize,
       |    blocks: Vec<Block>,
       |    super_pool: Option<Rc<RefCell<InstancePool>>>,
       |    sub_pools: Vec<Rc<RefCell<InstancePool>>>,
       |    base_pool: Option<Rc<RefCell<InstancePool>>>,
       |    static_count: usize,
       |    dynamic_count: usize,
       |    cached_count: usize,
       |    deleted_count: usize,
       |    invariant: bool,
       |}""".stripMargin
  }

  private final def genPoolImpl(base: UserType): String = {
    e"""impl ${storagePool(base)} {
       |    pub fn new(
       |        string_block: Rc<RefCell<StringBlock>>,
       |    ) -> ${storagePool(base)} {
       |        ${storagePool(base)} {
       |            string_block,
       |            instances: Rc::default(),
       |            book_static: Vec::new(),
       |            book_dynamic: Vec::new(),
       |            fields: Vec::new(),
       |            type_name_index: 0,
       |            type_id: 0,
       |            blocks: Vec::new(),
       |            super_pool: None,
       |            base_pool: None,
       |            sub_pools: Vec::new(),
       |            static_count: 0,
       |            dynamic_count: 0,
       |            cached_count: 0,
       |            deleted_count: 0,
       |            invariant: false,
       |        }
       |    }
       |
       |    pub fn get(&self, index: usize) -> Result<WeakPtr<${traitName(base)}>, SkillError> {
       |        match self.instances.borrow().get(index) {
       |            Some(obj) => match obj.nucast::<${traitName(base)}>() {
       |                Some(obj) => Ok(obj.downgrade()),
       |                None => panic!("Bad cast"),
       |            },
       |            None => Err(SkillError::BadSkillObjectID),
       |        }
       |    }
       |
       |    pub fn id(&self) -> usize {
       |        self.type_id
       |    }
       |}""".stripMargin
  }

  private final def genPoolImplInstancePool(base: UserType): String = {
    e"""impl InstancePool for ${storagePool(base)} {
       |    ${genPoolImplInstancePoolAddField(base)}
       |
       |    fn initialize(
       |        &self,
       |        file_reader: &Vec<FileReader>,
       |        string_block: &StringBlock,
       |        type_pools: &Vec<Rc<RefCell<InstancePool>>>,
       |    ) -> Result<(), SkillError> {
       |        for f in self.fields.iter() {
       |            let instances = self.instances.borrow();
       |            f.read(
       |                file_reader,
       |                string_block,
       |                &self.blocks,
       |                type_pools,
       |                &instances
       |            )?;
       |        }
       |        Ok(())
       |    }
       |
       |    fn allocate(&mut self, type_pools: &Vec<Rc<RefCell<InstancePool>>>) {
       |        let mut vec = self.instances.borrow_mut();
       |        if self.is_base() {
       |            let tmp = Ptr::new(UndefinedObject::new());
       |            info!(
       |                target:"SkillParsing",
       |                "Allocate space for:${base.getName} aka ${name(base)} amount:{}",
       |                self.get_global_cached_count(),
       |            );
       |            trace!(
       |                target:"SkillParsing",
       |                "Allocate space for:${name(base)} with:{:?}",
       |                tmp,
       |            );
       |
       |            vec.reserve(self.get_global_cached_count()); // FIXME check if dynamic count is the correct one
       |            // TODO figure out a better way - set_len doesn't wrk as dtor will be called on garbage data
       |
       |            for _ in 0..self.get_global_cached_count() {
       |                vec.push(tmp.clone());
       |            }
       |        }
       |        self.book_static.reserve(self.static_count);
       |
       |        info!(
       |            target:"SkillParsing",
       |            "Initialize ${base.getName} aka ${name(base)} id:{}",
       |            self.get_type_id(),
       |        );
       |
       |        for block in self.blocks.iter() {
       |            let begin = block.bpo + 1;
       |            let end = begin + block.static_count;
       |            for id in begin..end {
       |                trace!(
       |                    target:"SkillParsing",
       |                    "${name(base)} id:{} block:{:?}",
       |                    id,
       |                    block,
       |                );
       |
       |                self.book_static.push(Ptr::new(${name(base)}::new()));
       |                vec[id - 1] = self.book_static.last().unwrap().clone();
       |            }
       |        }
       |    }
       |    fn has_field(&self, name_id: usize) -> bool {
       |        for f in & self.fields {
       |            if f.name_id() == name_id {
       |                return true
       |            }
       |        }
       |        false
       |    }
       |
       |    fn field_amount(&self) -> usize {
       |        self.fields.len()
       |    }
       |
       |    fn add_chunk_to(&mut self, name_id: usize, chunk: FieldChunk) {
       |        for f in &mut self.fields {
       |            if f.name_id() == name_id {
       |                f.add_chunk(chunk);
       |                return;
       |            }
       |        }
       |        panic!("No field of id:{}", name_id);
       |    }
       |
       |    fn set_type_id(&mut self, id: usize) {
       |        self.type_id = id;
       |    }
       |    fn get_type_id(&self) -> usize {
       |        self.type_id
       |    }
       |    fn set_type_name_index(&mut self, id: usize) {
       |        self.type_name_index = id;
       |    }
       |    fn get_type_name_index(&self) -> usize {
       |        self.type_name_index
       |    }
       |
       |    fn get_base_vec(&self) -> Rc<RefCell<Vec<Ptr<SkillObject>>>> {
       |        self.instances.clone()
       |    }
       |
       |    fn read_object(&self, index: usize) -> Result<Ptr<SkillObject>, SkillError> {
       |        assert!(index >= 1);
       |        info!(
       |            target:"SkillParsing",
       |            "read user instance:{} from:{}",
       |            index,
       |            self.instances.borrow().len(),
       |        );
       |        Ok(self.instances.borrow()[index - 1].clone())
       |    }
       |
       |    fn add_block(&mut self, block: Block) {
       |        self.blocks.push(block);
       |    }
       |    fn blocks(&mut self) -> &mut Vec<Block> {
       |        &mut self.blocks
       |    }
       |
       |    fn add_sub(&mut self, pool: Rc<RefCell<InstancePool>>) {
       |        self.sub_pools.push(pool);
       |    }
       |
       |    fn set_super(&mut self, pool: Rc<RefCell<InstancePool>>) {
       |        if pool.borrow().is_base() {
       |            self.base_pool = Some(pool.clone());
       |        } else {
       |            self.base_pool = pool.borrow().get_base(); // TODO check?
       |        }
       |        self.instances = self.base_pool.as_ref().unwrap().borrow().get_base_vec();
       |        self.super_pool = Some(pool);
       |    }
       |    fn get_super(&self) -> Option<Rc<RefCell<InstancePool>>> {
       |        self.super_pool.clone()
       |    }
       |
       |    fn get_base(&self) -> Option<Rc<RefCell<InstancePool>>> {
       |        self.base_pool.clone()
       |    }
       |    fn is_base(&self) -> bool {
       |        self.super_pool.is_none()
       |    }
       |
       |    fn get_local_static_count(&self) -> usize {
       |        if let Some(block) = self.blocks.last() {
       |            return block.static_count;
       |        }
       |        panic!();
       |    }
       |    fn set_local_static_count(&mut self, count: usize) {
       |        if let Some(block) = self.blocks.last_mut() {
       |            block.static_count = count;
       |        } else {
       |            panic!();
       |        }
       |    }
       |
       |    fn get_local_dynamic_count(&self) -> usize {
       |        if let Some(block) = self.blocks.last() {
       |            return block.dynamic_count;
       |        }
       |        panic!();
       |    }
       |
       |    fn get_local_bpo(&self) -> usize {
       |        if let Some(block) = self.blocks.last() {
       |            return block.bpo;
       |        }
       |        panic!();
       |    }
       |
       |    fn set_invariant(&mut self, invariant: bool) {
       |        if self.invariant != invariant {
       |            self.invariant = invariant;
       |            if invariant {
       |                self.cached_count = self.static_count - self.deleted_count;
       |                for s in self.sub_pools.iter() {
       |                    let mut s = s.borrow_mut();
       |                    s.set_invariant(true);
       |                    self.cached_count += s.get_global_cached_count();
       |                }
       |            } else if self.super_pool.is_some() {
       |                self.super_pool
       |                    .as_ref()
       |                    .unwrap()
       |                    .borrow_mut()
       |                    .set_invariant(false);
       |            }
       |        }
       |    }
       |
       |    fn size(&self) -> usize {
       |        if self.invariant {
       |            self.cached_count
       |        } else {
       |            let mut ret = self.static_count;
       |            for s in self.sub_pools.iter() {
       |                ret += s.borrow().size();
       |            }
       |            ret
       |        }
       |    }
       |
       |    fn get_global_static_count(&self) -> usize {
       |        self.static_count
       |    }
       |    fn set_global_static_count(&mut self, count: usize) {
       |        self.static_count = count;
       |    }
       |
       |    fn get_global_cached_count(&self) -> usize {
       |        self.cached_count
       |    }
       |    fn set_global_cached_count(&mut self, count: usize) {
       |         self.cached_count = count;
       |    }
       |
       |    fn make_instance(&self) -> Ptr<SkillObject> {
       |        trace!(
       |            target:"SkillParsing",
       |            "Create new ${name(base)}",
       |        );
       |        Ptr::new(${name(base)}::new())
       |    }
       |
       |}""".stripMargin
  }

  private final def genPoolImplInstancePoolAddField(base: UserType): String = {
    e"""// TODO is the field type important for something? apparent from the restrictions?
       |fn add_field(
       |    &mut self,
       |    name_id: usize,
       |    field_name: &str,
       |    mut field_type: FieldType,
       |    chunk: FieldChunk,
       |) {
       |    match field_name {
       |        ${
      (for (f ← base.getAllFields.asScala) yield {
        genPoolImplInstancePoolAddFieldField(base, f)
      }).mkString.trim
    }
       |        _ => {
       |            let mut reader = Box::new(LazyFieldReader::new(name_id));
       |            reader.as_mut().add_chunk(chunk);
       |            self.fields.push(reader);
       |        },
       |    }
       |}
       |""".stripMargin
  }

  // TODO do something about these stupid names
  private final def genPoolImplInstancePoolAddFieldField(base: UserType,
                                                         field: Field): String = {
    e""""${field.getName.lower()}" => match field_type {
       |    ${
      field.getType match {
        case t@(_: SingleBaseTypeContainer | _: MapType) ⇒
          e"""${mapTypeToMagicMatch(t)} => {
             |    let mut object_readers: Vec<Rc<RefCell<InstancePool>>> = Vec::new();
             |    // TODO reserve size
             |    ${genPoolImplInstancePoolAddFieldFieldUnwrapValidate(t)}
             |    let mut reader = Box::new(${fieldReader(base, field)}::new(name_id, object_readers));
             |    reader.add_chunk(chunk);;
             |    self.fields.push(reader);
             |}""".stripMargin
        case t: GroundType                               ⇒
          e"""|${mapTypeToMagicMatch(t)} => {
              |   let mut reader = Box::new(${fieldReader(base, field)}::new(name_id));
              |   reader.add_chunk(chunk);
              |   self.fields.push(reader);
              |},""".stripMargin
        case t: UserType                                 ⇒
          e"""|${mapTypeToMagicMatch(t)} => {
              |   let mut reader = Box::new(${fieldReader(base, field)}::new(name_id, vec!(pool)));
              |   reader.add_chunk(chunk);
              |   self.fields.push(reader);
              |},""".stripMargin
        case _                                           ⇒
          throw new GeneratorException("Unexpected field type")
      }
    }
       |    _ => panic!("Expected: ${mapTypeToUser(field.getType)} Found: {}", field_type)
       |},
       |""".stripMargin
  }

  // FIXME We need the user provided pool to access the objects - validation cant be a separate step
  private final def genPoolImplInstancePoolAddFieldFieldUnwrapValidate(tt: Type): String = {
    tt match {
      case t: ConstantLengthArrayType ⇒
        e"""if length != ${t.getLength} {
           |    panic!("The length of the constant length ({}) array differs form the generated one (${
          t.getLength
        })!", length);
           |}
           |match **box_v {
           |  ${genPoolImplInstancePoolAddFieldFieldUnwrapValidate(t.getBaseType)}
           |  _ => panic!("Expected: ${mapTypeToUser(t.getBaseType)} Found: {}", **box_v)
           |}""".stripMargin
      case t: SingleBaseTypeContainer ⇒
        e"""match **box_v {
           |    ${genPoolImplInstancePoolAddFieldFieldUnwrapValidate(t.getBaseType)}
           |    _ => panic!("Expected: ${mapTypeToUser(t.getBaseType)} Found: {}", **box_v)
           |}""".stripMargin
      case t: MapType                 ⇒
        e"""${genPoolImplInstancePoolAddFieldFieldUnwrapValidateMap(t.getBaseTypes.asScala.toList)}
           |""".stripMargin
      case t: GroundType              ⇒
        e"""|${mapTypeToMagicMatch(t)} => {},
            |""".stripMargin
      case _: UserType                ⇒
        e"""|FieldType::User(ref object_reader, _) => object_readers.push(object_reader.clone()),
            |""".stripMargin
      case _                          ⇒
        throw new GeneratorException("Unexpected field type")
    }
  }.trim

  private final def genPoolImplInstancePoolAddFieldFieldUnwrapValidateMap(tts: List[Type]): String = {
    val (key, remainder) = tts.splitAt(1)

    e"""match **key_box_v {
       |    ${genPoolImplInstancePoolAddFieldFieldUnwrapValidate(key.head)}
       |    _ => panic!("Expected: ${mapTypeToUser(key.head)} Found: {}", **key_box_v)
       |}
       |match **box_v {
       |    ${ // FIXME Insert map match - the list only contains the contents - not the map
      if (remainder.size >= 2) {
        e"""FieldType::BuildIn(BuildInType::Tmap(ref key_box_v, ref box_v)) => {
           |    ${genPoolImplInstancePoolAddFieldFieldUnwrapValidateMap(remainder)}
           |}""".stripMargin
      } else {
        genPoolImplInstancePoolAddFieldFieldUnwrapValidate(remainder.head)
      }
    }
       |    _ => panic!("Expected: ${mapTypeToUser(remainder.head)} Found: {}", **box_v)
       |}
       |""".stripMargin
  }.trim

  //----------------------------------------
  // FieldReader
  //----------------------------------------
  private final def genFieldReader(base: UserType): String = {
    val ret = new StringBuilder()

    for (field ← base.getAllFields.asScala) {
      ret.append(
                  e"""//----------------------------------------
                     |// ${base.getName.camel()}${field.getName.capital()}FieldReader aka ${fieldReader(base, field)}
                     |//----------------------------------------
                     |${genFieldReaderType(base, field)}
                     |
                     |${genFieldReaderImpl(base, field)}
                     |
                     |${genFieldReaderImplFieldReader(base, field)}
                     |
                     |""".stripMargin
                )
    }
    ret.mkString.trim
  }

  private final def genFieldReaderType(base: UserType,
                                       field: Field): String = {
    e"""struct ${fieldReader(base, field)} {
       |    name_id: usize,
       |    chunks: Vec<FieldChunk>,${
      field.getType match {
        case _: GroundType ⇒
          ""
        case _             ⇒
          e"""
             |object_reader: Vec<Rc<RefCell<InstancePool>>>,""".stripMargin
      }
    }
       |}""".stripMargin
  }

  private final def genFieldReaderImpl(base: UserType,
                                       field: Field): String = {
    // TODO really check if an object has to be read (e.g. map<int,int> doesn't need to)
    field.getType match {
      case _: GroundType ⇒
        e"""impl ${fieldReader(base, field)} {
           |    fn new(
           |        name_id: usize,
           |    ) -> ${fieldReader(base, field)} {
           |        ${fieldReader(base, field)} {
           |            name_id,
           |            chunks: Vec::new(),
           |        }
           |    }
           |}""".stripMargin
      case _             ⇒
        e"""impl ${fieldReader(base, field)} {
           |    fn new(
           |        name_id: usize,
           |        object_reader: Vec<Rc<RefCell<InstancePool>>>
           |    ) -> ${fieldReader(base, field)} {
           |        ${fieldReader(base, field)} {
           |            name_id,
           |            chunks: Vec::new(),
           |            object_reader,
           |        }
           |    }
           |}""".stripMargin
    }
  }

  private final def genFieldReaderImplFieldReader(base: UserType,
                                                  field: Field): String = {
    e"""impl FieldReader for ${fieldReader(base, field)} {
       |    fn read(
       |        &self,
       |        file_reader: &Vec<FileReader>,
       |        string_block: &StringBlock,
       |        blocks: &Vec<Block>,
       |        type_pools: &Vec<Rc<RefCell<InstancePool>>>,
       |        instances: &[Ptr<SkillObject>],
       |    ) -> Result<(), SkillError> {
       |        let mut block_index = BlockIndex::from(0);
       |
       |        for chunk in self.chunks.iter() {
       |            match chunk {
       |                FieldChunk::Declaration(chunk) => {
       |                    block_index += chunk.appearance - 1;
       |
       |                    let block = &blocks[block_index.block];
       |                    let mut reader = file_reader[block.block.block].rel_view(chunk.begin, chunk.end);
       |                    block_index += 1;
       |
       |                    if chunk.count > 0 {
       |                        for block in blocks.iter().take(chunk.appearance.block) {
       |                            let mut o = 0;
       |
       |                            for obj in instances.iter()
       |                                .skip(block.bpo)
       |                                .take(block.dynamic_count)
       |                            {
       |                                debug!(
       |                                    target:"SkillParsing",
       |                                    "Block:{:?} Object:{}",
       |                                    block,
       |                                    o + block.bpo,
       |                                );
       |                                o += 1;
       |                                match obj.nucast::<${traitName(base)}>() {
       |                                    Some(obj) =>
       |                                        obj.borrow_mut().set_${name(field)}(${
      genFieldReaderImplFieldReaderRead(field.getType, Stream.iterate(0)(_ + 1).iterator)
    }),
       |                                    None => panic!("Casting error"), // FIXME
       |                                }
       |                            }
       |                        }
       |                    }
       |                },
       |                FieldChunk::Continuation(chunk) => {
       |                    let block = &blocks[block_index.block];
       |                    let mut reader = file_reader[block.block.block].rel_view(chunk.begin, chunk.end);
       |                    block_index += 1;
       |
       |                    if chunk.count > 0 {
       |                        let mut o = 0;
       |                        for obj in instances.iter()
       |                            .skip(chunk.bpo)
       |                            .take(chunk.count)
       |                        {
       |                            debug!(
       |                                target:"SkillParsing",
       |                                "Block:{:?} Object:{}",
       |                                block,
       |                                o + chunk.bpo,
       |                            );
       |                            o += 1;
       |
       |                            match obj.nucast::<${traitName(base)}>() {
       |                                Some(obj) =>
       |                                    obj.borrow_mut().set_${name(field)}(${
      genFieldReaderImplFieldReaderRead(field.getType, Stream.iterate(0)(_ + 1).iterator)
    }),
       |                                None => panic!("Casting error"), // FIXME
       |                            }
       |                        }
       |                    }
       |                }
       |            }
       |        }
       |        Ok(())
       |    }
       |    fn add_chunk(&mut self, chunk: FieldChunk) {
       |        self.chunks.push(chunk);
       |    }
       |    fn name_id(&self) -> usize {
       |        self.name_id
       |    }
       |}""".stripMargin
  }

  private final def genFieldReaderImplFieldReaderRead(base: Type,
                                                      user: Iterator[Int]): String = {
    base match {
      case t: GroundType
        if t.getName.lower().equals("string")     ⇒
        e"""string_block.get(reader.read_v64()? as usize)
           |""".stripMargin
      case t: GroundType
        if t.getName.lower().equals("annotation") ⇒
        e"""{
           |    let pool = reader.read_v64()? as usize;
           |    let object = reader.read_v64()? as usize;
           |    if pool != 0 && object != 0 {
           |        if let Some(object) = type_pools[pool - 1]
           |            .borrow()
           |            .read_object(object)?
           |            .nucast::<SkillObject>()
           |        {
           |            Some(object)
           |        } else {
           |            // NOTE this should only be reached if the generated code is faulty
           |            panic!("Failed to cast object to SkillObject");
           |        }
           |    } else {
           |        None
           |    }
           |}
           |""".stripMargin.trim
      case t: GroundType                          ⇒
        e"""reader.read_${unbox(t)}()?
           |""".stripMargin
      case t: ConstantLengthArrayType             ⇒
        // TODO check that everything was read?
        e"""{
           |    let mut vec = Vec::new();
           |    vec.reserve(${t.getLength});
           |    for _ in 0..${t.getLength} {
           |        vec.push(${genFieldReaderImplFieldReaderRead(t.getBaseType, user)});
           |    }
           |    vec
           |}
           |""".stripMargin
      case t: VariableLengthArrayType             ⇒
        e"""{
           |    let elements = reader.read_v64()? as usize;
           |    let mut vec = Vec::new();
           |    vec.reserve(elements);
           |    for _ in 0..elements {
           |        vec.push(${genFieldReaderImplFieldReaderRead(t.getBaseType, user)});
           |    }
           |    vec
           |}
           |""".stripMargin
      case t: ListType                            ⇒
        e"""{
           |    let elements = reader.read_v64()? as usize;
           |    let mut list = LinkedList::new();
           |    for _ in 0..elements {
           |        list.push_back(${genFieldReaderImplFieldReaderRead(t.getBaseType, user)});
           |    }
           |    list
           |}
           |""".stripMargin
      case t: SetType                             ⇒
        e"""{
           |    let elements = reader.read_v64()? as usize;
           |    let mut set = HashSet::new();
           |    set.reserve(elements);
           |    for _ in 0..elements {
           |        set.insert(${genFieldReaderImplFieldReaderRead(t.getBaseType, user)});
           |    }
           |    set
           |}
           |""".stripMargin
      case t: MapType                             ⇒
        genFieldReaderImplFieldReaderReadMap(t.getBaseTypes.asScala.toList, user)
      case t: UserType                            ⇒
        e"""{
           |    let object = reader.read_v64()? as usize;
           |    if object != 0 {
           |        if let Some(object) = self.object_reader[${user.next()}]
           |            .borrow()
           |            .read_object(object)?
           |            .nucast::<${traitName(t)}>()
           |        {
           |            Some(object)
           |        } else {
           |            panic!("Failed to cast object to:${t.getName} aka ${traitName(t)}")
           |        }
           |    } else {
           |        None
           |    }
           |}
           |""".stripMargin.trim
      case _                                      ⇒
        throw new GeneratorException(s"Unknown type $base")
    }
  }.trim


  private final def genFieldReaderImplFieldReaderReadMap(tts: List[Type], user: Iterator[Int]): String = {
    val (key, remainder) = tts.splitAt(1)

    if (remainder.nonEmpty) {
      e"""{
         |    let elements = reader.read_v64()? as usize;
         |    let mut map = HashMap::new();
         |    map.reserve(elements);
         |    for _ in 0..elements {
         |        map.insert(
         |            ${genFieldReaderImplFieldReaderRead(key.head, user)},
         |            ${genFieldReaderImplFieldReaderReadMap(remainder, user)}
         |        );
         |    }
         |    map
         |}""".stripMargin
    } else {
      genFieldReaderImplFieldReaderRead(key.head, user)
    }
  }

  private final def mapTypeToMagic(t: Type): String = t match {
    case t: GroundType ⇒ s"BuildInType::T${t.getName.lower}"

    case _: ConstantLengthArrayType ⇒ s"BuildInType::ConstTarray"
    case _: VariableLengthArrayType ⇒ s"BuildInType::Tarray"
    case _: ListType                ⇒ s"BuildInType::Tlist"
    case _: SetType                 ⇒ s"BuildInType::Tset"
    case _: MapType                 ⇒ s"BuildInType::Tmap"

    case _ ⇒ throw new GeneratorException(s"Unknown type $t")
  }

  private final def mapTypeToMagicMatch(t: Type): String = t match {
    case _: ConstantLengthArrayType ⇒ s"FieldType::BuildIn(${mapTypeToMagic(t)}(length, ref box_v))"
    case _: MapType                 ⇒ s"FieldType::BuildIn(${mapTypeToMagic(t)}(ref key_box_v, ref box_v))"
    case _@(_: VariableLengthArrayType | _: ListType | _: SetType)
                                    ⇒ s"FieldType::BuildIn(${mapTypeToMagic(t)}(ref box_v))"
    case _: UserType                ⇒ e"""FieldType::User(pool, type_id)"""
    case _                          ⇒ e"""FieldType::BuildIn(${mapTypeToMagic(t)})"""
  }

  private final def mapTypeToUser(t: Type): String = t match {
    case t: ConstantLengthArrayType ⇒ s"${t.getLength}[${mapTypeToUser(t.getBaseType)}]"
    case t: VariableLengthArrayType ⇒ s"v[${mapTypeToUser(t.getBaseType)}]"
    case t: ListType                ⇒ s"List[${mapTypeToUser(t.getBaseType)}]"
    case t: SetType                 ⇒ s"Set{{${mapTypeToUser(t.getBaseType)}}}"
    case t: MapType                 ⇒ s"${mapTypeToUserMap(t.getBaseTypes.asScala.toList)}"
    case _: GroundType              ⇒ s"T${t.getName.lower}"
    case _: UserType                ⇒ s"UserType"

    case _ ⇒ throw new GeneratorException(s"Unknown type $t")
  }

  private final def mapTypeToUserMap(tts: List[Type]): String = {
    val (key, remainder) = tts.splitAt(1)

    if (remainder.nonEmpty) {
      e"Map{{${mapTypeToUser(key.head)},${mapTypeToUserMap(remainder)}}}"
    } else {
      mapTypeToUser(key.head)
    }
  }

  private final def returnByRef(t: Type): Boolean = t match {
    case t: GroundType ⇒ t.getSkillName match {
      case "string"     ⇒ true
      case "annotation" ⇒ true
      case _            ⇒ false
    }

    case _: ConstantLengthArrayType ⇒ true
    case _: VariableLengthArrayType ⇒ true
    case _: ListType                ⇒ true
    case _: SetType                 ⇒ true
    case _: MapType                 ⇒ true

    case _: UserType ⇒ true

    case _ ⇒ throw new GeneratorException(s"Unknown type $t")
  }
}
