/*  ___ _  ___ _ _                                                            *\
** / __| |/ (_) | |       The SKilL Generator                                 **
** \__ \ ' <| | | |__     (c) 2013-18 University of Stuttgart                 **
** |___/_|\_\_|_|____|    see LICENSE                                         **
\*                                                                            */
package de.ust.skill.generator.rust

import de.ust.skill.generator.common.Indenter._

trait SkillFileMaker extends GeneralOutputMaker {
  abstract override def make {
    super.make

    makeSource()
  }

  private def makeSource() {
    val out = files.open("src/skill_file.rs")

    out.write(
               e"""${genUsage()}
                  |
                  |${genSkillFile()}
                  |
                  |${genSkillFileBuilder()}
                  |""".stripMargin
             )

    out.close()
  }

  //----------------------------------------
  // Usage
  //----------------------------------------
  private final def genUsage(): String = {
    val ret = new StringBuilder()

    ret.append(
                e"""use common::internal::InstancePool;
                   |use common::internal::ObjectReader;
                   |use common::internal::UndefinedPool;
                   |use common::io::{FieldReader, BlockIndex, FieldType, FileWriter, FileReader};
                   |use common::PoolMaker;
                   |use common::Ptr;
                   |use common::SkillError;
                   |use common::SkillFile as SkillFileTrait;
                   |use common::SkillString;
                   |use common::StringBlock;
                   |use common::TypeBlock;
                   |
                   |use memmap::Mmap;
                   |
                   |use std::cell::RefCell;
                   |use std::rc::Rc;
                   |
                   |""".stripMargin
              )

    for (base ← IR) {
      val mod = snakeCase(storagePool(base))

      ret.append(
                  e"""use $mod::${storagePool(base)};
                     |use $mod::${traitName(base)};
                     |use $mod::${name(base)};
                     |""".stripMargin
                )
    }
    ret.mkString.trim
  }

  //----------------------------------------
  // SkillFile
  //----------------------------------------
  private final def genSkillFile(): String = {
    e"""//----------------------------------------
       |// SkillFile
       |//----------------------------------------
       |${genSkillFileStruct()}
       |
       |${genSkillFileImplSkillFileTrait()}
       |""".stripMargin.trim
  }

  private final def genSkillFileStruct(): String = {
    e"""pub struct SkillFile {
       |    file: Rc<RefCell<std::fs::File>>,
       |    type_pool: TypeBlock,
       |    pub strings: Rc<RefCell<StringBlock>>,${
      (for (base ← IR) yield {
        e"""
           |pub ${field(base)}: Rc<RefCell<${storagePool(base)}>>,""".stripMargin
      }).mkString
    }
       |}""".stripMargin
  }

  private final def genSkillFileImplSkillFileTrait(): String = {
    e"""impl SkillFileTrait for SkillFile {
       |    type T = Self;
       |
       |    fn open(file: &str) -> Result<Self, SkillError> {
       |        let f = ::std::fs::OpenOptions::new()
       |            .read(true)
       |            .write(true)
       |            .open(&file)${
      "" // FIXME handle errors better
    }
       |            .or(Err(SkillError::NotAFile))?;
       |
       |        debug!(
       |              target: "SkillParsing", "File exists!",
       |        );
       |        let string_block = Rc::new(RefCell::new(StringBlock::new()));
       |        let mut type_pool = TypeBlock::new();
       |        let mut file_builder = SkillFileBuilder::new(string_block.clone());
       |        let mut data_chunk_reader = Vec::new();
       |${
      "" // FIXME handle errors better
    }
       |        if f.metadata().or(Err(SkillError::NotAFile))?.len() != 0 {
       |            let mmap = unsafe { Mmap::map(&f) }.or(Err(SkillError::NotAFile))?;
       |            debug!(
       |                  target: "SkillParsing", "File exists!",
       |            );
       |            let rmmap = Rc::new(mmap);
       |            let mut block_index = BlockIndex::from(0);
       |            {
       |                let mut reader = FileReader::from(rmmap.clone());
       |
       |                loop {
       |                    info!(
       |                          target: "SkillParsing", "Block: {:?} Reader:{:?}",
       |                          block_index,
       |                          reader
       |                    );
       |
       |                    // TODO implement blocks
       |                    string_block.borrow_mut().read_string_block(&mut reader)?;
       |                    type_pool.read_type_block(
       |                        block_index,
       |                        &mut reader,
       |                        &mut file_builder,
       |                        &string_block,
       |                        &mut data_chunk_reader,
       |                    )?;
       |                    if reader.is_empty() {
       |                        break;
       |                    }
       |                    block_index += 1;
       |                }
       |            }
       |        }
       |        file_builder.allocate(&mut type_pool);
       |        file_builder.initialize(
       |            &type_pool,
       |            &data_chunk_reader,
       |            &string_block.borrow(),
       |        )?;
       |        Ok(SkillFile {
       |            file: Rc::new(RefCell::new(f)),
       |            type_pool,
       |            strings: string_block,${
      (for (base ← IR) yield {
        e"""
           |${field(base)}: file_builder.${field(base)}.unwrap(),""".stripMargin
      }).mkString
    }
       |        })
       |    }
       |
       |    fn create(file: &str) -> Result<Self, SkillError> {
       |        let f = ::std::fs::OpenOptions::new()
       |            .write(true)
       |            .create(true)
       |            .open(&file)
       |            .or(Err(SkillError::NotAFile))?;
       |        let string_block = Rc::new(RefCell::new(StringBlock::new()));
       |        let mut type_pool = TypeBlock::new();
       |        let mut file_builder = SkillFileBuilder::new(string_block.clone());
       |        let mut data_chunk_reader = Vec::new();
       |
       |        file_builder.allocate(&mut type_pool);
       |        file_builder.initialize(
       |            &type_pool,
       |            &data_chunk_reader,
       |            &string_block.borrow(),
       |        )?;
       |        Ok(SkillFile {
       |            file: Rc::new(RefCell::new(f)),
       |            type_pool,
       |            strings: string_block,${
      (for (base ← IR) yield {
        e"""
           |${field(base)}: file_builder.${field(base)}.unwrap(),""".stripMargin
      }).mkString
    }
       |        })
       |    }
       |
       |    fn write(&self) -> Result<(), SkillError> {
       |        let mut writer = FileWriter::new(self.file.clone());
       |        self.strings.borrow().write_block(&mut writer)?;
       |        self.type_pool.write_block(&mut writer)?;
       |
       |        Ok(())
       |    }
       |
       |    fn close(self) -> Result<(), SkillError> {${
      "" // TODO check if more has to be done?
    }
       |        self.write()
       |    }
       |
       |    fn check(&self) -> Result<(), SkillError> {${
      "" // TODO implement check
    }
       |        Ok(())
       |    }
       |}""".stripMargin
  }

  //----------------------------------------
  // SkillFileBuilder
  //----------------------------------------
  private final def genSkillFileBuilder(): String = {
    e"""//----------------------------------------
       |// SkillFileBuilder
       |//----------------------------------------
       |${genSkillFileBuilderStruct()}
       |
       |${genSkillFileBuilderImpl()}
       |
       |${genSkillFileBuilderImplPoolMaker()}
       |""".stripMargin.trim
  }

  private final def genSkillFileBuilderStruct(): String = {
    e"""pub struct SkillFileBuilder {
       |    ${
      (for (base ← IR) yield {
        e"""pub ${field(base)}: Option<Rc<RefCell<${storagePool(base)}>>>,
           |""".stripMargin
      }).mkString.trim
    }
       |    undefined_pools: Vec<Rc<RefCell<UndefinedPool>>>,
       |    string_block: Rc<RefCell<StringBlock>>,
       |}""".stripMargin
  }

  private final def genSkillFileBuilderImpl(): String = {
    e"""impl SkillFileBuilder {
       |    pub fn new(string_block: Rc<RefCell<StringBlock>>) -> SkillFileBuilder {
       |        SkillFileBuilder {
       |            ${
      (for (base ← IR) yield {
        e"""${field(base)}: None,
           |""".stripMargin
      }).mkString.trim
    }
       |            undefined_pools: Vec::new(),
       |            string_block,
       |        }
       |    }
       |
       |    fn allocate(&mut self, type_pool: &mut TypeBlock) {
       |        self.string_block.borrow_mut().finalize();
       |        ${
      (for (base ← IR) yield {
        e"""if self.${field(base)}.is_none() {
           |    let pool = Rc::new(RefCell::new(
           |        ${storagePool(base)}::new(self.string_block.clone())
           |    ));${
          if (base.getSuperType != null) {
            e"""
               |self.${field(base.getSuperType)}.as_ref().unwrap().borrow_mut().add_sub(pool.clone());
               |pool.borrow_mut().set_super(self.${field(base.getSuperType)}.as_ref().unwrap().clone());""".stripMargin
          } else {
            ""
          }
        }
           |    self.${field(base)} = Some(pool.clone());
           |    type_pool.add(pool);
           |}
           |""".stripMargin
      }).mkString.trim
    }
       |        ${
      (for (base ← IR) yield {
        e"""self.${field(base)}.as_ref().unwrap().borrow_mut().allocate();
           |""".stripMargin
      }).mkString.trim
    }
       |        for pool in self.undefined_pools.iter() {
       |            pool.borrow_mut().allocate();
       |        }
       |    }
       |
       |    fn initialize(
       |        &self,
       |        type_pool: &TypeBlock,
       |        file_reader: &Vec<FileReader>,
       |        string_block: &StringBlock,
       |    ) -> Result<(), SkillError> {
       |        type_pool.initialize(string_block, file_reader)?;
       |        Ok(())
       |    }
       |}""".stripMargin
  }

  private final def genSkillFileBuilderImplPoolMaker(): String = {
    e"""impl PoolMaker for SkillFileBuilder {
       |    fn make_pool(
       |        &mut self,
       |        type_name_index: usize,
       |        type_name: &Rc<SkillString>,
       |        type_id: usize,
       |        super_pool: Option<Rc<RefCell<InstancePool>>>,
       |    ) -> Rc<RefCell<InstancePool>> {
       |        ${
      (for (base ← IR) yield {
        e"""if type_name.as_str() == self.string_block.borrow().lit().${field(base)}  {
           |    if self.${field(base)}.is_none() {
           |        self.${field(base)} = Some(Rc::new(RefCell::new(
           |            ${storagePool(base)}::new(self.string_block.clone())
           |        )));
           |
           |        if let Some(super_pool) = super_pool {
           |            let idx = super_pool.borrow().get_type_name_index();
           |            let super_name = self.string_block.borrow().get(idx);
           |            ${
          if (base.getSuperType == null) {
            e"""panic!(
               |    "The type '${base.getName.camel()}' aka '${name(base)}' does not expect a super type. Found:{}",
               |    super_name
               |);
               |""".stripMargin.trim
          } else {
            e"""if super_name.as_str() != self.string_block.borrow().lit().${field(base.getSuperType)} {
               |    panic!(
               |        "Wrong super type for '${base.getName.camel()}' aka '${name(base)}' expect:${
              field(base.getSuperType)
            } found:{}",
               |        super_name
               |    );
               |} else {
               |    super_pool.borrow_mut().add_sub(self.${field(base)}.as_ref().unwrap().clone());
               |    self.${field(base)}.as_ref().unwrap().borrow_mut().set_super(super_pool);
               |}
               |""".stripMargin.trim
          }
        }
           |        } ${
          if (base.getSuperType != null) {
            e"""else {
               |            panic!("The type '${base.getName.camel()}' aka '${name(base)}' expects a supertype.");
               |        }
               |""".stripMargin.trim
          } else {
            ""
          }
        }
           |    } else {
           |        panic!("Double creation of pool");
           |    }
           |    let mut ${field(base)} = self.${field(base)}.as_ref().unwrap().borrow_mut();
           |    ${field(base)}.set_type_id(type_id);
           |    ${field(base)}.set_type_name_index(type_name_index);
           |    self.${field(base)}.as_ref().unwrap().clone()
           |} else """.stripMargin
      }).mkString
    }{
       |            for pool in self.undefined_pools.iter() {
       |                if pool.borrow().get_type_id() == type_id {
       |                    return pool.clone();
       |                }
       |            }
       |            let pool = Rc::new(RefCell::new(UndefinedPool::new(
       |                self.string_block.clone(),
       |            )));
       |            {
       |                let mut pool = pool.borrow_mut();
       |                pool.set_type_id(type_id);
       |                pool.set_type_name_index(type_name_index);
       |            }
       |            if let Some(super_pool) = super_pool {
       |                super_pool.borrow_mut().add_sub(pool.clone());
       |                pool.borrow_mut().set_super(super_pool);
       |            }
       |            self.undefined_pools.push(pool.clone());
       |            pool
       |        }
       |    }
       |
       |    fn get_pool(&self, type_name_index: usize) -> Option<Rc<RefCell<InstancePool>>> {
       |        ${
      (for (base ← IR) yield {
        e"""if self.${field(base)}.is_some()
           |    && type_name_index == self.${field(base)}.as_ref().unwrap().borrow().get_type_name_index()
           |{
           |    return Some(self.${field(base)}.as_ref().unwrap().clone());
           |} else """.stripMargin
      }).mkString
    }{
       |            for pool in self.undefined_pools.iter() {
       |                if pool.borrow().get_type_name_index() == type_name_index {
       |                    return Some(pool.clone());
       |                }
       |            }
       |        }
       |        None
       |    }
       |}""".stripMargin
  }
}
