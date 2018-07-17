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

    // TODO add stuff
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
                  |""".stripMargin)

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
                   |use common::io::{FieldReader, BlockIndex, FieldType, FileReader};
                   |use common::PoolMaker;
                   |use common::Ptr;
                   |use common::SkillError;
                   |use common::SkillFile as SkillFileTrait;
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
       |    mmap: Rc<Mmap>,
       |    string_block: Rc<RefCell<StringBlock>>,
       |    type_pool: TypeBlock,
       |    pub file_builder: SkillFileBuilder,
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
       |            .open(&file)
       |            .or(Err(SkillError::NotAFile))?;
       |
       |        let string_block = Rc::new(RefCell::new(StringBlock::new())); // FIXME rename / move w/e.
       |        let mut type_pool = TypeBlock::new(); // FIXME rename / move w/e.
       |        let mut file_builder = SkillFileBuilder::new(string_block.clone());
       |        let mut type_pools = Vec::new();
       |
       |        let mmap = unsafe { Mmap::map(&f) }.or(Err(SkillError::NotAFile))?;
       |        let rmmap = Rc::new(mmap);
       |        let mut data_chunk_reader = Vec::new();
       |        let mut block_index = BlockIndex::from(0);
       |        {
       |            let mut reader = FileReader::from(rmmap.clone());
       |
       |            loop {
       |                info!(
       |                      target: "SkillParsing", "Block: {:?} Reader:{:?}",
       |                      block_index,
       |                      reader
       |                );
       |
       |                // TODO implement blocks
       |                string_block.borrow_mut().read_string_block(&mut reader)?;
       |                type_pool.read_type_block(
       |                    block_index,
       |                    &mut reader,
       |                    &mut file_builder,
       |                    &string_block,
       |                    &mut type_pools,
       |                    &mut data_chunk_reader,
       |                )?;
       |                if reader.is_empty() {
       |                    break;
       |                }
       |                block_index += 1;
       |            }
       |        }
       |        file_builder.allocate(&mut type_pools);
       |        file_builder.initialize(
       |            &type_pools,
       |            &data_chunk_reader,
       |            &string_block.borrow(),
       |
       |        )?;
       |        Ok(SkillFile {
       |            mmap: rmmap,
       |            string_block,
       |            type_pool,
       |            file_builder,
       |        })
       |    }
       |    fn create(_file: &str) -> Result<Self, SkillError> {
       |        Err(SkillError::UnexpectedEndOfInput)
       |    }
       |    fn write(&self) -> Result<(), SkillError> {
       |        Ok(())
       |    }
       |    fn close(&self) -> Result<(), SkillError> {
       |        Ok(())
       |    }
       |    fn check(&self) -> Result<(), SkillError> {
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
    e"""#[derive(Default)]
       |pub struct SkillFileBuilder {
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
       |    fn allocate(&mut self, type_pools: &mut Vec<Rc<RefCell<InstancePool>>>) {
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
           |    type_pools.push(pool);
           |}
           |""".stripMargin
      }).mkString.trim
    }
       |        ${
      (for (base ← IR) yield {
        e"""self.${field(base)}.as_ref().unwrap().borrow_mut().allocate(type_pools);
           |""".stripMargin
      }).mkString.trim
    }
       |    }
       |
       |    fn initialize(
       |        &mut self,
       |        type_pools: &Vec<Rc<RefCell<InstancePool>>>,
       |        file_reader: &Vec<FileReader>,
       |        string_block: &StringBlock,
       |    ) -> Result<(), SkillError> {
       |        for ref pool in type_pools {
       |            pool.borrow().initialize(file_reader, string_block, type_pools);
       |        }
       |        Ok(())
       |    }
       |}""".stripMargin
  }

  private final def genSkillFileBuilderImplPoolMaker(): String = {
    e"""impl PoolMaker for SkillFileBuilder {
       |    fn make_pool(
       |        &mut self,
       |        type_name_index: usize,
       |        type_name: &str,
       |        type_id: usize,
       |        super_pool: Option<Rc<RefCell<InstancePool>>>,
       |    ) -> Rc<RefCell<InstancePool>> {
       |        match type_name {
       |            ${
      (for (base ← IR) yield {
        e""""${field(base)}" => {
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
            e"""if super_name.as_str() != "${field(base.getSuperType)}" {
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
           |},
           |""".stripMargin
      }).mkString.trim
    }
       |            _ => {
       |                for pool in self.undefined_pools.iter() {
       |                    if pool.borrow().get_type_id() == type_id {
       |                        return pool.clone();
       |                    }
       |                }
       |                let pool = Rc::new(RefCell::new(UndefinedPool::new(
       |                    self.string_block.clone(),
       |                )));
       |                {
       |                    let mut pool = pool.borrow_mut();
       |                    pool.set_type_id(type_id);
       |                    pool.set_type_name_index(type_name_index);
       |                }
       |                if let Some(super_pool) = super_pool {
       |                    super_pool.borrow_mut().add_sub(pool.clone());
       |                    pool.borrow_mut().set_super(super_pool);
       |                }
       |                self.undefined_pools.push(pool.clone());
       |                pool
       |            }
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
