/*  ___ _  ___ _ _                                                            *\
** / __| |/ (_) | |       The SKilL Generator                                 **
** \__ \ ' <| | | |__     (c) 2013-18 University of Stuttgart                 **
** |___/_|\_\_|_|____|    see LICENSE                                         **
\*                                                                            */
package de.ust.skill.generator.rust

import de.ust.skill.generator.common.IndenterLaw._

trait SkillFileMaker extends GeneralOutputMaker {
  abstract override def make {
    super.make

    makeSource()
  }

  private def makeSource() {
    val out = files.open("src/skill_file.rs")

    out.write(
               e"""${genUsage()}
                  §
                  §${genSkillFile()}
                  §
                  §${genSkillFileBuilder()}
                  §""".stripMargin('§')
             )

    out.close()
  }

  //----------------------------------------
  // Usage
  //----------------------------------------
  private final def genUsage(): String = {
    val ret = new StringBuilder()

    ret.append(
                e"""use common::internal::*;
                   §use common::internal::io::*;
                   §use common::*;
                   §use common::error::*;
                   §
                   §use memmap::Mmap;
                   §
                   §use std::cell::RefCell;
                   §use std::error::Error;
                   §use std::rc::Rc;
                   §
                   §""".stripMargin('§')
              )

    for (base ← IR) {
      val mod = snakeCase(storagePool(base))

      ret.append(
                  e"""use $mod::${storagePool(base)};
                     §use $mod::${traitName(base)};
                     §use $mod::${name(base)};
                     §""".stripMargin('§')
                )
    }
    ret.mkString.trim
  }

  //----------------------------------------
  // SkillFile
  //----------------------------------------
  private final def genSkillFile(): String = {
    e"""//----------------------------------------
       §// SkillFile
       §//----------------------------------------
       §${genSkillFileStruct()}
       §
       §${genSkillFileImpl()}
       §""".stripMargin('§').trim
  }

  private final def genSkillFileStruct(): String = {
    // FIXME pub(crate) fields
    e"""pub struct SkillFile {
       §    file: Rc<RefCell<std::fs::File>>,
       §    block_reader: Rc<RefCell<Vec<FileReader>>>,
       §    type_pool: TypeBlock,
       §    string_pool: StringPool,${
      (for (base ← IR) yield {
        e"""
           §${field(base)}: Rc<RefCell<${storagePool(base)}>>,""".stripMargin('§')
      }).mkString
    }
       §    foreign_pools: Vec<Rc<RefCell<foreign::Pool>>>
       §}""".stripMargin('§')
  }

  private final def genSkillFileImpl(): String = {
    e"""impl SkillFile {
       §
       §    pub(crate) fn block_reader(&self) -> std::cell::Ref<Vec<FileReader>> {
       §        self.block_reader.borrow()
       §    }
       §
       §    pub(crate) fn type_pool(&self) -> &TypeBlock {
       §        &self.type_pool
       §    }
       §
       §    pub fn strings(&self) -> &StringPool {
       §        &self.string_pool
       §    }
       §    pub fn strings_mut(&mut self) -> &mut StringPool {
       §        &mut self.string_pool
       §    }
       §
       §    ${
      (for (base ← IR) yield {
        e"""pub fn ${field(base)}(&self) -> std::cell::Ref<${storagePool(base)}> {
           §    self.${field(base)}.borrow()
           §}
           §pub fn ${field(base)}_mut(&self) -> std::cell::RefMut<${storagePool(base)}> {
           §    self.${field(base)}.borrow_mut()
           §}
           §
           §""".stripMargin('§')
      }).mkString.trim
    }
       §
       §    pub fn delete(&self, instance: WeakPtr<SkillObject>) -> Result<(), SkillFail> {
       §        match instance.upgrade() {
       §            Some(instance) => {
       §                // NOTE parameter + (base_array || new_instances)
       §                if instance.weak_count() != 1 || instance.strong_count() > 1 {
       §                    return Err(SkillFail::user(UserFail::DeleteInUse { id: instance.borrow().get_skill_id() }));
       §                }
       §                let mut proxy = self.type_pool.pools()[instance.borrow().skill_type_id() - 32].borrow_mut();
       §                proxy.pool_mut().delete(instance);
       §            }
       §            None => {}
       §        }
       §        Ok(())
       §    }
       §
       §    pub fn delete_strong(&self, instance: Ptr<SkillObject>) -> Result<(), SkillFail> {
       §        // NOTE parameter + (base_array || new_instances)
       §        if instance.weak_count() != 0 || instance.strong_count() > 2 {
       §            return Err(SkillFail::user(UserFail::DeleteInUse { id: instance.borrow().get_skill_id() }));
       §        }
       §        let mut proxy = self.type_pool.pools()[instance.borrow().skill_type_id() - 32].borrow_mut();
       §        proxy.pool_mut().delete(instance);
       §        Ok(())
       §    }
       §
       §    /// This will delete an instance without checking if somewhere in the state another instance uses this one
       §    pub fn delete_force(&self, instance: WeakPtr<SkillObject>) {
       §        match instance.upgrade() {
       §            Some(instance) => {
       §                let mut proxy = self.type_pool.pools()[instance.borrow().skill_type_id() - 32].borrow_mut();
       §                proxy.pool_mut().delete(instance);
       §            }
       §            None => {}
       §        }
       §    }
       §
       §    pub fn open(file: &str) -> Result<Self, SkillFail> {
       §        debug!(
       §            target: "SkillWriting",
       §            "Start opening"
       §        );
       §        let f = match ::std::fs::OpenOptions::new()
       §            .read(true)
       §            .write(true)
       §            .open(&file)
       §        {
       §            Ok(f) => Ok(f),
       §            Err(e) => Err(SkillFail::user(UserFail::FailedToOpenFile {
       §                file: file.to_owned(),
       §                why: e.description().to_owned(),
       §            })),
       §        }?;
       §        let string_pool = Rc::new(RefCell::new(StringBlock::new()));
       §        let mut type_pool = TypeBlock::new();
       §        let mut file_builder = SkillFileBuilder::new(string_pool.clone());
       §        let mut data_chunk_reader = Vec::new();
       §
       §        let meta = match f.metadata() {
       §            Ok(m) => Ok(m),
       §            Err(e) => Err(SkillFail::user(UserFail::FailedToOpenFile {
       §                file: file.to_owned(),
       §                why: e.description().to_owned(),
       §            })),
       §        }?;
       §
       §        if meta.len() != 0 {
       §            let mmap = match unsafe { Mmap::map(&f) }{
       §                Ok(m) => Ok(m),
       §                Err(e) => Err(SkillFail::internal(InternalFail::FailedToCreateMMap {
       §                    why: e.description().to_owned(),
       §                })),
       §            }?;
       §            let rmmap = Rc::new(mmap);
       §            let mut block_index = BlockIndex::from(0);
       §            {
       §                let mut reader = FileReader::from(rmmap.clone());
       §
       §                loop {
       §                    debug!(
       §                          target: "SkillParsing", "Block: {:?} Reader:{:?}",
       §                          block_index,
       §                          reader
       §                    );
       §
       §                    // TODO implement blocks
       §                    string_pool.borrow_mut().read_string_pool(&mut reader)?;
       §                    type_pool.read_type_pool(
       §                        block_index,
       §                        &mut reader,
       §                        &mut file_builder,
       §                        &string_pool,
       §                        &mut data_chunk_reader,
       §                    )?;
       §                    if reader.is_empty() {
       §                        break;
       §                    }
       §                    block_index += 1;
       §                }
       §            }
       §        }
       §        file_builder.allocate(&mut type_pool)?;
       §        file_builder.initialize(
       §            &type_pool,
       §            &data_chunk_reader,
       §            &string_pool.borrow(),
       §        )?;
       §        file_builder.complete();
       §        let mut sf = SkillFile {
       §            file: Rc::new(RefCell::new(f)),
       §            block_reader: Rc::new(RefCell::new(data_chunk_reader)),
       §            type_pool,
       §            string_pool: StringPool::new(string_pool),${
      (for (base ← IR) yield {
        e"""
           §${field(base)}: file_builder.${field(base)}.unwrap(),""".stripMargin('§')
      }).mkString
    }
       §            foreign_pools: file_builder.foreign_pools,
       §        };
       §        debug!(
       §            target:"SkillWriting",
       §            "Done opening"
       §        );
       §        Ok(sf)
       §    }
       §
       §    pub fn create(file: &str) -> Result<Self, SkillFail> {
       §        debug!(
       §            target: "SkillWriting",
       §            "Start creating"
       §        );
       §        let f = match ::std::fs::OpenOptions::new()
       §            .write(true)
       §            .read(true)
       §            .create(true)
       §            .open(&file)
       §        {
       §            Ok(f) => Ok(f),
       §            Err(e) => Err(SkillFail::user(UserFail::FailedToCreateFile {
       §                file: file.to_owned(),
       §                why: e.description().to_owned(),
       §            })),
       §        }?;
       §        let string_pool = Rc::new(RefCell::new(StringBlock::new()));
       §        let mut type_pool = TypeBlock::new();
       §        let mut file_builder = SkillFileBuilder::new(string_pool.clone());
       §        let mut data_chunk_reader = Vec::new();
       §
       §        file_builder.allocate(&mut type_pool)?;
       §        file_builder.initialize(
       §            &type_pool,
       §            &data_chunk_reader,
       §            &string_pool.borrow(),
       §        )?;
       §        file_builder.complete();
       §        let mut sf = SkillFile {
       §            file: Rc::new(RefCell::new(f)),
       §            block_reader: Rc::new(RefCell::new(data_chunk_reader)),
       §            type_pool,
       §            string_pool: StringPool::new(string_pool),${
      (for (base ← IR) yield {
        e"""
           §${field(base)}: file_builder.${field(base)}.unwrap(),""".stripMargin('§')
      }).mkString
    }
       §            foreign_pools: file_builder.foreign_pools,
       §        };
       §        debug!(
       §            target: "SkillWriting",
       §            "Done creating"
       §        );
       §        Ok(sf)
       §    }
       §
       §    pub fn write(&mut self) -> Result<(), SkillFail> {
       §        debug!(
       §            target: "SkillWriting",
       §            "Start writing"
       §        );
       §        // invariant -> size queries are constant time
       §        self.type_pool.set_invariant(true);
       §
       §        // Load foreign fields
       §        for pool in self.type_pool.pools().iter() {
       §            pool.borrow().pool().deserialize(self)?;
       §        }
       §
       §        // check
       §        self.check()?;
       §
       §        // reorder
       §        let local_bpos = self.type_pool.compress()?;
       §
       §        let mut writer = FileWriter::new(self.file.clone());
       §        self.string_pool.string_block().borrow_mut().write_block(&mut writer)?;
       §        self.type_pool.write_block(&mut writer, &local_bpos)?;
       §
       §        self.type_pool.set_invariant(false);
       §        debug!(
       §            target:"SkillWriting",
       §            "Done writing"
       §        );
       §        Ok(())
       §    }
       §
       §    pub fn close(mut self) -> Result<(), SkillFail> {${
      "" // TODO check if more has to be done?
    }
       §        debug!(
       §            target: "SkillWriting",
       §            "Start closing"
       §        );
       §        self.write()?;
       §        debug!(
       §            target:"SkillWriting",
       §            "Done closing"
       §        );
       §        Ok(())
       §    }
       §
       §    pub fn check(&self) -> Result<(), SkillFail> {${
      "" // TODO implement check
    }
       §        Ok(())
       §    }
       §}""".stripMargin('§')
  }

  //----------------------------------------
  // SkillFileBuilder
  //----------------------------------------
  private final def genSkillFileBuilder(): String = {
    e"""//----------------------------------------
       §// SkillFileBuilder
       §//----------------------------------------
       §${genSkillFileBuilderStruct()}
       §
       §${genSkillFileBuilderImpl()}
       §
       §${genSkillFileBuilderImplPoolMaker()}
       §""".stripMargin('§').trim
  }

  private final def genSkillFileBuilderStruct(): String = {
    e"""pub(crate) struct SkillFileBuilder {
       §    ${
      (for (base ← IR) yield {
        e"""pub(crate) ${field(base)}: Option<Rc<RefCell<${storagePool(base)}>>>,
           §""".stripMargin('§')
      }).mkString.trim
    }
       §    foreign_pools: Vec<Rc<RefCell<foreign::Pool>>>,
       §    string_pool: Rc<RefCell<StringBlock>>,
       §}""".stripMargin('§')
  }

  private final def genSkillFileBuilderImpl(): String = {
    e"""impl SkillFileBuilder {
       §    pub(crate) fn new(string_pool: Rc<RefCell<StringBlock>>) -> SkillFileBuilder {
       §        SkillFileBuilder {
       §            ${
      (for (base ← IR) yield {
        e"""${field(base)}: None,
           §""".stripMargin('§')
      }).mkString.trim
    }
       §            foreign_pools: Vec::new(),
       §            string_pool,
       §        }
       §    }
       §
       §    fn allocate(&mut self, type_pool: &mut TypeBlock) -> Result<(), SkillFail> {
       §        self.string_pool.borrow_mut().finalize();
       §        ${
      (for (base ← IR) yield {
        e"""if self.${field(base)}.is_none() {
           §    let name = self.string_pool.borrow().lit().${field(base)};
           §    let name = self.string_pool.borrow_mut().add(name);
           §    let pool = Rc::new(RefCell::new(
           §        ${storagePool(base)}::new(
           §            self.string_pool.clone(),
           §            name,
           §            type_pool.len() + 32,
           §        )
           §    ));${
          if (base.getSuperType != null) {
            e"""
               §self.${field(base.getSuperType)}.as_ref().unwrap().borrow_mut().pool_mut().add_sub(pool.clone());
               §pool.borrow_mut().pool_mut().set_super(self.${field(base.getSuperType)}.as_ref().unwrap().clone());"""
              .stripMargin('§')
          } else {
            ""
          }
        }
           §    self.${field(base)} = Some(pool.clone());
           §    type_pool.add(pool);
           §}
           §""".stripMargin('§')
      }).mkString.trim
    }
       §        ${
      (for (base ← IR) yield {
        e"""self.${field(base)}.as_ref().unwrap().borrow_mut().pool_mut().allocate();${
          if (base.getBaseType.equals(base)) {
            e"""
               §self.${field(base)}.as_ref().unwrap().borrow_mut().pool_mut().set_next_pool(None);""".stripMargin('§')
          } else {
            ""
          }
        }
           §""".stripMargin('§')
      }).mkString.trim
    }
       §        for pool in self.foreign_pools.iter() {
       §            pool.borrow_mut().pool_mut().allocate();
       §            if pool.borrow().pool().is_base() {
       §                pool.borrow_mut().pool_mut().set_next_pool(None);
       §            }
       §        }
       §        Ok(())
       §    }
       §
       §    fn initialize(
       §        &self,
       §        type_pool: &TypeBlock,
       §        file_reader: &Vec<FileReader>,
       §        string_pool: &StringBlock,
       §    ) -> Result<(), SkillFail> {
       §        type_pool.initialize(string_pool, file_reader)?;
       §        Ok(())
       §    }
       §
       §    fn complete(&mut self) {
       §        ${
      (for (base ← IR) yield {
        e"""self.${field(base)}.as_ref().unwrap().borrow_mut().complete(&self);
           §""".stripMargin('§')
      }).mkString.trim
    }
       §    }
       §}""".stripMargin('§')
  }

  private final def genSkillFileBuilderImplPoolMaker(): String = {
    e"""impl PoolMaker for SkillFileBuilder {
       §    fn make_pool(
       §        &mut self,
       §        type_name: &Rc<SkillString>,
       §        type_id: usize,
       §        super_pool: Option<Rc<RefCell<PoolProxy>>>,
       §    ) -> Result<Rc<RefCell<PoolProxy>>, SkillFail> {
       §        ${
      (for (base ← IR) yield {
        e"""if type_name.as_str() == self.string_pool.borrow().lit().${field(base)}  {
           §    if self.${field(base)}.is_none() {
           §        self.${field(base)} = Some(Rc::new(RefCell::new(
           §            ${storagePool(base)}::new(
           §                self.string_pool.clone(),
           §                type_name.clone(),
           §                type_id,
           §            )
           §        )));
           §
           §        if let Some(super_pool) = super_pool {
           §            let super_name = {
           §                let tmp = super_pool.borrow();
           §                tmp.pool().name().as_str().to_owned()
           §            };
           §            ${
          if (base.getSuperType == null) {
            e"""return Err(SkillFail::internal(InternalFail::UnexpectedSuperType {
               §    base: self.string_pool.borrow().lit().${field(base)},
               §    super_name
               §}));
               §""".stripMargin('§').trim
          } else {
            e"""if super_name.as_str() != self.string_pool.borrow().lit().${field(base.getSuperType)} {
               §    return Err(SkillFail::internal(InternalFail::WrongSuperType {
               §        base: self.string_pool.borrow().lit().${field(base)},
               §        expected: self.string_pool.borrow().lit().${field(base.getSuperType)},
               §        found: super_name,
               §    }));
               §} else {
               §    super_pool.borrow_mut().pool_mut().add_sub(self.${field(base)}.as_ref().unwrap().clone());
               §    self.${field(base)}.as_ref().unwrap().borrow_mut().pool_mut().set_super(super_pool);
               §}
               §""".stripMargin('§').trim
          }
        }
           §        } ${
          if (base.getSuperType != null) {
            e"""else {
               §    return Err(SkillFail::internal(InternalFail::MissingSuperType {
               §        base: self.string_pool.borrow().lit().${field(base)},
               §        expected: self.string_pool.borrow().lit().${field(base.getSuperType)},
               §    }));
               §}
               §""".stripMargin('§').trim
          } else {
            ""
          }
        }
           §    }
           §    Ok(self.${field(base)}.as_ref().unwrap().clone())
           §} else """.stripMargin('§')
      }).mkString
    }{
       §            for pool in self.foreign_pools.iter() {
       §                if pool.borrow().pool().get_type_id() == type_id {
       §                    return Ok(pool.clone());
       §                }
       §            }
       §            let pool = Rc::new(RefCell::new(foreign::Pool::new(
       §                type_name.clone(),
       §                type_id,
       §                super_pool.clone(),
       §            )));
       §            if let Some(super_pool) = super_pool {
       §                super_pool.borrow_mut().pool_mut().add_sub(pool.clone());
       §                pool.borrow_mut().pool_mut().set_super(super_pool);
       §            }
       §            self.foreign_pools.push(pool.clone());
       §            Ok(pool)
       §        }
       §    }
       §
       §    fn get_pool(&self, type_name_index: usize) -> Option<Rc<RefCell<PoolProxy>>> {
       §        ${
      (for (base ← IR) yield {
        e"""if self.${field(base)}.is_some()
           §    && type_name_index == self.${field(base)}.as_ref().unwrap().borrow().pool().name().get_id()
           §{
           §    return Some(self.${field(base)}.as_ref().unwrap().clone());
           §} else """.stripMargin('§')
      }).mkString
    }{
       §            for pool in self.foreign_pools.iter() {
       §                if pool.borrow().pool().name().get_id() == type_name_index {
       §                    return Some(pool.clone());
       §                }
       §            }
       §        }
       §        None
       §    }
       §}""".stripMargin('§')
  }
}
