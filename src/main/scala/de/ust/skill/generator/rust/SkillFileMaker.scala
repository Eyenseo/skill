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
                e"""use common::internal::{InstancePool, ObjectReader, SkillObject, UndefinedPool};
                   §use common::io::{FieldDeclaration, BlockIndex, FieldType, FileWriter, FileReader};
                   §use common::PoolMaker;
                   §use common::Ptr;
                   §use common::error::*;
                   §use common::SkillString;
                   §use common::StringBlock;
                   §use common::TypeBlock;
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
    e"""pub struct SkillFile {
       §    file: Rc<RefCell<std::fs::File>>,
       §    type_pool: TypeBlock,
       §    pub strings: Rc<RefCell<StringBlock>>,${
      (for (base ← IR) yield {
        e"""
           §pub ${field(base)}: Rc<RefCell<${storagePool(base)}>>,""".stripMargin('§')
      }).mkString
    }
       §}""".stripMargin('§')
  }

  private final def genSkillFileImpl(): String = {
    e"""impl SkillFile {
       §    fn complete(&mut self) {
       §        ${
      (for (base ← IR) yield {
        e"""self.${field(base)}.borrow_mut().complete(&self);
           §""".stripMargin('§')
      }).mkString.trim
    }
       §    }
       §
       §    pub fn open(file: &str) -> Result<Self, SkillFail> {
       §        info!(
       §            target: "SkillWriting",
       §            "Start opening"
       §        );
       §        let f = match ::std::fs::OpenOptions::new()
       §            .read(true)
       §            .write(true)
       §            .open(&file)
       §        {
       §            Ok(f) => Ok(f),
       §            Err(e) => Err(SkillFail::internal(InternalFail::FailedToOpenFile {
       §                file: file.to_owned(),
       §                why: e.description().to_owned(),
       §            })),
       §        }?;
       §        let string_block = Rc::new(RefCell::new(StringBlock::new()));
       §        let mut type_pool = TypeBlock::new();
       §        let mut file_builder = SkillFileBuilder::new(string_block.clone());
       §        let mut data_chunk_reader = Vec::new();
       §
       §        let meta = match f.metadata() {
       §            Ok(m) => Ok(m),
       §            Err(e) => Err(SkillFail::internal(InternalFail::FailedToOpenFile {
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
       §                    info!(
       §                          target: "SkillParsing", "Block: {:?} Reader:{:?}",
       §                          block_index,
       §                          reader
       §                    );
       §
       §                    // TODO implement blocks
       §                    string_block.borrow_mut().read_string_block(&mut reader)?;
       §                    type_pool.read_type_block(
       §                        block_index,
       §                        &mut reader,
       §                        &mut file_builder,
       §                        &string_block,
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
       §            &string_block.borrow(),
       §        )?;
       §        let mut sf = SkillFile {
       §            file: Rc::new(RefCell::new(f)),
       §            type_pool,
       §            strings: string_block,${
      (for (base ← IR) yield {
        e"""
           §${field(base)}: file_builder.${field(base)}.unwrap(),""".stripMargin('§')
      }).mkString
    }
       §        };
       §        sf.complete();
       §        info!(
       §            target:"SkillWriting",
       §            "Done opening"
       §        );
       §        Ok(sf)
       §    }
       §
       §    pub fn create(file: &str) -> Result<Self, SkillFail> {
       §        info!(
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
       §            Err(e) => Err(SkillFail::internal(InternalFail::FailedToCreateFile {
       §                file: file.to_owned(),
       §                why: e.description().to_owned(),
       §            })),
       §        }?;
       §        let string_block = Rc::new(RefCell::new(StringBlock::new()));
       §        let mut type_pool = TypeBlock::new();
       §        let mut file_builder = SkillFileBuilder::new(string_block.clone());
       §        let mut data_chunk_reader = Vec::new();
       §
       §        file_builder.allocate(&mut type_pool)?;
       §        file_builder.initialize(
       §            &type_pool,
       §            &data_chunk_reader,
       §            &string_block.borrow(),
       §        )?;
       §        let mut sf = SkillFile {
       §            file: Rc::new(RefCell::new(f)),
       §            type_pool,
       §            strings: string_block,${
      (for (base ← IR) yield {
        e"""
           §${field(base)}: file_builder.${field(base)}.unwrap(),""".stripMargin('§')
      }).mkString
    }
       §        };
       §        sf.complete();
       §        info!(
       §            target: "SkillWriting",
       §            "Done creating"
       §        );
       §        Ok(sf)
       §    }
       §
       §    pub fn write(&mut self) -> Result<(), SkillFail> {
       §        info!(
       §            target: "SkillWriting",
       §            "Start writing"
       §        );
       §        // invariant -> size queries are constant time
       §        self.type_pool.set_invariant(true);
       §
       §        // Load lazy fields
       §        // TODO Load lazy fields
       §        ${
      "" // TODO Load lazy fields
    }
       §
       §        // check
       §        self.check()?;
       §
       §        // reorder
       §        let local_bpos = self.type_pool.compress()?;
       §
       §        let mut writer = FileWriter::new(self.file.clone());
       §        self.strings.borrow().write_block(&mut writer)?;
       §        self.type_pool.write_block(&mut writer, &local_bpos)?;
       §
       §        self.type_pool.set_invariant(false);
       §        info!(
       §            target:"SkillWriting",
       §            "Done writing"
       §        );
       §        Ok(())
       §    }
       §
       §    pub fn close(mut self) -> Result<(), SkillFail> {${
      "" // TODO check if more has to be done?
    }
       §        info!(
       §            target: "SkillWriting",
       §            "Start closing"
       §        );
       §        self.write()?;
       §        info!(
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
    e"""pub struct SkillFileBuilder {
       §    ${
      (for (base ← IR) yield {
        e"""pub ${field(base)}: Option<Rc<RefCell<${storagePool(base)}>>>,
           §""".stripMargin('§')
      }).mkString.trim
    }
       §    undefined_pools: Vec<Rc<RefCell<UndefinedPool>>>,
       §    string_block: Rc<RefCell<StringBlock>>,
       §}""".stripMargin('§')
  }

  private final def genSkillFileBuilderImpl(): String = {
    e"""impl SkillFileBuilder {
       §    pub fn new(string_block: Rc<RefCell<StringBlock>>) -> SkillFileBuilder {
       §        SkillFileBuilder {
       §            ${
      (for (base ← IR) yield {
        e"""${field(base)}: None,
           §""".stripMargin('§')
      }).mkString.trim
    }
       §            undefined_pools: Vec::new(),
       §            string_block,
       §        }
       §    }
       §
       §    fn allocate(&mut self, type_pool: &mut TypeBlock) -> Result<(), SkillFail> {
       §        self.string_block.borrow_mut().finalize()?;
       §        ${
      (for (base ← IR) yield {
        e"""if self.${field(base)}.is_none() {
           §    let name = self.string_block.borrow().lit().${field(base)};
           §    let name = self.string_block.borrow_mut().add(name);
           §    let pool = Rc::new(RefCell::new(
           §        ${storagePool(base)}::new(
           §            self.string_block.clone(),
           §            name,
           §            type_pool.len() + 32,
           §        )
           §    ));${
          if (base.getSuperType != null) {
            e"""
               §self.${field(base.getSuperType)}.as_ref().unwrap().borrow_mut().add_sub(pool.clone());
               §pool.borrow_mut().set_super(self.${field(base.getSuperType)}.as_ref().unwrap().clone());"""
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
        e"""self.${field(base)}.as_ref().unwrap().borrow_mut().allocate();${
          if (base.getBaseType.equals(base)) {
            e"""
               §self.${field(base)}.as_ref().unwrap().borrow_mut().set_next_pool(None);""".stripMargin('§')
          } else {
            ""
          }
        }
           §""".stripMargin('§')
      }).mkString.trim
    }
       §        for pool in self.undefined_pools.iter() {
       §            pool.borrow_mut().allocate();
       §            if pool.borrow().is_base() {
       §                pool.borrow_mut().set_next_pool(None);
       §            }
       §        }
       §        Ok(())
       §    }
       §
       §    fn initialize(
       §        &self,
       §        type_pool: &TypeBlock,
       §        file_reader: &Vec<FileReader>,
       §        string_block: &StringBlock,
       §    ) -> Result<(), SkillFail> {
       §        type_pool.initialize(string_block, file_reader)?;
       §        Ok(())
       §    }
       §}""".stripMargin('§')
  }

  private final def genSkillFileBuilderImplPoolMaker(): String = {
    e"""impl PoolMaker for SkillFileBuilder {
       §    fn make_pool(
       §        &mut self,
       §        type_name: &Rc<SkillString>,
       §        type_id: usize,
       §        super_pool: Option<Rc<RefCell<InstancePool>>>,
       §    ) -> Result<Rc<RefCell<InstancePool>>, SkillFail> {
       §        ${
      (for (base ← IR) yield {
        e"""if type_name.as_str() == self.string_block.borrow().lit().${field(base)}  {
           §    if self.${field(base)}.is_none() {
           §        self.${field(base)} = Some(Rc::new(RefCell::new(
           §            ${storagePool(base)}::new(
           §                self.string_block.clone(),
           §                type_name.clone(),
           §                type_id,
           §            )
           §        )));
           §
           §        if let Some(super_pool) = super_pool {
           §            let super_name = {
           §                let tmp = super_pool.borrow();
           §                tmp.name().as_str().to_owned()
           §            };
           §            ${
          if (base.getSuperType == null) {
            e"""return Err(SkillFail::internal(InternalFail::UnexpectedSuperType {
               §    base: self.string_block.borrow().lit().${field(base)},
               §    super_name
               §}));
               §""".stripMargin('§').trim
          } else {
            e"""if super_name.as_str() != self.string_block.borrow().lit().${field(base.getSuperType)} {
               §    return Err(SkillFail::internal(InternalFail::WrongSuperType {
               §        base: self.string_block.borrow().lit().${field(base)},
               §        expected: self.string_block.borrow().lit().${field(base.getSuperType)},
               §        found: super_name,
               §    }));
               §} else {
               §    super_pool.borrow_mut().add_sub(self.${field(base)}.as_ref().unwrap().clone());
               §    self.${field(base)}.as_ref().unwrap().borrow_mut().set_super(super_pool);
               §}
               §""".stripMargin('§').trim
          }
        }
           §        } ${
          if (base.getSuperType != null) {
            e"""else {
               §    return Err(SkillFail::internal(InternalFail::MissingSuperType {
               §        base: self.string_block.borrow().lit().${field(base)},
               §        expected: self.string_block.borrow().lit().${field(base.getSuperType)},
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
       §            for pool in self.undefined_pools.iter() {
       §                if pool.borrow().get_type_id() == type_id {
       §                    return Ok(pool.clone());
       §                }
       §            }
       §            let pool = Rc::new(RefCell::new(UndefinedPool::new(
       §                self.string_block.clone(),
       §                type_name.clone(),
       §                type_id,
       §            )));
       §            if let Some(super_pool) = super_pool {
       §                super_pool.borrow_mut().add_sub(pool.clone());
       §                pool.borrow_mut().set_super(super_pool);
       §            }
       §            self.undefined_pools.push(pool.clone());
       §            Ok(pool)
       §        }
       §    }
       §
       §    fn get_pool(&self, type_name_index: usize) -> Option<Rc<RefCell<InstancePool>>> {
       §        ${
      (for (base ← IR) yield {
        e"""if self.${field(base)}.is_some()
           §    && type_name_index == self.${field(base)}.as_ref().unwrap().borrow().name().get_skill_id()
           §{
           §    return Some(self.${field(base)}.as_ref().unwrap().clone());
           §} else """.stripMargin('§')
      }).mkString
    }{
       §            for pool in self.undefined_pools.iter() {
       §                if pool.borrow().name().get_skill_id() == type_name_index {
       §                    return Some(pool.clone());
       §                }
       §            }
       §        }
       §        None
       §    }
       §}""".stripMargin('§')
  }
}
