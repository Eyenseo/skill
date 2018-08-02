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
    e"""use common::error::*;
       §use common::internal::*;
       §use common::io::magic::bytes_v64;
       §use common::io::*;
       §use common::iterator::*;
       §use common::*;
       §
       §use skill_file::SkillFile;
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
       §    skill_id: Cell<usize>,
       §    skill_type_id: usize,
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
          if (f.getType.isInstanceOf[ReferenceType] || f.getType.isInstanceOf[ContainerType]) {
            "&" + mapType(f.getType)
          } else {
            mapType(f.getType)
          }
        };${
          if (f.getType.isInstanceOf[ReferenceType] || f.getType.isInstanceOf[ContainerType]) {
            e"""
               §fn get_${name(f)}_mut(&mut self) -> &mut ${mapType(f.getType)};""".stripMargin('§')
          } else {
            ""
          }
        }
           §${com}fn set_${name(f)}(&mut self, ${name(f)}: ${mapType(f.getType)});
           §
           §""".stripMargin('§')
      }).mkString.trim
    }
       §}""".stripMargin('§')
  }.trim

  private final def genGetSetImpl(field: Field): String = {
    e"""fn get_${name(field)}(&self) -> ${
      if (field.getType.isInstanceOf[ReferenceType] || field.getType.isInstanceOf[ContainerType]) {
        "&" + mapType(field.getType)
      } else {
        mapType(field.getType)
      }
    } {
       §    ${
      if (field.getType.isInstanceOf[ReferenceType] || field.getType.isInstanceOf[ContainerType]) {
        e"&self.${name(field)}"
      } else {
        e"self.${name(field)}"
      }
    }
       §}${
      if (field.getType.isInstanceOf[ReferenceType] || field.getType.isInstanceOf[ContainerType]) {
        e"""
           §fn get_${name(field)}_mut(&mut self) -> &mut ${mapType(field.getType)} {
           §    &mut self.${name(field)}
           §}""".stripMargin('§')
      } else {
        ""
      }
    }
       §fn set_${name(field)}(&mut self, value: ${mapType(field.getType)}) {
       §    self.${name(field)} = value;
       §}""".stripMargin('§')
  }

  private final def genTypeImpl(base: UserType): String = {
    // gen New
    e"""impl ${name(base)} {
       §    pub fn new(skill_id: usize, skill_type_id: usize) -> ${name(base)} {
       §        ${name(base)} {
       §            skill_id: Cell::new(skill_id),
       §            skill_type_id,
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
       §    fn skill_type_id(&self) -> usize {
       §        self.skill_type_id
       §    }
       §
       §    fn get_skill_id(&self) -> usize {
       §        self.skill_id.get()
       §    }
       §    fn set_skill_id(&self, skill_id: usize) -> Result<(), SkillFail> {
       §        if skill_id == skill_object::DELETE {
       §            return Err(SkillFail::user(UserFail::ReservedID { id: skill_id }));
       §        }
       §        self.skill_id.set(skill_id);
       §        Ok(())
       §    }
       §
       §    fn mark_for_pruning(&self) {
       §        self.skill_id.set(skill_object::DELETE);
       §    }
       §    fn to_prune(&self) -> bool {
       §        self.skill_id.get() == skill_object::DELETE
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
       §    fields: Vec<Box<RefCell<FieldDeclaration>>>,
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
       §    pub fn complete(&mut self, file: &SkillFile) {
       §        ${
      if (base.getFields.size() > 0) {
        e"""
           §        let mut set = HashSet::with_capacity(${base.getFields.size()});
           §        let mut string_pool = self.string_block.borrow_mut();
           §        {
           §            let lit = string_pool.lit();
           §            ${
          (for (field ← base.getFields.asScala) yield {
            e"""set.insert(lit.${name(field)});
               §""".stripMargin('§')
          }).mkString.trim()
        }
           §        }
           §
           §        for f in self.fields.iter() {
           §            set.remove(f.borrow().name().as_str());
           §        }
           §
           §        ${
          (for (ft ← base.getFields.asScala) yield {
            e"""if set.contains(string_pool.lit().${name(ft)}) {
               §    let index = self.fields.len() + 1;
               §    let name = string_pool.lit().${name(ft)};${
              "" // FIXME accessing the fields of lit will create _copies_! else this would be illegal
            }
               §    self.fields.push(Box::new(RefCell::new(
               §        ${fieldDeclaration(base, ft)}::new(
               §            string_pool.add(name),
               §            index,
               §            ${mapTypeToMagicDef(ft.getType)},${
              val userTypes = collectUserTypes(ft.getType)
              if (userTypes.nonEmpty) {
                e"""
                   §vec!{
                   §    ${
                  (for (ut ← userTypes) yield {
                    e"""file.${field(ut)}.clone(),
                       §""".stripMargin('§')
                  }).mkString.trim
                }
                   §}""".stripMargin('§')
              } else {
                ""
              }
            }
               §        )
               §    )));
               §}
               §""".stripMargin('§')
          }).mkString
        }
           §""".stripMargin('§').trim
      } else {
        "// nothing to do"
      }
    }
       §    }
       §
       §    pub fn get(&self, index: usize) -> Result<Ptr<${traitName(base)}>, SkillFail> {
       §        if index == 0 {
       §            return Err(SkillFail::user(UserFail::ReservedID { id: 0 }));
       §        }
       §        match self.instances.borrow().get(index - 1) {
       §            Some(obj) => {
       §                if obj.borrow().get_skill_id() == skill_object::DELETE {
       §                    return Err(SkillFail::user(UserFail::AccessDeleted));
       §                }
       §                match obj.nucast::<${traitName(base)}>() {
       §                    Some(obj) => Ok(obj.clone()),
       §                    None => Err(SkillFail::user(UserFail::BadCastID { id:index })),
       §                }
       §            },
       §            None => Err(SkillFail::user(UserFail::UnknownObjectID { id:index })),
       §        }
       §    }
       §
       §    pub fn add(&mut self) -> Ptr<${name(base)}> {
       §        let ret = Ptr::new(${name(base)}::new(0, self.type_id));
       §        self.own_new_instances.push(ret.clone());
       §        ret
       §    }
       §
       §    pub fn delete(&mut self, ${field(base)}: Ptr<${name(base)}>) -> Result<(), SkillFail> {${
      "" // NOTE this is wrong after a compress but we consume the state on write, so its "fine"
    }
       §        // NOTE this is wrong after a compress but we consume the state on write, so its fine
       §        // NOTE base array + own (book) array + parameter = 3 strong counts
       §        //      in case of a deserialized instance
       §        //      own (book) array + parameter = 2 strong count in case of a new instance
       §        if ${field(base)}.weak_count() != 0
       §            || (${field(base)}.borrow().get_skill_id() != 0 && ${field(base)}.strong_count() > 3)
       §            || ${field(base)}.strong_count() > 2
       §        {
       §            return Err(SkillFail::user(UserFail::DeleteInUse { id: ${field(base)}.borrow().get_skill_id() }));
       §        }
       §        ${field(base)}.borrow().mark_for_pruning();
       §        self.deleted_count += 1;
       §        Ok(())
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
       §    ) -> Result<(), SkillFail> {
       §        for f in self.fields.iter() {
       §            let instances = self.instances.borrow();
       §            f.borrow().read(
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
       §            let tmp = Ptr::new(UndefinedObject::new(0, 0));
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
       §                self.own_static_instances.push(Ptr::new(${name(base)}::new(id, self.type_id)));
       §                vec[id - 1] = self.own_static_instances.last().unwrap().clone();
       §            }
       §        }
       §    }
       §    fn has_field(&self, name_id: usize) -> bool {
       §        for f in self.fields.iter() {
       §            if f.borrow().name().get_skill_id() == name_id {
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
       §    fn add_chunk_to(&mut self, field_index: usize, chunk: FieldChunk) -> Result<(), SkillFail> {
       §        for f in &mut self.fields.iter() {
       §            let mut f = f.borrow_mut();
       §            if f.index() == field_index {
       §                f.add_chunk(chunk);
       §                return Ok(());
       §            }
       §        }
       §        Err(SkillFail::internal(InternalFail::UnknownField {
       §            id: field_index,
       §        }))
       §    }
       §
       §    fn set_type_id(&mut self, id: usize) {
       §        self.type_id = id;
       §    }
       §    fn get_type_id(&self) -> usize {
       §        self.type_id
       §    }
       §
       §    fn name(&self) -> &Rc<SkillString> {
       §        &self.name
       §    }
       §
       §    fn get_base_vec(&self) -> Rc<RefCell<Vec<Ptr<SkillObject>>>> {
       §        self.instances.clone()
       §    }
       §
       §    fn read_object(&self, index: usize) -> Result<Ptr<SkillObject>, SkillFail> {
       §        if index == 0 {
       §            return Err(SkillFail::internal(InternalFail::ReservedID { id: 0 }));
       §        }
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
       §        self.type_hierarchy_height = pool.borrow().type_hierarchy_height() + 1;
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
       §        return self.blocks.last().unwrap().static_count;
       §    }
       §    fn set_local_static_count(&mut self, count: usize) {
       §        self.blocks.last_mut().unwrap().static_count = count
       §    }
       §
       §    fn get_local_dynamic_count(&self) -> usize {
       §        return self.blocks.last().unwrap().dynamic_count;
       §    }
       §
       §    fn get_local_bpo(&self) -> usize {
       §        self.blocks.last().unwrap().bpo
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
       §    fn make_instance(&self, skill_id: usize, skill_type_id: usize) -> Ptr<SkillObject> {
       §        trace!(
       §            target:"SkillParsing",
       §            "Create new ${name(base)}",
       §        );
       §        Ptr::new(${name(base)}::new(skill_id, skill_type_id))
       §    }
       §
       §    fn update_after_compress(
       §        &mut self,
       §        local_bpo: &Vec<usize>,
       §        vec: Rc<RefCell<Vec<Ptr<SkillObject>>>>,
       §    ) {
       §        self.instances = vec;
       §        self.static_count += self.own_new_instances.len();
       §        self.own_new_instances = Vec::new();
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
       §        self.static_count + self.own_new_instances.len()
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
       §    fn set_next_pool(&mut self, pool: Option<Rc<RefCell<InstancePool>>>) {
       §        if self.sub_pools.len() > 0 {
       §            self.next_pool = Some(self.sub_pools.first().unwrap().clone());
       §            for i in 0..self.sub_pools.len() - 1 {
       §                self.sub_pools[i]
       §                    .borrow_mut()
       §                    .set_next_pool(Some(self.sub_pools[i + 1].clone()));
       §            }
       §            self.sub_pools
       §                .last()
       §                .unwrap()
       §                .borrow_mut()
       §                .set_next_pool(pool);
       §        } else {
       §            self.next_pool = pool;
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
       §        for f in self.fields.iter() {
       §            f.borrow_mut().compress_chunks(total_count);
       §        }
       §    }
       §    fn write_type_meta(&self, writer: &mut FileWriter, local_bpos: &Vec<usize>) -> Result<(), SkillFail> {
       §        info!(
       §            target:"SkillWriting",
       §            "~~~Write Meta Data for ${name(base)}:{} Instances; Static:{} Dynamic:{}",
       §            self.name.as_ref(),
       §            self.get_local_static_count(),
       §            self.get_local_dynamic_count(),
       §        );
       §
       §        writer.write_v64(self.name().get_skill_id() as i64)?;
       §        writer.write_v64(self.get_local_dynamic_count() as i64)?;
       §        // FIXME restrictions
       §        writer.write_v64(0)?;
       §        if let Some(s) = self.get_super() {
       §            writer.write_v64((s.borrow().get_type_id() - 31) as i64)?;
       §            if self.get_local_dynamic_count() != 0 {
       §                writer.write_v64(local_bpos[self.get_type_id() - 32] as i64)?;
       §            }
       §        } else {
       §            // tiny optimisation
       §            writer.write_i8(0)?;
       §        }
       §        writer.write_v64(self.field_amount() as i64)?;
       §        Ok(())
       §    }
       §    fn write_field_meta(
       §        &self,
       §        writer: &mut FileWriter,
       §        iter: dynamic_data::Iter,
       §        mut offset: usize
       §    ) -> Result<usize, SkillFail> {
       §        info!(
       §            target:"SkillWriting",
       §            "~~~Write Field Meta Data for ${name(base)}:{} Fields:{}",
       §            self.name.as_ref(),
       §            self.fields.len(),
       §        );
       §        for f in self.fields.iter() {
       §            offset = f.borrow_mut().write_meta(writer, iter.clone(), offset)?;
       §        }
       §        Ok(offset)
       §    }
       §    fn write_field_data(
       §        &self,
       §        writer: &mut FileWriter,
       §        iter: dynamic_data::Iter
       §    ) -> Result<(), SkillFail> {
       §        info!(
       §            target:"SkillWriting",
       §            "~~~Write Field Data for ${name(base)}:{} Fields:{}",
       §            self.name.as_ref(),
       §            self.fields.len(),
       §        );
       §        for f in self.fields.iter() {
       §            f.borrow().write_data(writer, iter.clone())?;
       §        }
       §        Ok(())
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
       §) -> Result<(), SkillFail> {
       §    ${
      (for (f ← base.getFields.asScala) yield {
        genPoolImplInstancePoolAddFieldField(base, f)
      }).mkString.trim
    } {
       §        let mut reader = Box::new(RefCell::new(
       §            LazyFieldDeclaration::new(field_name, index, field_type)
       §        ));
       §        reader.borrow_mut().add_chunk(chunk);
       §        self.fields.push(reader);
       §    }
       §    Ok(())
       §}""".stripMargin('§')
  }

  // TODO do something about these stupid names
  private final def genPoolImplInstancePoolAddFieldField(base: UserType,
                                                         f: Field): String = {
    val userType = collectUserTypes(f.getType)
    e"""if self.string_block.borrow().lit().${field(f)} == field_name.as_str() {
       §    ${
      if (userType.isEmpty) {
        e"""match field_type {
           §    ${genPoolImplInstancePoolAddFieldFieldUnwrapValidate(f.getType)},
           §    _ => Err(SkillFail::internal(InternalFail::BadFieldType {
           §        expected: "${mapTypeToUser(f.getType)}",
           §        found: format!("{}", field_type)
           §    })),
           §}?;
           §let mut reader = Box::new(RefCell::new(${fieldDeclaration(base, f)}::new(
           §    field_name,
           §    index,
           §    field_type,
           §)));
           §reader.borrow_mut().add_chunk(chunk);;
           §self.fields.push(reader);
           §""".stripMargin('§')
      } else {
        e"""let mut object_readers: Vec<Rc<RefCell<InstancePool>>> = Vec::new();
           §object_readers.reserve(${userType.size});
           §match field_type {
           §    ${genPoolImplInstancePoolAddFieldFieldUnwrapValidate(f.getType)},
           §    _ => Err(SkillFail::internal(InternalFail::BadFieldType {
           §        expected: "${mapTypeToUser(f.getType)}",
           §        found: format!("{}", field_type)
           §    })),
           §}?;
           §let mut reader = Box::new(RefCell::new(${fieldDeclaration(base, f)}::new(
           §    field_name,
           §    index,
           §    field_type,
           §    object_readers,
           §)));
           §reader.borrow_mut().add_chunk(chunk);;
           §self.fields.push(reader);
           §""".stripMargin('§')
      }
    }
       §} else """.stripMargin('§')
  }

  private final def genPoolImplInstancePoolAddFieldFieldUnwrapValidate(tt: Type): String = {
    tt match {
      case t: ConstantLengthArrayType ⇒
        e"""${mapTypeToMagicMatch(t)} => {
           §    if length != ${t.getLength} {
           §        return Err(SkillFail::internal(InternalFail::BadConstantLength {
           §            expected: ${t.getLength},
           §            found: length as usize,
           §        }));
           §    }
           §    match **box_v {
           §        ${genPoolImplInstancePoolAddFieldFieldUnwrapValidate(t.getBaseType)},
           §        _ => Err(SkillFail::internal(InternalFail::BadFieldType {
           §            expected: "${mapTypeToUser(t.getBaseType)}",
           §            found: format!("{}", **box_v)
           §        })),
           §    }
           §}""".stripMargin('§')
      case t: SingleBaseTypeContainer ⇒
        e"""${mapTypeToMagicMatch(t)} => {
           §    match **box_v {
           §        ${genPoolImplInstancePoolAddFieldFieldUnwrapValidate(t.getBaseType)},
           §        _ => Err(SkillFail::internal(InternalFail::BadFieldType {
           §            expected: "${mapTypeToUser(t.getBaseType)}",
           §            found: format!("{}", **box_v)
           §        })),
           §    }
           §}""".stripMargin('§')
      case t: MapType                 ⇒
        e"""${mapTypeToMagicMatch(t)} => {
           §    ${genPoolImplInstancePoolAddFieldFieldUnwrapValidateMap(t.getBaseTypes.asScala.toList)}
           §}
           §""".stripMargin('§')
      case t: GroundType              ⇒
        e"""${mapTypeToMagicMatch(t)} => Ok(())
           §""".stripMargin('§')
      case _: UserType                ⇒
        e"""FieldType::User(ref object_reader) => {
           §    object_readers.push(object_reader.clone());
           §    Ok(())
           §}
           §""".stripMargin('§')
      case _                          ⇒
        throw new GeneratorException("Unexpected field type")
    }
  }.trim

  private final def genPoolImplInstancePoolAddFieldFieldUnwrapValidateMap(tts: List[Type]): String = {
    val (key, remainder) = tts.splitAt(1)

    e"""match **key_box_v {
       §    ${genPoolImplInstancePoolAddFieldFieldUnwrapValidate(key.head)},
       §    _ => Err(SkillFail::internal(InternalFail::BadFieldType {
       §        expected: "${mapTypeToUser(key.head)}",
       §        found: format!("{}", **key_box_v)
       §    })),
       §}?;
       §match **box_v {
       §    ${
      if (remainder.size >= 2) {
        e"""FieldType::BuildIn(BuildInType::Tmap(ref key_box_v, ref box_v)) => {
           §    ${genPoolImplInstancePoolAddFieldFieldUnwrapValidateMap(remainder)}
           §}""".stripMargin('§')
      } else {
        genPoolImplInstancePoolAddFieldFieldUnwrapValidate(remainder.head)
      }
    },
       §    _ => Err(SkillFail::internal(InternalFail::BadFieldType {
       §        expected: "${mapTypeToUser(remainder.head)}",
       §        found: format!("{}", **box_v)
       §    })),
       §}
       §""".stripMargin('§')
  }.trim

  //----------------------------------------
  // FieldDeclaration
  //----------------------------------------
  private final def genFieldDeclaration(base: UserType): String = {
    val ret = new StringBuilder()

    for (field ← base.getFields.asScala) {
      ret.append(
                  e"""//----------------------------------------
                     §// ${base.getName.camel() + field.getName.capital()}FieldDeclaration aka ${
                    fieldDeclaration(base, field)
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
      val userType = collectUserTypes(field.getType)
      if (userType.isEmpty) {
        ""
      } else {
        e"""
           §object_reader: Vec<Rc<RefCell<InstancePool>>>,""".stripMargin('§')
      }
    }
       §}""".stripMargin('§')
  }

  private final def genFieldDeclarationImpl(base: UserType,
                                            field: Field): String = {
    val userType = collectUserTypes(field.getType)
    if (userType.isEmpty) {
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
    } else {
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
                                                            f: Field): String = {
    e"""impl FieldDeclaration for ${fieldDeclaration(base, f)} {
       §    fn read(
       §        &self,
       §        file_reader: &Vec<FileReader>,
       §        string_block: &StringBlock,
       §        blocks: &Vec<Block>,
       §        type_pools: &Vec<Rc<RefCell<InstancePool>>>,
       §        instances: &[Ptr<SkillObject>],
       §    ) -> Result<(), SkillFail> {
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
       §                                        obj.borrow_mut().set_${name(f)}(${
      genFieldDeclarationImplFieldDeclarationRead(f.getType, Stream.iterate(0)(_ + 1).iterator)
    }),
       §                                    None => return Err(SkillFail::internal(InternalFail::BadCast)),
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
       §                                    obj.borrow_mut().set_${name(f)}(${
      genFieldDeclarationImplFieldDeclarationRead(f.getType, Stream.iterate(0)(_ + 1).iterator)
    }),
       §                                None => return Err(SkillFail::internal(InternalFail::BadCast)),
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
       §    fn index(&self) -> usize {
       §        self.index
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
       §    fn offset(&self, iter: dynamic_data::Iter) -> usize {
       §        ${genFieldDeclarationImplFieldDeclarationOffset(base, f)}
       §    }
       §    fn write_meta(&mut self, writer: &mut FileWriter, iter: dynamic_data::Iter, offset: usize) -> Result<usize, SkillFail> {
       §        info!(
       §            target:"SkillWriting",
       §            "~~~~Write Field Meta Data for Field:{}",
       §            self.name.as_ref(),
       §        );
       §        writer.write_v64(self.index as i64)?;
       §        writer.write_v64(self.name.get_skill_id() as i64)?;
       §        writer.write_field_type(&self.field_type)?;
       §        writer.write_i8(0)?; // TODO write restrictions
       §        let end_offset = offset + self.offset(iter.clone());
       §        info!(
       §            target:"SkillWriting",
       §            "~~~~Field:{} end offset:{}",
       §            self.name.as_ref(),
       §            end_offset,
       §        );
       §        writer.write_v64(end_offset as i64)?;
       §
       §        match self.chunks.first_mut().unwrap() {
       §            FieldChunk::Declaration(ref mut dec) => {
       §                dec.begin = offset;
       §                dec.end = end_offset;
       §                Ok(())
       §            }
       §            _ => Err(SkillFail::internal(InternalFail::BadChunk)),
       §        }?;
       §
       §        Ok(end_offset)
       §    }
       §    fn write_data(
       §        &self,
       §        writer: &mut FileWriter,
       §        iter: dynamic_data::Iter
       §    ) -> Result<(), SkillFail> {
       §       info!(
       §            target:"SkillWriting",
       §            "~~~~Write Field Data for Field:{}",
       §            self.name.as_ref(),
       §        );
       §        let mut writer = match self.chunks.first().unwrap() {
       §            FieldChunk::Declaration(ref chunk) => writer.rel_view(chunk.begin, chunk.end)?,
       §            FieldChunk::Continuation(_) => panic!()
       §        };
       §        for i in iter {
       §            let tmp = i.nucast::<${traitName(base)}>().unwrap();
       §            let tmp = tmp.borrow(); // borrowing madness
       §            let val = tmp.get_${field(f)}();
       §            ${genFieldDeclarationImplFieldDeclarationWrite(f.getType)}
       §        }
       §        Ok(())
       §    }
       §}""".stripMargin('§')
  }

  private final def genFieldDeclarationImplFieldDeclarationOffset(base: Type, f: Field): String = {
    f.getType match {
      case ft: GroundType              ⇒
        ft.getSkillName match {
          case "bool" | "i8" ⇒
            e"""iter.count()
               §""".stripMargin('§')
          case "i16"         ⇒
            e"""2 * iter.count()
               §""".stripMargin('§')
          case "i32" | "f32" ⇒
            e"""4 * iter.count()
               §""".stripMargin('§')
          case "f64" | "i64" ⇒
            e"""8 * iter.count()
               §""".stripMargin('§')
          case "v64"         ⇒
            e"""let mut offset = 0;
               §for i in iter {
               §    let tmp = i.nucast::<${traitName(base)}>().unwrap();
               §    let tmp = tmp.borrow(); // borrowing madness
               §    offset += bytes_v64(tmp.get_${field(f)}() as i64);
               §}
               §offset
               §""".stripMargin('§')
          case "string"      ⇒
            e"""let mut offset = 0;
               §for i in iter {
               §    let tmp = i.nucast::<${traitName(base)}>().unwrap();
               §    let tmp = tmp.borrow(); // borrowing madness
               §    offset += bytes_v64(tmp.get_${field(f)}().get_skill_id() as i64);
               §}
               §offset
               §""".stripMargin('§')
          case "annotation"  ⇒
            e"""let mut offset = 0;
               §for i in iter {
               §    let tmp = i.nucast::<${traitName(base)}>().unwrap();
               §    let tmp = tmp.borrow(); // borrowing madness
               §    offset += match tmp.get_${field(f)}() {
               §        Some(ref val) => {
               §          bytes_v64(val.borrow().skill_type_id() as i64)
               §              + bytes_v64(val.borrow().get_skill_id() as i64)
               §        },
               §        None => 2,
               §    };
               §}
               §offset
               §""".stripMargin('§')
          case _             ⇒
            throw new GeneratorException(s"Unhandled type $ft")
        }
      case ft: ConstantLengthArrayType ⇒
        e"""let mut offset = 0;
           §for i in iter {
           §    let tmp = i.nucast::<${traitName(base)}>().unwrap();
           §    let tmp = tmp.borrow(); // borrowing madness
           §    for val in tmp.get_${field(f)}().iter() {
           §        offset += ${genFieldDeclarationImplFieldDeclarationOffsetInner(ft.getBaseType)};
           §    }
           §}
           §offset
           §""".stripMargin('§')
      case ft: SingleBaseTypeContainer ⇒
        e"""let mut offset = 0;
           §for i in iter {
           §    let tmp = i.nucast::<${traitName(base)}>().unwrap();
           §    let tmp = tmp.borrow(); // borrowing madness
           §    offset += bytes_v64(tmp.get_${field(f)}().len() as i64);
           §    for val in tmp.get_${field(f)}().iter() {
           §        offset += ${genFieldDeclarationImplFieldDeclarationOffsetInner(ft.getBaseType)};
           §    }
           §}
           §offset
           §""".stripMargin('§')
      case ft: MapType                 ⇒
        e"""let mut offset = 0;
           §for i in iter {
           §    let tmp = i.nucast::<${traitName(base)}>().unwrap();
           §    let tmp = tmp.borrow(); // borrowing madness
           §    let val = tmp.get_${field(f)}();
           §    ${genFieldDeclarationImplFieldDeclarationOffsetMap(ft.getBaseTypes.asScala.toList)}
           §}
           §offset
           §""".stripMargin('§')
      case _: UserType                 ⇒
        e"""let mut offset = 0;
           §for i in iter {
           §    let tmp = i.nucast::<${traitName(base)}>().unwrap();
           §    let tmp = tmp.borrow(); // borrowing madness
           §    offset += match tmp.get_${field(f)}() {
           §        Some(ref val) => bytes_v64(val.borrow().get_skill_id() as i64),
           §        None => 1,
           §    };
           §}
           §offset
           §""".stripMargin('§')
      case ft                          ⇒
        throw new GeneratorException(s"Unknown type $ft")
    }
  }.trim

  private final def genFieldDeclarationImplFieldDeclarationOffsetInner(base: Type): String = {
    base match {
      case t: GroundType              ⇒
        t.getSkillName match {
          case "bool" | "i8" ⇒
            e"""1
               §""".stripMargin('§')
          case "i16"         ⇒
            e"""2
               §""".stripMargin('§')
          case "i32" | "f32" ⇒
            e"""4
               §""".stripMargin('§')
          case "f64" | "i64" ⇒
            e"""8
               §""".stripMargin('§')
          case "v64"         ⇒
            e"""bytes_v64(*val as i64)
               §""".stripMargin('§')
          case "string"      ⇒
            e"""bytes_v64(val.get_skill_id()  as i64)
               §""".stripMargin('§')
          case "annotation"  ⇒
            e"""match val {
               §    Some(ref val) => bytes_v64(val.borrow().get_skill_id() as i64),
               §    None => 1,
               §}
               §""".stripMargin('§')
          case _             ⇒
            throw new GeneratorException(s"Unhandled type $t")
        }
      case t: ConstantLengthArrayType ⇒
        e"""{
           §    let mut offset = 0;
           §    for val in val {
           §        offset += ${genFieldDeclarationImplFieldDeclarationOffsetInner(t.getBaseType)};
           §    }
           §    offset
           §}
           §""".stripMargin('§')
      case t: SingleBaseTypeContainer ⇒
        e"""{
           §    let mut offset = 0;
           §    offset += bytes_v64(val.len() as i64);
           §    for val in val {
           §        offset += ${genFieldDeclarationImplFieldDeclarationOffsetInner(t.getBaseType)};
           §    }
           §    offset
           §}
           §""".stripMargin('§')
      case t: MapType                 ⇒
        e"""{
           §    let mut offset = 0;
           §    ${genFieldDeclarationImplFieldDeclarationOffsetMap(t.getBaseTypes.asScala.toList)}
           §}
           §offset
           §""".stripMargin('§')
      case _: UserType                ⇒
        e"""match val {
           §    Some(ref val) => bytes_v64(val.borrow().get_skill_id() as i64),
           §    None => 1,
           §}
           §""".stripMargin('§')
      case t                          ⇒
        throw new GeneratorException(s"Unknown type $t")
    }
  }.trim


  private final def genFieldDeclarationImplFieldDeclarationOffsetMap(tts: List[Type]): String = {
    val (key, remainder) = tts.splitAt(1)

    if (remainder.size > 1) {
      e"""offset += bytes_v64(val.len() as i64);
         §for (key, val) in val.iter() {
         §    {
         §        let val = key;
         §        offset += ${genFieldDeclarationImplFieldDeclarationOffsetInner(key.head)};
         §    }
         §    ${genFieldDeclarationImplFieldDeclarationOffsetMap(remainder)}
         §}
         §""".stripMargin('§')
    } else {
      e"""offset += bytes_v64(val.len() as i64);
         §for (key, val) in val.iter() {
         §    {
         §        let val = key;
         §        offset += ${genFieldDeclarationImplFieldDeclarationOffsetInner(key.head)};
         §    }
         §    offset += ${genFieldDeclarationImplFieldDeclarationOffsetInner(remainder.head)};
         §}
         §""".stripMargin('§')
    }
  }.trim

  private final def genFieldDeclarationImplFieldDeclarationWrite(ft: Type): String = {
    ft match {
      case ft: GroundType              ⇒
        ft.getSkillName match {
          case "bool"       ⇒
            e"""writer.write_bool(val)?;
               §""".stripMargin('§')
          case "i8"         ⇒
            e"""writer.write_i8(val)?;
               §""".stripMargin('§')
          case "i16"        ⇒
            e"""writer.write_i16(val)?;
               §""".stripMargin('§')
          case "i32"        ⇒
            e"""writer.write_i32(val)?;
               §""".stripMargin('§')
          case "f32"        ⇒
            e"""writer.write_f32(val)?;
               §""".stripMargin('§')
          case "f64"        ⇒
            e"""writer.write_f64(val)?;
               §""".stripMargin('§')
          case "i64"        ⇒
            e"""writer.write_i64(val)?;
               §""".stripMargin('§')
          case "v64"        ⇒
            e"""writer.write_v64(val as i64)?;
               §""".stripMargin('§')
          case "string"     ⇒
            e"""writer.write_v64(val.get_skill_id() as i64)?;
               §""".stripMargin('§')
          case "annotation" ⇒
            e"""match val {
               §    Some(ref val) => {
               §        writer.write_v64(val.borrow().skill_type_id() as i64)?;
               §        writer.write_v64(val.borrow().get_skill_id() as i64)?;
               §    },
               §    None => writer.write_i8(0)?,
               §}
               §""".stripMargin('§')
          case _            ⇒
            throw new GeneratorException(s"Unhandled type $ft")
        }
      case ft: ConstantLengthArrayType ⇒
        e"""for val in val.iter() {
           §    ${genFieldDeclarationImplFieldDeclarationWriteInner(ft.getBaseType)};
           §}
           §""".stripMargin('§')
      case ft: SingleBaseTypeContainer ⇒
        e"""writer.write_v64(val.len() as i64)?;
           §for val in val.iter() {
           §    ${genFieldDeclarationImplFieldDeclarationWriteInner(ft.getBaseType)};
           §}
           §""".stripMargin('§')
      case ft: MapType                 ⇒
        genFieldDeclarationImplFieldDeclarationWriteMap(ft.getBaseTypes.asScala.toList)
      case _: UserType                 ⇒
        e"""match val {
           §    Some(ref val) => writer.write_v64(val.borrow().get_skill_id() as i64)?,
           §    None => writer.write_i8(0)?,
           §}
           §""".stripMargin('§')
      case _                           ⇒
        throw new GeneratorException(s"Unknown type $ft")
    }
  }.trim

  private final def genFieldDeclarationImplFieldDeclarationWriteInner(ft: Type): String = {
    ft match {
      case ft: GroundType              ⇒
        ft.getSkillName match {
          case "bool"       ⇒
            e"""writer.write_bool(*val)?;
               §""".stripMargin('§')
          case "i8"         ⇒
            e"""writer.write_i8(*val)?;
               §""".stripMargin('§')
          case "i16"        ⇒
            e"""writer.write_i16(*val)?;
               §""".stripMargin('§')
          case "i32"        ⇒
            e"""writer.write_i32(*val)?;
               §""".stripMargin('§')
          case "f32"        ⇒
            e"""writer.write_f32(*val)?;
               §""".stripMargin('§')
          case "f64"        ⇒
            e"""writer.write_f64(*val)?;
               §""".stripMargin('§')
          case "i64"        ⇒
            e"""writer.write_i64(*val)?;
               §""".stripMargin('§')
          case "v64"        ⇒
            e"""writer.write_v64(*val as i64)?;
               §""".stripMargin('§')
          case "string"     ⇒
            e"""writer.write_v64(val.get_skill_id() as i64)?;
               §""".stripMargin('§')
          case "annotation" ⇒
            e"""match val {
               §    Some(ref val) => writer.write_v64(val.borrow().get_skill_id() as i64)?,
               §    None => writer.write_i8(0)?,
               §}
               §""".stripMargin('§')
          case _            ⇒
            throw new GeneratorException(s"Unhandled type $ft")
        }
      case ft: ConstantLengthArrayType ⇒
        e"""for val in val.iter() {
           §    ${genFieldDeclarationImplFieldDeclarationWrite(ft.getBaseType)};
           §}
           §""".stripMargin('§')
      case ft: SingleBaseTypeContainer ⇒
        e"""writer.write_v64(val.len() as i64)?;
           §for val in val.iter() {
           §    ${genFieldDeclarationImplFieldDeclarationWrite(ft.getBaseType)};
           §}
           §""".stripMargin('§')
      case ft: MapType                 ⇒
        genFieldDeclarationImplFieldDeclarationWriteMap(ft.getBaseTypes.asScala.toList)
      case _: UserType                 ⇒
        e"""match val {
           §    Some(ref val) => writer.write_v64(val.borrow().get_skill_id() as i64)?,
           §    None => writer.write_i8(0)?,
           §}
           §""".stripMargin('§')
      case _                           ⇒
        throw new GeneratorException(s"Unknown type $ft")
    }
  }.trim

  private final def genFieldDeclarationImplFieldDeclarationWriteMap(tts: List[Type]): String = {
    val (key, remainder) = tts.splitAt(1)

    if (remainder.size > 1) {
      e"""writer.write_v64(val.len() as i64)?;
         §for (key, val) in val.iter() {
         §    {
         §        let val = key;
         §        ${genFieldDeclarationImplFieldDeclarationWriteInner(key.head)};
         §    }
         §    ${genFieldDeclarationImplFieldDeclarationWriteMap(remainder)}
         §}
         §""".stripMargin('§')
    } else {
      e"""writer.write_v64(val.len() as i64)?;
         §for (key, val) in val.iter() {
         §    {
         §        let val = key;
         §        ${genFieldDeclarationImplFieldDeclarationWriteInner(key.head)};
         §    }
         §    ${genFieldDeclarationImplFieldDeclarationWriteInner(remainder.head)}
         §}
         §""".stripMargin('§')
    }
  }.trim

  private final def genFieldDeclarationImplFieldDeclarationRead(base: Type,
                                                                user: Iterator[Int]): String = {
    base match {
      case t: GroundType
        if t.getName.lower().equals("string")     ⇒
        e"""string_block.get(reader.read_v64()? as usize)?
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
           §            Ok(Some(object))
           §        } else {
           §            Err(SkillFail::internal(InternalFail::BadCast))
           §        }
           §    } else {
           §        Ok(None)
           §    }
           §}?
           §""".stripMargin('§').trim
      case t: GroundType                          ⇒
        e"""reader.read_${readName(t)}()?
           §""".stripMargin('§')
      case t: ConstantLengthArrayType             ⇒
        // TODO check that everything was read?
        e"""{
           §    let mut arr:${mapType(t)} = ${defaultValue(t)};
           §    for i in 0..${t.getLength} {
           §        arr[i] = ${genFieldDeclarationImplFieldDeclarationRead(t.getBaseType, user)};
           §    }
           §    arr
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
           §            Ok(Some(object))
           §        } else {
           §            return Err(SkillFail::internal(InternalFail::BadCast))
           §        }
           §    } else {
           §        Ok(None)
           §    }
           §}?
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
         §            ${genFieldDeclarationImplFieldDeclarationReadMap(remainder, user)},
         §        );
         §    }
         §    map
         §}""".stripMargin('§')
    } else {
      genFieldDeclarationImplFieldDeclarationRead(key.head, user)
    }
  }

  // TODO better names
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
    case _: UserType                ⇒ e"""FieldType::User(ref pool)"""
    case _                          ⇒ e"""FieldType::BuildIn(${mapTypeToMagic(t)})"""
  }

  private final def mapTypeToMagicDef(t: Type): String = t match {
    case t: ConstantLengthArrayType ⇒
      e"""FieldType::BuildIn(${mapTypeToMagic(t)}(
         §    ${t.getLength},
         §    Box::new(
         §        ${mapTypeToMagicDef(t.getBaseType)}
         §    ),
         §))""".stripMargin('§')
    case t: MapType                 ⇒
      e"""FieldType::BuildIn(${mapTypeToMagic(t)}(
         §    ${mapTypeToMagicDefMap(t, t.getBaseTypes.asScala.toList)}
         §))""".stripMargin('§').trim
    case t: SingleBaseTypeContainer ⇒
      e"""FieldType::BuildIn(${mapTypeToMagic(t)}(
         §    Box::new(
         §        ${mapTypeToMagicDef(t.getBaseType)}
         §    ),
         §))""".stripMargin('§')
    case t: UserType                ⇒
      e"FieldType::User(file.${field(t)}.clone())"
    case _                          ⇒
      e"""FieldType::BuildIn(${mapTypeToMagic(t)})"""
  }

  private final def mapTypeToMagicDefMap(t: Type, tts: List[Type]): String = {
    val (key, remainder) = tts.splitAt(1)

    if (remainder.size > 1) {
      e"""Box::new(
         §    ${mapTypeToMagicDef(key.head)}
         §),
         §Box::new(FieldType::BuildIn(${mapTypeToMagic(t)}(
         §    ${mapTypeToMagicDefMap(t, remainder)},
         §)))
         §""".stripMargin('§').trim
    } else {
      e"""Box::new(
         §    ${mapTypeToMagicDef(key.head)}
         §),
         §Box::new(
         §    ${mapTypeToMagicDef(remainder.head)}
         §)
         §""".stripMargin('§').trim
    }
  }

  private final def collectUserTypes(t: Type): List[UserType] = t match {
    case t: MapType                 ⇒
      collectUserTypesMap(t, t.getBaseTypes.asScala.toList)
    case t: SingleBaseTypeContainer ⇒
      collectUserTypes(t.getBaseType)
    case t: UserType                ⇒
      List[UserType](t)
    case _                          ⇒
      List()
  }

  private final def collectUserTypesMap(t: Type, tts: List[Type]): List[UserType] = {
    val (key, remainder) = tts.splitAt(1)

    if (remainder.size > 1) {
      collectUserTypes(key.head) ::: collectUserTypesMap(t, remainder)
    } else {
      collectUserTypes(key.head) ::: collectUserTypes(remainder.head)
    }
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

  protected def defaultValue(t: Type): String
}
