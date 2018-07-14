/*  ___ _  ___ _ _                                                            *\
** / __| |/ (_) | |       The SKilL Generator                                 **
** \__ \ ' <| | | |__     (c) 2013-18 University of Stuttgart                 **
** |___/_|\_\_|_|____|    see LICENSE                                         **
\*                                                                            */
package de.ust.skill.generator.rust

import de.ust.skill.generator.common.Indenter._
import de.ust.skill.ir._

import scala.collection.JavaConverters._
import scala.collection.mutable

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
       |use common::io::{Block, BlockIndex, FileReader, FieldReader, FieldChunk, BuildInType, FieldType};
       |use common::StringBlock;
       |use common::SkillError;
       |use common::{Ptr, WeakPtr};
       |
       |use skill_file::SkillFileBuilder;
       |
       |${getUsageStd(base)}
       |
       |${getUsageUser(base)}
       |""".stripMargin
  }

  private final def getUsageUser(base: UserType): String = {
    // TODO just collect all user types ...
    val usedTypes = new mutable.HashSet[UserType]()

    for (f ← base.getAllFields.asScala) {
      f.getType match {
        case t: SingleBaseTypeContainer ⇒ t.getBaseType match {
          case t: UserType ⇒ usedTypes += t
          case _           ⇒
        }
        case t: MapType                 ⇒ usedTypes ++= t.getBaseTypes.asScala.collect { case t: UserType ⇒ t }
        case t: UserType                ⇒ usedTypes += t
        case _                          ⇒
      }
    }

    usedTypes.toArray.map(t ⇒ s"use ${snakeCase(storagePool(t))}::*;\n").sorted.mkString
  }

  private final def getUsageStd(base: UserType): String = {
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
       |// ${base.getName.capital()}
       |//----------------------------------------
       |${genTypeStruct(base)}
       |
       |${genTypeTrait(base)}
       |
       |${genTypeImpl(base)}
       |""".stripMargin.trim
  }

  private final def genTypeStruct(base: UserType): String = {
    val cap_base = base.getName.capital()

    e"""#[derive(Default, Debug,  PartialEq)]
       |pub struct $cap_base {
       |    ${
      (for (f ← base.getAllFields.asScala) yield {
        e"""${f.getName.lower()}: ${mapType(f.getType)},
           |""".stripMargin
      }).mkString.trim
    }
       |}""".stripMargin
  }

  private final def genTypeTrait(base: UserType): String = {
    val cap_base = base.getName.capital()
    var superTrait = ""

    if (base.getSuperType != null) {
      superTrait = base.getSuperType.getName.capital() + "T + " + superTrait
    }

    e"""pub trait ${cap_base}T : $superTrait {
       |    ${
      (for (f ← base.getFields.asScala) yield {
        e"""fn get_${f.getName.lower()}(&self) -> ${mapType(f.getType)};
fn set_${f.getName.lower()}(&mut self, ${f.getName.lower()}: ${mapType(f.getType)});
""".stripMargin
      }
      ).mkString.trim
    }
       |}""".stripMargin
  }

  private final def genGetSetImpl(field: Field): String = {
    val low_field = field.getName.lower()

    e"""fn get_$low_field(&self) -> ${mapType(field.getType)} {
       |    self.${
      field.getType match {
        case _: UserType                            ⇒
          e"$low_field.clone()"
        case t: GroundType
          if t.getName.lower().equals("annotation") ⇒
          e"$low_field.clone()"
        case t if isPtr(t)                          ⇒
          e"$low_field.clone()"
        case _                                      ⇒
          low_field
      }
    }
       |}
       |
       |fn set_$low_field(&mut self, value: ${mapType(field.getType)}) {
       |    self.$low_field = value;
       |}""".stripMargin
  }

  private final def genTypeImpl(base: UserType): String = {
    val cap_base = base.getName.capital()

    // gen New
    e"""impl $cap_base {
       |    pub fn new() -> $cap_base {
       |        // TODO optimize for small / single field objects?
       |        $cap_base {
       |            ${
      (for (f ← base.getAllFields.asScala) yield {
        e"""${f.getName.lower()}: ${defaultValue(f)},
           |""".stripMargin
      }
      ).mkString.trim
    }
       |        }
       |    }
       |}
       |
       |impl ${cap_base}T for $cap_base {
       |    ${ // Impl base
      (for (f ← base.getFields.asScala) yield {
        e"""${genGetSetImpl(f)}
           |
           |""".stripMargin
      }
      ).mkString.trim
    }
       |}
       |
       |${
      // Impl super
      (for (parent ← base.getAllSuperTypes.asScala) yield {
        e"""${genTypeImplSuper(base, parent)}
           |
           |""".stripMargin
      }).mkString.trim
    }
       |impl SkillObject for $cap_base {}
       |""".stripMargin.trim
  }

  private final def genTypeImplSuper(base: UserType,
                                     parent: Declaration): String = {
    val fields = IR.filter(x ⇒ x.equals(parent)).head.getAllFields.asScala

    val cap_parent = parent.getName.capital()
    val cap_base = base.getName.capital()

    e"""impl ${cap_parent}T for $cap_base {
       |    ${
      (for (f ← fields) yield {
        e"""${genGetSetImpl(f)}
           |
           |""".stripMargin
      }
      ).mkString.trim
    }
       |}""".stripMargin
  }


  //----------------------------------------
  // TypePool
  //----------------------------------------
  private final def genPool(base: UserType): String = {
    val cap_base = base.getName.capital()

    e"""//----------------------------------------
       |// ${cap_base}Pool
       |//----------------------------------------
       |${genPoolStruct(base)}
       |
       |${genPoolImpl(base)}
       |
       |${genPoolImplInstancePool(base)}
       |""".stripMargin.trim
  }

  private final def genPoolStruct(base: UserType): String = {
    val cap_base = base.getName.capital()

    e"""#[derive(Default)]
       |pub struct ${cap_base}Pool {
       |    string_block: Rc<RefCell<StringBlock>>,
       |    instances: Rc<RefCell<Vec<Ptr<SkillObject>>>>,
       |    // TODO needed after construction?
       |    fields: Vec<Box<FieldReader>>,
       |    type_name_index: usize,
       |    type_id: usize,
       |    blocks: Vec<Block>,
       |    super_pool: Option<Rc<RefCell<InstancePool>>>,
       |    base_pool: Option<Rc<RefCell<InstancePool>>>,
       |    static_count: usize,
       |    dynamic_count: usize,
       |}""".stripMargin
  }

  private final def genPoolImpl(base: UserType): String = {
    val cap_base = base.getName.capital()

    e"""impl ${cap_base}Pool {
       |    pub fn new(
       |        string_block: Rc<RefCell<StringBlock>>,
       |    ) -> ${cap_base}Pool {
       |        ${cap_base}Pool {
       |            string_block,
       |            instances: Rc::default(),
       |            fields: Vec::new(),
       |            type_name_index: 0,
       |            type_id: 0,
       |            blocks: Vec::new(),
       |            super_pool: None,
       |            base_pool: None,
       |            static_count: 0,
       |            dynamic_count: 0,
       |        }
       |    }
       |
       |    pub fn get(&self, index: usize) -> Result<WeakPtr<${cap_base}T>, SkillError> {
       |        match self.instances.borrow().get(index) {
       |            Some(obj) => match obj.nucast::<${cap_base}T>() {
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
    val cap_base = base.getName.capital()

    e"""impl InstancePool for ${cap_base}Pool {
       |    ${genPoolImplInstancePoolAddField(base)}
       |
       |    fn make_state(
       |        &mut self,
       |        file_reader: &Vec<FileReader>,
       |        string_block: &StringBlock,
       |        type_pools: &Vec<Rc<RefCell<InstancePool>>>,
       |    ) -> Result<(), SkillError> {
       |        for mut f in &mut self.fields {
       |            let mut instances = self.instances.borrow_mut();
       |            f.read(file_reader, string_block, &self.blocks, type_pools, instances.deref_mut())?;
       |        }
       |        Ok(())
       |    }
       |
       |    fn initialize(&mut self) {
       |        let mut vec = self.instances.borrow_mut();
       |        if self.is_base() {
       |            vec.reserve(self.dynamic_count); // FIXME check if dynamic count is the correct one
       |        }
       |
       |        info!(target:"SkillParsing", "Initialize {} objects", self.dynamic_count);
       |
       |        for _ in 0..self.static_count {
       |            vec.push(Ptr::new($cap_base::new()));
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
       |        error!(target:"SkillParsing", "Tried to get user instance: {}", index);
       |        unimplemented!();
       |    }
       |
       |    fn add_block(&mut self, block: Block) {
       |        self.blocks.push(block);
       |    }
       |    fn blocks(&mut self) -> &mut Vec<Block> {
       |        &mut self.blocks
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
       |    fn get_global_static_count(&self) -> usize {
       |        self.static_count
       |    }
       |    fn set_global_static_count(&mut self, count: usize) {
       |        self.static_count = count;
       |    }
       |
       |    fn get_global_cached_count(&self) -> usize {
       |        self.dynamic_count
       |    }
       |    fn set_global_cached_count(&mut self, count: usize) {
       |        self.dynamic_count = count;
       |    }
       |
       |}""".stripMargin
  }

  private final def genPoolImplInstancePoolAddField(base: UserType): String = {
    val cap_base = base.getName.capital()

    e"""// TODO is the field type important for something? apparent from the restrictions?
       |fn add_field(
       |    &mut self,
       |    name_id: usize,
       |    field_name: &str,
       |    field_type: FieldType,
       |    chunk: FieldChunk,
       |) {
       |    match field_name {
       |        ${
      (for (f ← base.getAllFields.asScala) yield {
        genPoolImplInstancePoolAddFieldField(cap_base, f)
      }
      ).mkString.trim
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
  private final def genPoolImplInstancePoolAddFieldField(cap_base: String,
                                                         field: Field): String = {
    // TODO fix snake_case
    // TODO extract to class for better names?
    val low_field = field.getName.lower()
    val reader = cap_base + field.getName.capital() + "FieldReader"

    e""""$low_field" => match field_type {
       |    ${genPoolImplInstancePoolAddFieldFieldUnwrap(reader, field.getType)}
       |    _ => panic!("Expected: ${mapTypeToUser(field.getType)} Found: {}", field_type)
       |},
       |""".stripMargin
  }

  private final def genPoolImplInstancePoolAddFieldFieldUnwrap(reader: String,
                                                               tt: Type): String = {
    tt match {
      case t@(_: SingleBaseTypeContainer | _: MapType) ⇒
        e"""${mapTypeToMagicMatch(t)} => {
           |    let mut object_readers: Vec<Rc<RefCell<InstancePool>>> = Vec::new();
           |    // TODO reserve size
           |    ${genPoolImplInstancePoolAddFieldFieldUnwrapValidate(t)}
           |    let mut reader = Box::new($reader::new(name_id, object_readers));
           |    reader.add_chunk(chunk);;
           |    self.fields.push(reader);
           |}""".stripMargin
      case t: GroundType                               ⇒
        e"""|${mapTypeToMagicMatch(t)} => {
            |   let mut reader = Box::new($reader::new(name_id));
            |   reader.add_chunk(chunk);
            |   self.fields.push(reader);
            |},""".stripMargin
      case t: UserType                                 ⇒
        e"""|${mapTypeToMagicMatch(t)} => {
            |   let mut reader = Box::new($reader::new(name_id, vec!(pool)));
            |   reader.add_chunk(chunk);
            |   self.fields.push(reader);
            |},""".stripMargin
      case _                                           ⇒
        throw new GeneratorException("Unexpected field type")
    }
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
           |match *box_v {
           |  ${genPoolImplInstancePoolAddFieldFieldUnwrapValidate(t.getBaseType)}
           |  _ => panic!("Expected: ${mapTypeToUser(t.getBaseType)} Found: {}", *box_v)
           |}""".stripMargin
      case t: SingleBaseTypeContainer ⇒
        e"""match *box_v {
           |    ${genPoolImplInstancePoolAddFieldFieldUnwrapValidate(t.getBaseType)}
           |    _ => panic!("Expected: ${mapTypeToUser(t.getBaseType)} Found: {}", *box_v)
           |}""".stripMargin
      case t: MapType                 ⇒
        e"""${genPoolImplInstancePoolAddFieldFieldUnwrapValidateMap(t.getBaseTypes.asScala.toList)}
           |""".stripMargin
      case t: GroundType              ⇒
        e"""|${mapTypeToMagicMatch(t)} => {},
            |""".stripMargin
      case _: UserType                ⇒
        e"""|FieldType::User(object_reader, _) => object_readers.push(object_reader.clone()),
            |""".stripMargin
      case _                          ⇒ throw new GeneratorException("Unexpected field type")
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
    val cap_base = base.getName.capital()

    for (field ← base.getAllFields.asScala) {
      val cap_name = cap_base + field.getName.capital()

      ret.append(
                  e"""//----------------------------------------
                     |// ${cap_name}FieldReader
                     |//----------------------------------------
                     |${genFieldReaderType(base, field)}
                     |
                     |${genFieldReaderImpl(base, field)}
                     |
                     |${genFieldReaderImplFieldReader(base, field, Stream.iterate(0)(_ + 1).iterator)}
                     |
                     |""".stripMargin
                )
    }
    ret.mkString.trim
  }

  private final def genFieldReaderType(base: UserType,
                                       field: Field): String = {
    val cap_name = base.getName.capital() + field.getName.capital()
    e"""struct ${cap_name}FieldReader {
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
    val cap_name = base.getName.capital() + field.getName.capital()

    // TODO really check if an object has to be read (e.g. map<int,int> doesn't need to)
    field.getType match {
      case _: GroundType ⇒
        e"""impl ${cap_name}FieldReader {
           |    fn new(
           |        name_id: usize,
           |    ) -> ${cap_name}FieldReader {
           |        ${cap_name}FieldReader {
           |            name_id,
           |            chunks: Vec::new(),
           |        }
           |    }
           |}""".stripMargin
      case _             ⇒
        e"""impl ${cap_name}FieldReader {
           |    fn new(
           |        name_id: usize,
           |        object_reader: Vec<Rc<RefCell<InstancePool>>>
           |    ) -> ${cap_name}FieldReader {
           |        ${cap_name}FieldReader {
           |            name_id,
           |            chunks: Vec::new(),
           |            object_reader,
           |        }
           |    }
           |}""".stripMargin
    }
  }

  private final def genFieldReaderImplFieldReader(base: UserType,
                                                  field: Field,
                                                  user: Iterator[Int]): String = {
    val cap_base = base.getName.capital()
    val cap_name = cap_base + field.getName.capital()
    val low_field = field.getName.lower()

    // TODO implement container reader

    e"""impl FieldReader for ${cap_name}FieldReader {
       |    fn read(
       |        &mut self,
       |        file_reader: &Vec<FileReader>,
       |        string_block: &StringBlock,
       |        blocks: &Vec<Block>,
       |        type_pools: &Vec<Rc<RefCell<InstancePool>>>,
       |        instances: &mut [Ptr<SkillObject>],
       |    ) -> Result<(), SkillError> {
       |        let mut block_index = BlockIndex::from(0);
       |
       |        for chunk in self.chunks.iter() {
       |            match chunk {
       |                FieldChunk::Declaration(chunk) => {
       |                    block_index += chunk.appearance - 1;
       |
       |                    let mut reader = file_reader[blocks[block_index.block].block.block].clone();
       |                    block_index += 1;
       |
       |                    if chunk.count > 0 {
       |                        for block in blocks.iter().take(chunk.appearance.block) {
       |                            for obj in instances.iter()
       |                                .skip(block.bpo + 1)
       |                                .take(block.dynamic_count)
       |                            {
       |                                match obj.nucast::<${cap_base}T>() {
       |                                    Some(obj) =>
       |                                        obj.borrow_mut().set_$low_field(${
      genFieldReaderImplFieldReaderRead(field.getType, user)
    }),
       |                                    None => panic!("Casting error"), // FIXME
       |                                }
       |                            }
       |                        }
       |                    }
       |                },
       |                FieldChunk::Continuation(chunk) => {
       |                    let mut reader = file_reader[blocks[block_index.block].block.block].clone();
       |                    block_index += 1;
       |
       |                    if chunk.count > 0 {
       |                        for obj in instances.iter()
       |                            .skip(chunk.bpo + 1)
       |                            .take(chunk.count)
       |                        {
       |                            match obj.nucast::<${cap_base}T>() {
       |                                Some(obj) =>
       |                                    obj.borrow_mut().set_$low_field(${
      genFieldReaderImplFieldReaderRead(field.getType, user)
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
      // FIXME Strings have to be obtained from the string pool
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
           |        if let Some(object) = type_pools[pool + 1]
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
           |    Ptr::new(vec)
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
           |    Ptr::new(vec)
           |}
           |""".stripMargin
      case t: ListType                            ⇒
        e"""{
           |    let elements = reader.read_v64()? as usize;
           |    let mut list = LinkedList::new();
           |    for _ in 0..elements {
           |        list.push_back(${genFieldReaderImplFieldReaderRead(t.getBaseType, user)});
           |    }
           |    Ptr::new(list)
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
           |    Ptr::new(set)
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
           |            .nucast::<${t.getName.camel()}T>()
           |        {
           |            Some(object)
           |        } else {
           |            panic!("Failed to cast object to:${t.getName.camel()}T")
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
         |    Ptr::new(map)
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
    case _: ConstantLengthArrayType ⇒ s"FieldType::BuildIn(${mapTypeToMagic(t)}(length, box_v))"
    case _: MapType                 ⇒ s"FieldType::BuildIn(${mapTypeToMagic(t)}(ref key_box_v, ref box_v))"
    case _@(_: VariableLengthArrayType | _: ListType | _: SetType)
                                    ⇒ s"FieldType::BuildIn(${mapTypeToMagic(t)}(box_v))"
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


  private final def isPtr(t: Type): Boolean = t match {
    case t: GroundType ⇒ t.getSkillName match {
      case "string" ⇒ true
      case _        ⇒ false
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
