/*  ___ _  ___ _ _                                                            *\
** / __| |/ (_) | |       The SKilL Generator                                 **
** \__ \ ' <| | | |__     (c) 2013-18 University of Stuttgart                 **
** |___/_|\_\_|_|____|    see LICENSE                                         **
\*                                                                            */
package de.ust.skill.generator.rust

import de.ust.skill.generator.common.IndenterLaw._
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
                    §
                    §${genType(base)}
                    §
                    §${genPool(base)}
                    §
                    §${genFieldDeclaration(base)}
                    §""".stripMargin('§'))
      out.close()
    }
  }

  //----------------------------------------
  // Usage
  //----------------------------------------
  private final def genUsage(base: UserType): String = {
    // TODO Sort
    e"""use common::internal::InstancePool;
       §use common::internal::LazyFieldDeclaration;
       §use common::internal::ObjectReader;
       §use common::internal::skill_object;
       §use common::internal::SkillObject;
       §use common::internal::UndefinedObject;
       §use common::io::{Block, FileWriter, BlockIndex, DeclarationFieldChunk, FileReader, FieldDeclaration, FieldChunk, BuildInType, FieldType};
       §use common::StringBlock;
       §use common::SkillError;
       §use common::SkillString;
       §use common::{Ptr, WeakPtr};
       §
       §use skill_file::SkillFileBuilder;
       §
       §${getUsageStd()}
       §
       §${getUsageUser(base)}
       §""".stripMargin('§')
  }.trim

  private final def getUsageUser(base: UserType): String = {
    IR
    .filterNot(t ⇒ t.equals(base))
    .toArray
    .map(t ⇒ s"use ${snakeCase(storagePool(t))}::*;\n")
    .sorted
    .mkString
  }.trim

  private final def getUsageStd(): String = {
    e"""use std::cell::Cell;
       §use std::cell::RefCell;
       §use std::collections::HashMap;
       §use std::collections::HashSet;
       §use std::collections::LinkedList;
       §use std::ops::DerefMut;
       §use std::rc::Rc;
       §""".stripMargin('§')
  }

  //----------------------------------------
  // Type
  //----------------------------------------
  private final def genType(base: UserType): String = {
    e"""//----------------------------------------
       §// ${base.getName} aka ${name(base)}
       §//----------------------------------------
       §${genTypeStruct(base)}
       §
       §${genTypeTrait(base)}
       §
       §${genTypeImpl(base)}
       §""".stripMargin('§')
  }.trim

  private final def genTypeStruct(base: UserType): String = {
    e"""#[derive(Default, Debug,  PartialEq)]
       §pub struct ${name(base)} {
       §    id: Cell<usize>,
       §    ${
      (for (f ← base.getAllFields.asScala) yield {
        e"""${name(f)}: ${mapType(f.getType)},
           §""".stripMargin('§')
      }).mkString.trim
    }
       §}""".stripMargin('§')
  }

  private final def genTypeTrait(base: UserType): String = {
    var com = comment(base)
    if (!com.isEmpty) {
      com += "\n"
    }

    e"""${com}pub trait ${traitName(base)} : ${
      if (base.getSuperType != null) {
        traitName(base.getSuperType)
      } else {
        "SkillObject"
      }
    } {
       §    ${
      (for (f ← base.getFields.asScala) yield {
        var com = comment(f)
        if (!com.isEmpty) {
          com += "\n"
        }

        e"""${com}fn get_${name(f)}(&self) -> ${
          if (returnByRef(f.getType)) {
            "&" + mapType(f.getType)
          } else {
            mapType(f.getType)
          }
        };
           §${com}fn set_${name(f)}(&mut self, ${name(f)}: ${mapType(f.getType)});
           §
           §""".stripMargin('§')
      }).mkString.trim
    }
       §}""".stripMargin('§')
  }.trim

  private final def genGetSetImpl(field: Field): String = {
    e"""fn get_${name(field)}(&self) -> ${
      if (returnByRef(field.getType)) {
        "&" + mapType(field.getType)
      } else {
        mapType(field.getType)
      }
    } {
       §    ${
      if (returnByRef(field.getType)) {
        e"&self.${name(field)}"
      } else {
        e"self.${name(field)}"
      }
    }
       §}
       §fn set_${name(field)}(&mut self, value: ${mapType(field.getType)}) {
       §    self.${name(field)} = value;
       §}""".stripMargin('§')
  }

  private final def genTypeImpl(base: UserType): String = {
    // gen New
    e"""impl ${name(base)} {
       §    pub fn new(id: usize) -> ${name(base)} {
       §        ${name(base)} {
       §            id: Cell::new(id),
       §            ${
      (for (f ← base.getAllFields.asScala) yield {
        e"""${name(f)}: ${defaultValue(f)},
           §""".stripMargin('§')
      }).mkString.trim
    }
       §        }
       §    }
       §}
       §
       §impl ${traitName(base)} for ${name(base)} {
       §    ${ // Impl base
      (for (f ← base.getFields.asScala) yield {
        e"""${genGetSetImpl(f)}
           §
           §""".stripMargin('§')
      }).mkString.trim
    }
       §}${
      // Impl super
      var parent = base.getSuperType
      val ret = new StringBuilder()

      while (parent != null) {
        ret.append(
                    e"""${genTypeImplSuper(base, parent)}
                       §
                       §""".stripMargin('§')
                  )
        parent = parent.getSuperType
      }
      if (ret.nonEmpty) {
        s"\n\n${ret.mkString.trim}"
      } else {
        ""
      }
    }
       §
       §impl SkillObject for ${name(base)} {
       §    fn get_skill_id(&self) -> usize {
       §        self.id.get()
       §    }
       §    fn set_skill_id(&self, id: usize) {
       §        if id == skill_object::DELETE {
       §            panic!("Tried to set the skill id to a reserved value")
       §        }
       §        self.id.set(id);
       §    }
       §
       §    fn mark_for_pruning(&self) {
       §        self.id.set(skill_object::DELETE);
       §    }
       §    fn to_prune(&self) -> bool {
       §        self.id.get() == skill_object::DELETE
       §    }
       §}
       §""".stripMargin('§').trim
  }

  private final def genTypeImplSuper(base: UserType,
                                     parent: UserType): String = {
    e"""impl ${traitName(parent)} for ${name(base)} {
       §    ${
      (for (f ← parent.getFields.asScala) yield {
        e"""${genGetSetImpl(f)}
           §
           §""".stripMargin('§')
      }).mkString.trim
    }
       §}""".stripMargin('§')
  }


  //----------------------------------------
  // TypePool
  //----------------------------------------
  private final def genPool(base: UserType): String = {
    e"""//----------------------------------------
       §// ${base.getName}Pool aka ${storagePool(base)}
       §//----------------------------------------
       §${genPoolStruct(base)}
       §
       §${genPoolImpl(base)}
       §
       §${genPoolImplInstancePool(base)}
       §""".stripMargin('§')
  }.trim

  private final def genPoolStruct(base: UserType): String = {
    e"""#[derive(Default)]
       §pub struct ${storagePool(base)} {
       §    string_block: Rc<RefCell<StringBlock>>,
       §    instances: Rc<RefCell<Vec<Ptr<SkillObject>>>>,
       §    own_static_instances: Vec<Ptr<SkillObject>>,
       §    own_new_instances: Vec<Ptr<SkillObject>>,
       §    fields: Vec<Box<FieldDeclaration>>,
       §    name: Rc<SkillString>,
       §    type_id: usize,
       §    blocks: Vec<Block>,
       §    super_pool: Option<Rc<RefCell<InstancePool>>>,
       §    sub_pools: Vec<Rc<RefCell<InstancePool>>>,
       §    base_pool: Option<Rc<RefCell<InstancePool>>>,
       §    next_pool: Option<Rc<RefCell<InstancePool>>>,
       §    static_count: usize,
       §    dynamic_count: usize,
       §    cached_count: usize,
       §    deleted_count: usize,
       §    type_hierarchy_height: usize,
       §    invariant: bool,
       §}""".stripMargin('§')
  }

  private final def genPoolImpl(base: UserType): String = {
    e"""impl ${storagePool(base)} {
       §    pub fn new(
       §        string_block: Rc<RefCell<StringBlock>>,
       §        name: Rc<SkillString>,
       §        type_id: usize,
       §    ) -> ${storagePool(base)} {
       §        ${storagePool(base)} {
       §            string_block,
       §            instances: Rc::default(),
       §            own_static_instances: Vec::new(),
       §            own_new_instances: Vec::new(),
       §            fields: Vec::new(),
       §            name,
       §            type_id,
       §            blocks: Vec::new(),
       §            super_pool: None,
       §            base_pool: None,
       §            next_pool: None,
       §            sub_pools: Vec::new(),
       §            static_count: 0,
       §            dynamic_count: 0,
       §            cached_count: 0,
       §            deleted_count: 0,
       §            type_hierarchy_height: 0,
       §            invariant: false,
       §        }
       §    }
       §
       §    pub fn get(&self, index: usize) -> Result<Ptr<${traitName(base)}>, SkillError> {
       §        if index == 0 {
       §            panic!("Skill instance index starts at 1 not 0");
       §        }
       §        match self.instances.borrow().get(index - 1) {
       §            Some(obj) => {
       §                if obj.borrow().get_skill_id() == skill_object::DELETE {
       §                    panic!("Tried to access instance that is marked for deletion.");
       §                }
       §                match obj.nucast::<${traitName(base)}>() {
       §                    Some(obj) => Ok(obj.clone()),
       §                    None => panic!("Bad cast"),
       §                }
       §            },
       §            None => Err(SkillError::BadSkillObjectID),
       §        }
       §    }
       §
       §    pub fn id(&self) -> usize {
       §        self.type_id
       §    }
       §
       §    pub fn add(&mut self) -> Ptr<${name(base)}> {
       §        let ret = Ptr::new(${name(base)}::new(0));
       §        self.own_new_instances.push(ret.clone());
       §        ret
       §    }
       §
       §    pub fn delete(&mut self, ${field(base)}: Ptr<${name(base)}>) {${
      "" // NOTE this is wrong after a compress but we consume the state on write, so its fine
    }
       §        // NOTE this is wrong after a compress but we consume the state on write, so its fine
       §        // NOTE base array + own (book) array + parameter = 3 strong counts
       §        //      in case of a deserialized instance
       §        //      own (book) array + parameter = 2 strong count in case of a new instance
       §        if ${field(base)}.weak_count() != 0
       §            || (${field(base)}.borrow().get_skill_id() != 0 && ${field(base)}.strong_count() > 3)
       §            || ${field(base)}.strong_count() > 2
       §        {
       §            panic!("Tried to mark object for pruning that s still in use");
       §        }
       §        ${field(base)}.borrow().mark_for_pruning();
       §        self.deleted_count += 1;
       §    }
       §}""".stripMargin('§')
  }

  private final def genPoolImplInstancePool(base: UserType): String = {
    e"""impl InstancePool for ${storagePool(base)} {
       §    ${genPoolImplInstancePoolAddField(base)}
       §
       §    fn initialize(
       §        &self,
       §        file_reader: &Vec<FileReader>,
       §        string_block: &StringBlock,
       §        type_pools: &Vec<Rc<RefCell<InstancePool>>>,
       §    ) -> Result<(), SkillError> {
       §        for f in self.fields.iter() {
       §            let instances = self.instances.borrow();
       §            f.read(
       §                file_reader,
       §                string_block,
       §                &self.blocks,
       §                type_pools,
       §                &instances
       §            )?;
       §        }
       §        Ok(())
       §    }
       §
       §    fn allocate(&mut self) {
       §        let mut vec = self.instances.borrow_mut();
       §        if self.is_base() {${
      "" // TODO add garbage type
    }
       §            let tmp = Ptr::new(UndefinedObject::new(0));
       §            info!(
       §                target:"SkillParsing",
       §                "Allocate space for:${base.getName} aka ${name(base)} amount:{}",
       §                self.get_global_cached_count(),
       §            );
       §            trace!(
       §                target:"SkillParsing",
       §                "Allocate space for:${name(base)} with:{:?}",
       §                tmp,
       §            );
       §
       §            vec.reserve(self.get_global_cached_count()); // FIXME check if dynamic count is the correct one
       §            // TODO figure out a better way - set_len doesn't wrk as dtor will be called on garbage data
       §
       §            for _ in 0..self.get_global_cached_count() {
       §                vec.push(tmp.clone());
       §            }
       §        }
       §        self.own_static_instances.reserve(self.static_count);
       §
       §        info!(
       §            target:"SkillParsing",
       §            "Initialize ${base.getName} aka ${name(base)} id:{}",
       §            self.get_type_id(),
       §        );
       §
       §        for block in self.blocks.iter() {
       §            let begin = block.bpo + 1;
       §            let end = begin + block.static_count;
       §            for id in begin..end {
       §                trace!(
       §                    target:"SkillParsing",
       §                    "${name(base)} id:{} block:{:?}",
       §                    id,
       §                    block,
       §                );
       §
       §                self.own_static_instances.push(Ptr::new(${name(base)}::new(id)));
       §                vec[id - 1] = self.own_static_instances.last().unwrap().clone();
       §            }
       §        }
       §    }
       §    fn has_field(&self, name_id: usize) -> bool {
       §        for f in & self.fields {
       §            if f.name().get_skill_id() == name_id {
       §                return true
       §            }
       §        }
       §        false
       §    }
       §
       §    fn field_amount(&self) -> usize {
       §        self.fields.len()
       §    }
       §
       §    fn add_chunk_to(&mut self, name_id: usize, chunk: FieldChunk) {
       §        for f in &mut self.fields {
       §            if f.name().get_skill_id() == name_id {
       §                f.add_chunk(chunk);
       §                return;
       §            }
       §        }
       §        panic!("No field of id:{}", name_id);
       §    }
       §
       §    fn set_type_id(&mut self, id: usize) {
       §        self.type_id = id;
       §    }
       §    fn get_type_id(&self) -> usize {
       §        self.type_id
       §    }
       §
       §    fn name(&self) -> &SkillString {
       §        &self.name
       §    }
       §
       §    fn get_base_vec(&self) -> Rc<RefCell<Vec<Ptr<SkillObject>>>> {
       §        self.instances.clone()
       §    }
       §
       §    fn read_object(&self, index: usize) -> Result<Ptr<SkillObject>, SkillError> {
       §        assert!(index >= 1);
       §        info!(
       §            target:"SkillParsing",
       §            "read user instance:{} from:{}",
       §            index,
       §            self.instances.borrow().len(),
       §        );
       §        Ok(self.instances.borrow()[index - 1].clone())
       §    }
       §
       §    fn add_block(&mut self, block: Block) {
       §        self.blocks.push(block);
       §    }
       §    fn blocks(&self) -> &Vec<Block> {
       §        &self.blocks
       §    }
       §    fn blocks_mut(&mut self) -> &mut Vec<Block> {
       §        &mut self.blocks
       §    }
       §
       §    fn add_sub(&mut self, pool: Rc<RefCell<InstancePool>>) {
       §        self.sub_pools.push(pool);
       §    }
       §
       §    fn set_super(&mut self, pool: Rc<RefCell<InstancePool>>) {
       §        if pool.borrow().is_base() {
       §            self.base_pool = Some(pool.clone());
       §        } else {
       §            self.base_pool = pool.borrow().get_base(); // TODO check?
       §        }
       §        self.instances = self.base_pool.as_ref().unwrap().borrow().get_base_vec();
       §        self.super_pool = Some(pool);
       §    }
       §    fn get_super(&self) -> Option<Rc<RefCell<InstancePool>>> {
       §        self.super_pool.clone()
       §    }
       §
       §    fn get_base(&self) -> Option<Rc<RefCell<InstancePool>>> {
       §        self.base_pool.clone()
       §    }
       §    fn is_base(&self) -> bool {
       §        self.super_pool.is_none()
       §    }
       §
       §    fn get_local_static_count(&self) -> usize {
       §        if let Some(block) = self.blocks.last() {
       §            return block.static_count;
       §        }
       §        panic!();
       §    }
       §    fn set_local_static_count(&mut self, count: usize) {
       §        if let Some(block) = self.blocks.last_mut() {
       §            block.static_count = count;
       §        } else {
       §            panic!();
       §        }
       §    }
       §
       §    fn get_local_dynamic_count(&self) -> usize {
       §        if let Some(block) = self.blocks.last() {
       §            return block.dynamic_count;
       §        }
       §        panic!();
       §    }
       §
       §    fn get_local_bpo(&self) -> usize {
       §        if let Some(block) = self.blocks.last() {
       §            return block.bpo;
       §        }
       §        panic!();
       §    }
       §
       §    fn set_invariant(&mut self, invariant: bool) {
       §        if self.invariant != invariant {
       §            self.invariant = invariant;
       §            if invariant {
       §                self.cached_count = self.static_size() - self.deleted_count;
       §                for s in self.sub_pools.iter() {
       §                    let mut s = s.borrow_mut();
       §                    s.set_invariant(true);
       §                    self.cached_count += s.get_global_cached_count();
       §                }
       §            } else if self.super_pool.is_some() {
       §                self.super_pool
       §                    .as_ref()
       §                    .unwrap()
       §                    .borrow_mut()
       §                    .set_invariant(false);
       §            }
       §        }
       §    }
       §
       §    fn get_global_static_count(&self) -> usize {
       §        self.static_count
       §    }
       §    fn set_global_static_count(&mut self, count: usize) {
       §        self.static_count = count;
       §    }
       §
       §    fn get_global_cached_count(&self) -> usize {
       §        self.cached_count
       §    }
       §    fn set_global_cached_count(&mut self, count: usize) {
       §         self.cached_count = count;
       §    }
       §
       §    fn make_instance(&self, id: usize) -> Ptr<SkillObject> {
       §        trace!(
       §            target:"SkillParsing",
       §            "Create new ${name(base)}",
       §        );
       §        Ptr::new(${name(base)}::new(id))
       §    }
       §
       §    fn update_after_compress(
       §        &mut self,
       §        local_bpo: &Vec<usize>,
       §        vec: Rc<RefCell<Vec<Ptr<SkillObject>>>>,
       §    ) {
       §        self.instances = vec;
       §        self.blocks = Vec::with_capacity(1);
       §        let static_size = self.static_size();
       §        self.blocks.push(Block {
       §            block: BlockIndex::from(0), // TODO remove strong type
       §            bpo: local_bpo[self.type_id - 32],
       §            static_count: static_size,
       §            dynamic_count: self.cached_count,
       §        });
       §        trace!(
       §            target:"SkillWriting",
       §            "Updated Block:{:?}",
       §            self.blocks.last().unwrap(),
       §        );
       §    }
       §
       §    fn static_instances(&self) -> &Vec<Ptr<SkillObject>> {
       §        &self.own_static_instances
       §    }
       §    fn new_instances(&self) -> &Vec<Ptr<SkillObject>> {
       §        &self.own_new_instances
       §    }
       §
       §    fn static_size(&self) -> usize {
       §        self.static_count + self.own_new_instances.len() - self.deleted_count
       §    }
       §    fn dynamic_size(&self) -> usize {
       §        if self.invariant {
       §            self.cached_count
       §        } else {
       §            let mut ret = self.static_size();
       §            for s in self.sub_pools.iter() {
       §                ret += s.borrow().static_size();
       §            }
       §            ret
       §        }
       §    }
       §    fn deleted(&self) -> usize {
       §        self.deleted_count
       §    }
       §
       §    fn set_next_pool(&mut self, pool: Rc<RefCell<InstancePool>>) {
       §        if self.sub_pools.len() > 0 {
       §            self.next_pool = Some(self.sub_pools.first().unwrap().clone());
       §            for i in 0..self.sub_pools.len() - 1 {
       §                self.sub_pools[i].borrow_mut().set_next_pool(self.sub_pools[i + 1].clone());
       §            }
       §            self.sub_pools.last().unwrap().borrow_mut().set_next_pool(pool);
       §        } else {
       §            self.next_pool = Some(pool);
       §        }
       §    }
       §    fn get_next_pool(&self) -> Option<Rc<RefCell<InstancePool>>> {
       §        self.next_pool.clone()
       §    }
       §    fn type_hierarchy_height(&self) -> usize {
       §        self.type_hierarchy_height
       §    }
       §
       §    fn compress_field_chunks(&mut self, local_bpo: &Vec<usize>) {
       §        let total_count = self.get_global_cached_count();
       §        for f in self.fields.iter_mut() {
       §            f.compress_chunks(total_count);
       §        }
       §    }
       §    fn write_type_meta(&self, writer: &mut FileWriter, local_bpos: &Vec<usize>) {
       §        writer.write_v64(self.name().get_skill_id() as i64);
       §        writer.write_v64(self.get_local_dynamic_count() as i64);
       §        // FIXME restrictions
       §        writer.write_v64(0);
       §        if let Some(s) = self.get_super() {
       §            writer.write_v64((s.borrow().get_type_id() - 32) as i64); // TODO +1?
       §            if self.get_local_dynamic_count() != 0 {
       §                writer.write_v64(local_bpos[self.get_type_id() - 32] as i64);
       §            }
       §        } else {
       §            // tiny optimisation
       §            writer.write_i8(0);
       §        }
       §        writer.write_v64(self.field_amount() as i64);
       §    }
       §    fn write_field_meta(&mut self, writer: &mut FileWriter, mut offset: usize) -> usize {
       §        for f in self.fields.iter_mut() {
       §            offset = f.write_meta(writer, offset);
       §        }
       §        offset
       §    }
       §    fn write_field_data(&self, writer: &mut FileWriter) {
       §        for f in self.fields.iter() {
       §            f.write_data(writer)
       §        }
       §    }
       §}""".stripMargin('§')
  }

  private final def genPoolImplInstancePoolAddField(base: UserType): String = {
    e"""// TODO is the field type important for something? apparent from the restrictions?
       §fn add_field(
       §    &mut self,
       §    index: usize,
       §    field_name: Rc<SkillString>,
       §    mut field_type: FieldType,
       §    chunk: FieldChunk,
       §) {
       §    ${
      (for (f ← base.getAllFields.asScala) yield {
        genPoolImplInstancePoolAddFieldField(base, f)
      }).mkString.trim
    } {
       §        let mut reader = Box::new(LazyFieldDeclaration::new(field_name, index, field_type));
       §        reader.as_mut().add_chunk(chunk);
       §        self.fields.push(reader);
       §    }
       §}""".stripMargin('§')
  }

  // TODO do something about these stupid names
  private final def genPoolImplInstancePoolAddFieldField(base: UserType,
                                                         f: Field): String = {
    e"""if self.string_block.borrow().lit().${field(f)} == field_name.as_str() {
       §    match field_type {
       §        ${
      f.getType match {
        case t@(_: SingleBaseTypeContainer | _: MapType) ⇒
          e"""${mapTypeToMagicMatch(t)} => {
             §    let mut object_readers: Vec<Rc<RefCell<InstancePool>>> = Vec::new();
             §    // TODO reserve size
             §    ${genPoolImplInstancePoolAddFieldFieldUnwrapValidate(t)}
             §    let mut reader = Box::new(${fieldDeclaration(base, f)}::new(field_name, index, field_type, object_readers));
             §    reader.add_chunk(chunk);;
             §    self.fields.push(reader);
             §}""".stripMargin('§')
        case t: GroundType                               ⇒
          e"""${mapTypeToMagicMatch(t)} => {
             §   let mut reader = Box::new(${fieldDeclaration(base, f)}::new(field_name, index, field_type));
             §   reader.add_chunk(chunk);
             §   self.fields.push(reader);
             §},""".stripMargin('§')
        case t: UserType                                 ⇒
          e"""${mapTypeToMagicMatch(t)} => {
              §   let mut reader = Box::new(${fieldDeclaration(base, f)}::new(field_name, index, field_type, vec!(pool)));
              §   reader.add_chunk(chunk);
              §   self.fields.push(reader);
              §},""".stripMargin('§')
        case _                                           ⇒
          throw new GeneratorException("Unexpected field type")
      }
    }
       §        _ => panic!("Expected: ${mapTypeToUser(f.getType)} Found: {}", field_type)
       §    }
       §} else """.stripMargin('§')
  }

  private final def genPoolImplInstancePoolAddFieldFieldUnwrapValidate(tt: Type): String = {
    tt match {
      case t: ConstantLengthArrayType ⇒
        e"""if length != ${t.getLength} {
           §    panic!("The length of the constant length ({}) array differs form the generated one (${
          t.getLength
        })!", length);
           §}
           §match **box_v {
           §  ${genPoolImplInstancePoolAddFieldFieldUnwrapValidate(t.getBaseType)}
           §  _ => panic!("Expected: ${mapTypeToUser(t.getBaseType)} Found: {}", **box_v)
           §}""".stripMargin('§')
      case t: SingleBaseTypeContainer ⇒
        e"""match **box_v {
           §    ${genPoolImplInstancePoolAddFieldFieldUnwrapValidate(t.getBaseType)}
           §    _ => panic!("Expected: ${mapTypeToUser(t.getBaseType)} Found: {}", **box_v)
           §}""".stripMargin('§')
      case t: MapType                 ⇒
        e"""${genPoolImplInstancePoolAddFieldFieldUnwrapValidateMap(t.getBaseTypes.asScala.toList)}
           §""".stripMargin('§')
      case t: GroundType              ⇒
        e"""${mapTypeToMagicMatch(t)} => {},
           §"stripMargin('§')
      case _: UserType                ⇒
        eFieldType::User(ref object_reader, _) => object_readers.push(object_reader.clone()),
           §""".stripMargin('§')
      case _                          ⇒
        throw new GeneratorException("Unexpected field type")
    }
  }.trim

  private final def genPoolImplInstancePoolAddFieldFieldUnwrapValidateMap(tts: List[Type]): String = {
    val (key, remainder) = tts.splitAt(1)

    e"""match **key_box_v {
       §    ${genPoolImplInstancePoolAddFieldFieldUnwrapValidate(key.head)}
       §    _ => panic!("Expected: ${mapTypeToUser(key.head)} Found: {}", **key_box_v)
       §}
       §match **box_v {
       §    ${
      if (remainder.size >= 2) {
        e"""FieldType::BuildIn(BuildInType::Tmap(ref key_box_v, ref box_v)) => {
           §    ${genPoolImplInstancePoolAddFieldFieldUnwrapValidateMap(remainder)}
           §}""".stripMargin('§')
      } else {
        genPoolImplInstancePoolAddFieldFieldUnwrapValidate(remainder.head)
      }
    }
       §    _ => panic!("Expected: ${mapTypeToUser(remainder.head)} Found: {}", **box_v)
       §}
       §""".stripMargin('§')
  }.trim

  //----------------------------------------
  // FieldDeclaration
  //----------------------------------------
  private final def genFieldDeclaration(base: UserType): String = {
    val ret = new StringBuilder()

    for (field ← base.getAllFields.asScala) {
      ret.append(
                  e"""//----------------------------------------
                     §// ${base.getName.camel()}${field.getName.capital()}FieldDeclaration aka ${
                    fieldDeclaration(base,
                                      field)
                  }
                     §//----------------------------------------
                     §${genFieldDeclarationType(base, field)}
                     §
                     §${genFieldDeclarationImpl(base, field)}
                     §
                     §${genFieldDeclarationImplFieldDeclaration(base, field)}
                     §
                     §""".stripMargin('§')
                )
    }
    ret.mkString.trim
  }

  private final def genFieldDeclarationType(base: UserType,
                                            field: Field): String = {
    e"""struct ${fieldDeclaration(base, field)} {
       §    name: Rc<SkillString>,
       §    index: usize, // Index into the pool fields vector
       §    field_type: FieldType,
       §    chunks: Vec<FieldChunk>,${
      field.getType match {
        case _: GroundType ⇒
          ""
        case _             ⇒
          e"""
             §object_reader: Vec<Rc<RefCell<InstancePool>>>,""".stripMargin('§')
      }
    }
       §}""".stripMargin('§')
  }

  private final def genFieldDeclarationImpl(base: UserType,
                                            field: Field): String = {
    field.getType match {
      case _: GroundType ⇒
        e"""impl ${fieldDeclaration(base, field)} {
           §    fn new(
           §        name: Rc<SkillString>,
           §        index: usize,
           §        field_type: FieldType,
           §    ) -> ${fieldDeclaration(base, field)} {
           §        ${fieldDeclaration(base, field)} {
           §            name,
           §            index,
           §            field_type,
           §            chunks: Vec::new(),
           §        }
           §    }
           §}""".stripMargin('§')
      case _             ⇒
        e"""impl ${fieldDeclaration(base, field)} {
           §    fn new(
           §        name: Rc<SkillString>,
           §        index: usize,
           §        field_type: FieldType,
           §        object_reader: Vec<Rc<RefCell<InstancePool>>>
           §    ) -> ${fieldDeclaration(base, field)} {
           §        ${fieldDeclaration(base, field)} {
           §            name,
           §            index,
           §            field_type,
           §            chunks: Vec::new(),
           §            object_reader,
           §        }
           §    }
           §}""".stripMargin('§')
    }
  }

  private final def genFieldDeclarationImplFieldDeclaration(base: UserType,
                                                            field: Field): String = {
    e"""impl FieldDeclaration for ${fieldDeclaration(base, field)} {
       §    fn read(
       §        &self,
       §        file_reader: &Vec<FileReader>,
       §        string_block: &StringBlock,
       §        blocks: &Vec<Block>,
       §        type_pools: &Vec<Rc<RefCell<InstancePool>>>,
       §        instances: &[Ptr<SkillObject>],
       §    ) -> Result<(), SkillError> {
       §        let mut block_index = BlockIndex::from(0);
       §
       §        for chunk in self.chunks.iter() {
       §            match chunk {
       §                FieldChunk::Declaration(chunk) => {
       §                    block_index += chunk.appearance - 1;
       §
       §                    let block = &blocks[block_index.block];
       §                    let mut reader = file_reader[block.block.block].rel_view(chunk.begin, chunk.end);
       §                    block_index += 1;
       §
       §                    if chunk.count > 0 {
       §                        for block in blocks.iter().take(chunk.appearance.block) {
       §                            let mut o = 0;
       §
       §                            for obj in instances.iter()
       §                                .skip(block.bpo)
       §                                .take(block.dynamic_count)
       §                            {
       §                                info!(
       §                                    target:"SkillParsing",
       §                                    "Block:{:?} Object:{}",
       §                                    block,
       §                                    o + block.bpo,
       §                                );
       §                                o += 1;
       §                                match obj.nucast::<${traitName(base)}>() {
       §                                    Some(obj) =>
       §                                        obj.borrow_mut().set_${name(field)}(${
      genFieldDeclarationImplFieldDeclarationRead(field.getType, Stream.iterate(0)(_ + 1).iterator)
    }),
       §                                    None => panic!("Casting error"), // FIXME
       §                                }
       §                            }
       §                        }
       §                    }
       §                },
       §                FieldChunk::Continuation(chunk) => {
       §                    let block = &blocks[block_index.block];
       §                    let mut reader = file_reader[block.block.block].rel_view(chunk.begin, chunk.end);
       §                    block_index += 1;
       §
       §                    if chunk.count > 0 {
       §                        let mut o = 0;
       §                        for obj in instances.iter()
       §                            .skip(chunk.bpo)
       §                            .take(chunk.count)
       §                        {
       §                            info!(
       §                                target:"SkillParsing",
       §                                "Block:{:?} Object:{}",
       §                                block,
       §                                o + chunk.bpo,
       §                            );
       §                            o += 1;
       §
       §                            match obj.nucast::<${traitName(base)}>() {
       §                                Some(obj) =>
       §                                    obj.borrow_mut().set_${name(field)}(${
      genFieldDeclarationImplFieldDeclarationRead(field.getType, Stream.iterate(0)(_ + 1).iterator)
    }),
       §                                None => panic!("Casting error"), // FIXME
       §                            }
       §                        }
       §                    }
       §                }
       §            }
       §        }
       §        Ok(())
       §    }
       §    fn add_chunk(&mut self, chunk: FieldChunk) {
       §        self.chunks.push(chunk);
       §    }
       §    fn name(&self) -> &Rc<SkillString> {
       §        &self.name
       §    }
       §
       §    fn compress_chunks(&mut self, total_count: usize) {
       §        self.chunks = Vec::with_capacity(1);
       §        self.chunks
       §            .push(FieldChunk::Declaration(DeclarationFieldChunk {
       §                begin: 0,
       §                end: 0,
       §                count: total_count,
       §                appearance: BlockIndex::from(1),
       §            }));
       §    }
       §    fn offset(&self) -> usize {
       §        unimplemented!();
       §    }
       §    fn write_meta(&mut self, writer: &mut FileWriter, offset: usize) -> usize {
       §        writer.write_v64(self.index as i64);
       §        writer.write_v64(self.name.get_skill_id() as i64);
       §
       §        // TODO write type
       §        writer.write_i8(0); // TODO write restrictions
       §        let end_offset = offset + self.offset();
       §        writer.write_v64(end_offset as i64);
       §
       §        match self.chunks.first_mut().unwrap() {
       §            FieldChunk::Declaration(ref mut dec) => {
       §                dec.begin = offset;
       §                dec.end = end_offset;
       §            }
       §            _ => panic!("Expected an declaration chunk after compress!"),
       §        };
       §
       §        end_offset
       §    }
       §    fn write_data(&self, writer: &mut FileWriter) {
       §        unimplemented!();
       §    }
       §}""".stripMargin('§')
  }

  private final def genFieldDeclarationImplFieldDeclarationRead(base: Type,
                                                                user: Iterator[Int]): String = {
    base match {
      case t: GroundType
        if t.getName.lower().equals("string")     ⇒
        e"""string_block.get(reader.read_v64()? as usize)
           §""".stripMargin('§')
      case t: GroundType
        if t.getName.lower().equals("annotation") ⇒
        e"""{
           §    let pool = reader.read_v64()? as usize;
           §    let object = reader.read_v64()? as usize;
           §    if pool != 0 && object != 0 {
           §        if let Some(object) = type_pools[pool - 1]
           §            .borrow()
           §            .read_object(object)?
           §            .nucast::<SkillObject>()
           §        {
           §            Some(object)
           §        } else {
           §            // NOTE this should only be reached if the generated code is faulty
           §            panic!("Failed to cast object to SkillObject");
           §        }
           §    } else {
           §        None
           §    }
           §}
           §""".stripMargin('§').trim
      case t: GroundType                          ⇒
        e"""reader.read_${readName(t)}()?
           §""".stripMargin('§')
      case t: ConstantLengthArrayType             ⇒
        // TODO check that everything was read?
        e"""{
           §    let mut vec = Vec::new();
           §    vec.reserve(${t.getLength});
           §    for _ in 0..${t.getLength} {
           §        vec.push(${genFieldDeclarationImplFieldDeclarationRead(t.getBaseType, user)});
           §    }
           §    vec
           §}
           §""".stripMargin('§')
      case t: VariableLengthArrayType             ⇒
        e"""{
           §    let elements = reader.read_v64()? as usize;
           §    let mut vec = Vec::new();
           §    vec.reserve(elements);
           §    for _ in 0..elements {
           §        vec.push(${genFieldDeclarationImplFieldDeclarationRead(t.getBaseType, user)});
           §    }
           §    vec
           §}
           §""".stripMargin('§')
      case t: ListType                            ⇒
        e"""{
           §    let elements = reader.read_v64()? as usize;
           §    let mut list = LinkedList::new();
           §    for _ in 0..elements {
           §        list.push_back(${genFieldDeclarationImplFieldDeclarationRead(t.getBaseType, user)});
           §    }
           §    list
           §}
           §""".stripMargin('§')
      case t: SetType                             ⇒
        e"""{
           §    let elements = reader.read_v64()? as usize;
           §    let mut set = HashSet::new();
           §    set.reserve(elements);
           §    for _ in 0..elements {
           §        set.insert(${genFieldDeclarationImplFieldDeclarationRead(t.getBaseType, user)});
           §    }
           §    set
           §}
           §""".stripMargin('§')
      case t: MapType                             ⇒
        genFieldDeclarationImplFieldDeclarationReadMap(t.getBaseTypes.asScala.toList, user)
      case t: UserType                            ⇒
        e"""{
           §    let object = reader.read_v64()? as usize;
           §    if object != 0 {
           §        if let Some(object) = self.object_reader[${user.next()}]
           §            .borrow()
           §            .read_object(object)?
           §            .nucast::<${traitName(t)}>()
           §        {
           §            Some(object)
           §        } else {
           §            panic!("Failed to cast object to:${t.getName} aka ${traitName(t)}")
           §        }
           §    } else {
           §        None
           §    }
           §}
           §""".stripMargin('§').trim
      case _                                      ⇒
        throw new GeneratorException(s"Unknown type $base")
    }
  }.trim


  private final def genFieldDeclarationImplFieldDeclarationReadMap(tts: List[Type], user: Iterator[Int]): String = {
    val (key, remainder) = tts.splitAt(1)

    if (remainder.nonEmpty) {
      e"""{
         §    let elements = reader.read_v64()? as usize;
         §    let mut map = HashMap::new();
         §    map.reserve(elements);
         §    for _ in 0..elements {
         §        map.insert(
         §            ${genFieldDeclarationImplFieldDeclarationRead(key.head, user)},
         §            ${genFieldDeclarationImplFieldDeclarationReadMap(remainder, user)}
         §        );
         §    }
         §    map
         §}""".stripMargin('§')
    } else {
      genFieldDeclarationImplFieldDeclarationRead(key.head, user)
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
