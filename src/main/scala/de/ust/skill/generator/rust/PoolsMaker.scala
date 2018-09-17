/*  ___ _  ___ _ _                                                            *\
** / __| |/ (_) | |       The SKilL Generator                                 **
** \__ \ ' <| | | |__     (c) 2013-18 University of Stuttgart                 **
** |___/_|\_\_|_|____|    see LICENSE                                         **
\*                                                                            */
package de.ust.skill.generator.rust

import de.ust.skill.generator.common.IndenterLaw._
import de.ust.skill.ir._

import scala.collection.JavaConverters._
import scala.collection.mutable.ArrayBuffer

trait PoolsMaker extends GeneralOutputMaker {
  abstract override def make {
    super.make

    // one file per base type
    for (base ← IR) {
      val out = files.open(s"src/${field(base)}.rs")

      out.write(
                 e"""${genUsage(base)}
                    §
                    §${genType(base)}
                    §
                    §${genPoolPartsMaker(base)}
                    §
                    §${genPool(base)}
                    §
                    §${genFieldDeclaration(base)}
                    §""".stripMargin('§'))
      out.close()
    }

    for (base ← IRInterfaces) {
      val out = files.open(s"src/${field(base)}.rs")

      out.write(
                 e"""${genUsage(base)}
                    §
                    §${genInterface(base)}
                    §""".stripMargin('§'))
      out.close()
    }
  }

  //----------------------------------------
  // Usage
  //----------------------------------------
  private final def genUsage(base: Type): String = {
    // TODO Sort
    e"""use common::error::*;
       §use common::internal::*;
       §use common::internal::io::magic::bytes_v64;
       §use common::internal::io::*;
       §use common::iterator::*;
       §use common::skill_object;
       §use common::*;
       §
       §use skill_file::SkillFileBuilder;${
      getUsageUser(base) +
      getCustomUser() +
      getUsageStd()
    }
       §""".stripMargin('§')
  }.trim

  private final def getUsageUser(base: Type): String = {
    val ret = (IR
               .filterNot(t ⇒ t.equals(base))
               .toArray
               .map(t ⇒ s"use ${field(t)}::*;\n")
               .sorted
               .mkString + IRInterfaces
                           .filterNot(t ⇒ t.equals(base))
                           .toArray
                           .map(t ⇒ s"use ${snakeCase(interface(t))}::*;\n")
                           .sorted
                           .mkString).trim
    if (ret != "") {
      "\n\n" + ret
    } else {
      ret
    }
  }

  def getCustomUser(): String = {
    val ret = (IR ::: IRInterfaces).flatMap(gatherCustomUses)
                                   .sorted
                                   .distinct
                                   .map(t ⇒ s"use $t;\n")
                                   .mkString.trim
    if (ret != "") {
      "\n\n" + ret
    } else {
      ret
    }
  }

  private final def getUsageStd(): String = {
    e"""
       §
       §use std::cell::{Cell, RefCell};
       §use std::collections::{HashMap, HashSet, LinkedList};
       §use std::ops::DerefMut;
       §use std::rc::{Rc, Weak};
       §""".stripMargin('§')
  }

  //----------------------------------------
  // Interface
  //----------------------------------------
  private final def genInterface(base: InterfaceType): String = {
    e"""//----------------------------------------
       §// ${base.getName} aka ${name(base)}
       §//----------------------------------------
       §
       §${genTypeTrait(base)}
       §
       §""".stripMargin('§')
  }.trim

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
    // NOTE be sure to change foreign::ForeignObject too!
    e"""#[derive(Default, Debug)]
       §#[repr(C)]
       §pub struct ${name(base)} {
       §    skill_id: Cell<usize>,
       §    skill_type_id: usize,
       §    foreign_data: Vec<foreign::FieldData>,
       §    ${
      (for (t ← getAllSupers(base)) yield {
        (for (f ← t.getFields.asScala.filterNot(_.isConstant)) yield {
          // the field before interface projection
          val orig = this.types.removeTypedefs().removeEnums()
                         .get(base.getSkillName).asInstanceOf[UserType].getAllFields.asScala
                                                                       .find(_.getName == f.getName).get

          e"""${internalName(f)}: ${mapType(orig.getType)},
             §""".stripMargin('§')
        }).mkString +
        (for (c ← t.getCustomizations.asScala.filter(c ⇒ c.language.equals("rust")).flatMap({
          case null ⇒ ArrayBuffer[LanguageCustomization]()
          case c    ⇒ ArrayBuffer[LanguageCustomization](c)
        })) yield {
          e"""${c.getName}: ${c.`type`},
             §""".stripMargin('§')
        }).mkString.trim
      }).mkString.trim
    }
       §}""".stripMargin('§')
  }

  private final def genTypeTrait[Base <: Declaration with WithInheritance](base: Base): String = {
    var com = comment(base)
    if (!com.isEmpty) {
      com += "\n"
    }

    e"""${com}pub trait ${traitName(base)}: ${
      var supers = (base.getSuperInterfaces.asScala.toList ::: List(base.getSuperType)).filterNot(_ == null)

      base match {
        case _: InterfaceType ⇒
          base.getSuperType match {
            case _: UserType ⇒
            case _           ⇒ supers = List()
          }
        case _                ⇒
      }

      if (supers.nonEmpty) {
        supers.map(s ⇒ traitName(s)).mkString(" + ")
      } else {
        "SkillObject"
      }
    } {${
      if (base.getFields.asScala.nonEmpty || gatherCustoms(base).nonEmpty) {
        e"""
           §    ${
          ((for (f ← base.getFields.asScala) yield {
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
              if (!f.isConstant && (f.getType.isInstanceOf[ReferenceType] || f.getType.isInstanceOf[ContainerType])) {
                e"""
                   §${com}fn get_${name(f)}_mut(&mut self) -> &mut ${mapType(f.getType)};""".stripMargin('§')
              } else {
                ""
              }
            }${
              if (!f.isConstant) {
                e"""
                   §${com}fn set_${name(f)}(&mut self, ${name(f)}: ${mapType(f.getType)});""".stripMargin('§')
              } else {
                ""
              }
            }
               §
               §""".stripMargin('§')
          }).mkString +
           (for (c ← gatherCustoms(base)) yield {
             var com = c.getComment
                       // NOTE 4 spaces indent
                       .format("", "/// ", lineLength - 4, "")
                       .trim
             if (com != "") {
               com += "\n"
             }
             e"""${com}fn get_${c.getName}(&self) -> &${c.`type`};
                §${com}fn get_${c.getName}_mut(&mut self) -> &mut ${c.`type`};
                §${com}fn set_${c.getName}(&mut self, ${c.getName}: ${c.`type`});
                §
                §""".stripMargin('§')
           }).mkString).trim
        }
           §""".stripMargin('§')
      } else {
        ""
      }
    }}""".stripMargin('§')
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
      if (field.isConstant) {
        e"""unsafe {
           §    std::mem::transmute::<${
          mapType(field.getType).replace('i', 'u')
        }, ${
          mapType(field.getType)
        }>(${field.constantValue()})
           §}""".stripMargin('§')
      } else if (field.getType.isInstanceOf[ReferenceType] || field.getType.isInstanceOf[ContainerType]) {
        e"&self.${internalName(field)}"
      } else {
        e"self.${internalName(field)}"
      }
    }
       §}${
      if (!field.isConstant &&
          (field.getType.isInstanceOf[ReferenceType] || field.getType.isInstanceOf[ContainerType])) {
        e"""
           §fn get_${name(field)}_mut(&mut self) -> &mut ${mapType(field.getType)} {
           §    &mut self.${internalName(field)}
           §}""".stripMargin('§')
      } else {
        ""
      }
    }${
      if (!field.isConstant) {
        e"""
           §fn set_${name(field)}(&mut self, value: ${mapType(field.getType)}) {
           §    self.${internalName(field)} = value;
           §}""".stripMargin('§')
      } else {
        ""
      }
    }""".stripMargin('§')
  }

  private final def genTypeImpl(base: UserType): String = {
    // gen New
    e"""impl ${name(base)} {
       §    pub fn new(skill_id: usize, skill_type_id: usize) -> ${name(base)} {
       §        ${name(base)} {
       §            skill_id: Cell::new(skill_id),
       §            skill_type_id,
       §            foreign_data: Vec::default(),
       §            ${
      ((for (f ← base.getAllFields.asScala.filterNot(_.isConstant)) yield {
        e"""${internalName(f)}: ${defaultValue(f)},
           §""".stripMargin('§')
      }).mkString +
       (for (c ← gatherCustoms(base)) yield {
         e"""${c.getName}: ${c.getOptions.get("init").asScala.head},
            §""".stripMargin('§')
       }).mkString).trim
    }
       §        }
       §    }
       §}
       §
       §${
      if (base.getFields.asScala.nonEmpty || gatherCustoms(base).nonEmpty) {
        e"""impl ${traitName(base)} for ${name(base)} {
           §    ${ // Impl base
          ((for (f ← base.getFields.asScala) yield {
            e"""${genGetSetImpl(f)}
               §
               §""".stripMargin('§')
          }).mkString +
           (for (c ← gatherCustoms(base)) yield {
             e"""fn get_${c.getName}(&self) -> &${c.`type`} {
                §    &self.${c.getName}
                §}
                §fn get_${c.getName}_mut(&mut self) -> &mut ${c.`type`} {
                §   &mut self.${c.getName}
                §}
                §fn set_${c.getName}(&mut self, ${c.getName}: ${c.`type`}) {
                §   self.${c.getName} = ${c.getName}
                §}
                §
                §""".stripMargin('§')
           }).mkString).trim
        }
           §}""".stripMargin('§')
      } else {
        e"impl ${traitName(base)} for ${name(base)} {}"
      }
    }${
      // Impl super
      val ret = new StringBuilder()
      var toImplement: List[Declaration] = List(base.getSuperType) :::
                                           allSuperInterfaces(base)
      var implemented: List[Declaration] = List()

      while (toImplement.nonEmpty && base.getSuperType != null) {
        if (implemented.contains(toImplement.head)) {
          val (_, tmp) = toImplement.splitAt(1)
          toImplement = tmp
        } else {
          toImplement.head match {
            case declaration: Declaration with WithFields ⇒
              ret.append(
                          e"""${genTypeImplSuper(base, declaration)}
                             §
                             §""".stripMargin('§')
                        )
            case _                                        ⇒
          }
          toImplement.head match {
            case t: WithInheritance ⇒
              toImplement = toImplement ::: allSuperInterfaces(t)
              if (t.getSuperType != null) {
                t.getSuperType match {
                  case t: Declaration ⇒ toImplement = toImplement ::: List(t)
                  case _              ⇒
                }
              }
            case _                  ⇒
          }

          implemented = implemented ::: List(toImplement.head)
          val (_, tmp) = toImplement.splitAt(1)
          toImplement = tmp
        }
      }

      if (ret.nonEmpty) {
        s"\n\n${ret.mkString.trim}"
      }
      else {
        ""
      }
    }
       §impl foreign::ForeignObject for ${name(base)} {
       §    fn foreign_fields(&self) -> &Vec<foreign::FieldData> {
       §        &self.foreign_data
       §    }
       §    fn foreign_fields_mut(&mut self) -> &mut Vec<foreign::FieldData> {
       §        &mut self.foreign_data
       §    }
       §}
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
       §}
       §
       §impl Deletable for ${name(base)} {
       §    fn mark_for_deletion(&mut self) {
       §        self.skill_id.set(skill_object::DELETE);
       §    }
       §    fn to_delete(&self) -> bool {
       §        self.skill_id.get() == skill_object::DELETE
       §    }
       §}
       §""".stripMargin('§').trim
  }

  private final def genTypeImplSuper[Base <: Declaration with WithFields](base: Base,
                                                                          parent: Base): String = {
    if (parent.getFields.asScala.nonEmpty || gatherCustoms(parent).nonEmpty) {
      e"""impl ${traitName(parent)} for ${name(base)} {
         §    ${
        ((for (f ← parent.getFields.asScala) yield {
          e"""${genGetSetImpl(f)}
             §
             §""".stripMargin('§')
        }).mkString.trim +
         (for (c ← gatherCustoms(parent)) yield {
           e"""fn get_${c.getName}(&self) -> &${c.`type`} {
              §    &self.${c.getName}
              §}
              §fn get_${c.getName}_mut(&mut self) -> &mut ${c.`type`} {
              §   &mut self.${c.getName}
              §}
              §fn set_${c.getName}(&mut self, ${c.getName}: ${c.`type`}) {
              §   self.${c.getName} = ${c.getName}
              §}
              §
              §""".stripMargin('§')
         }).mkString).trim
      }
         §}""".stripMargin('§')
    } else {
      e"impl ${traitName(parent)} for ${name(base)} {}"
    }
  }

  //----------------------------------------
  // TypePoolPartsMaker
  //----------------------------------------
  private final def genPoolPartsMaker(base: UserType): String = {
    e"""//----------------------------------------
       §// ${base.getName}PoolPartsMaker aka ${poolPartsMaker(base)}
       §//----------------------------------------
       §#[derive(Default)]
       §pub(crate) struct ${poolPartsMaker(base)} {
       §    type_name: Rc<SkillString>,
       §}
       §
       §impl ${poolPartsMaker(base)} {
       §    fn new(type_name: Rc<SkillString>) -> ${poolPartsMaker(base)} {
       §        ${poolPartsMaker(base)} {
       §            type_name,
       §        }
       §    }
       §}
       §
       §impl PoolPartsMaker for ${poolPartsMaker(base)} {
       §    fn make_field(
       §        &self,
       §        index: usize,
       §        field_name: Rc<SkillString>,
       §        mut field_type: FieldType,
       §        string_pool: &StringBlock,
       §    ) -> Result<(bool, Box<RefCell<FieldDeclaration>>), SkillFail> {
       §        ${
      ((for (f ← base.getFields.asScala.toList :::
                 allSuperInterfaces(base).flatMap(t ⇒ t.getFields.asScala)) yield {
        if (f.isAuto) {
          e"""if string_pool.lit().${field(f)} == field_name.as_str() {
             §    Err(SkillFail::internal(InternalFail::AutoNotAuto {
             §        field: field_name.string().clone(),
             §    }))
             §} else """.stripMargin('§')
        } else {
          val userType = collectUserTypes(f.getType)
          e"""if string_pool.lit().${field(f)} == field_name.as_str() {
             §    ${
            if (userType.isEmpty) {
              e"""match field_type {
                 §    ${genPoolPartsMakerImplPoolPartsMakerMakeFieldValidate(f.getType)},
                 §    _ => Err(SkillFail::internal(InternalFail::BadFieldType {
                 §        expected: "${mapTypeToUser(f.getType)}",
                 §        found: format!("{}", field_type)
                 §    })),
                 §}?;
                 §Ok((
                 §    false,
                 §    Box::new(RefCell::new(${fieldDeclaration(base, f)}::new(
                 §        field_name,
                 §        index,
                 §        field_type,
                 §    ))),
                 §))
                 §""".stripMargin('§')
            } else {
              e"""let mut object_readers: Vec<Weak<RefCell<PoolProxy>>> = Vec::new();
                 §object_readers.reserve(${userType.size});
                 §match field_type {
                 §    ${genPoolPartsMakerImplPoolPartsMakerMakeFieldValidate(f.getType)},
                 §    _ => Err(SkillFail::internal(InternalFail::BadFieldType {
                 §        expected: "${mapTypeToUser(f.getType)}",
                 §        found: format!("{}", field_type)
                 §    })),
                 §}?;
                 §Ok((
                 §    false,
                 §    Box::new(RefCell::new(${fieldDeclaration(base, f)}::new(
                 §        field_name,
                 §        index,
                 §        field_type,
                 §        object_readers,
                 §    ))),
                 §))
                 §""".stripMargin('§')
            }
          }
             §} else """.stripMargin('§')
        }
      }).mkString +
       (for (c ← gatherCustoms(base)) yield {
         e"""if string_pool.lit().${c.getName} == field_name.as_str() {
            §    Err(SkillFail::internal(InternalFail::AutoNotAuto {
            §        field: field_name.string().clone(),
            §    }))
            §} else """.stripMargin('§')
       }).mkString).trim
    } {
       §            match &field_type {
       §                FieldType::BuildIn(field_type) =>{
       §                    match field_type {
       §                        BuildInType::ConstTi8 => Err(SkillFail::internal(
       §                            InternalFail::UnknownConstantField{
       §                                field: field_name.string().clone(),
       §                                type_name: self.type_name.string().clone(),
       §                            }
       §                        ))?,
       §                        BuildInType::ConstTi16 => Err(SkillFail::internal(
       §                            InternalFail::UnknownConstantField{
       §                                field: field_name.string().clone(),
       §                                type_name: self.type_name.string().clone(),
       §                            }
       §                        ))?,
       §                        BuildInType::ConstTi32 => Err(SkillFail::internal(
       §                            InternalFail::UnknownConstantField{
       §                                field: field_name.string().clone(),
       §                                type_name: self.type_name.string().clone(),
       §                            }
       §                        ))?,
       §                        BuildInType::ConstTi64 => Err(SkillFail::internal(
       §                            InternalFail::UnknownConstantField{
       §                                field: field_name.string().clone(),
       §                                type_name: self.type_name.string().clone(),
       §                            }
       §                        ))?,
       §                        BuildInType::ConstTv64 => Err(SkillFail::internal(
       §                            InternalFail::UnknownConstantField{
       §                                field: field_name.string().clone(),
       §                                type_name: self.type_name.string().clone(),
       §                            }
       §                        ))?,
       §                        _ => {}
       §                    }
       §                },
       §                _ => {}
       §            }
       §            Ok((
       §                true,
       §                Box::new(RefCell::new(foreign::FieldDeclaration::new(
       §                    field_name,
       §                    index,
       §                    field_type
       §                ))),
       §            ))
       §        }
       §    }
       §
       §    fn make_instance(&self, skill_id: usize, skill_type_id: usize) -> Ptr<SkillObject> {
       §        trace!(
       §            target:"SkillParsing",
       §            "Create new ${name(base)}",
       §        );
       §        Ptr::new(${name(base)}::new(skill_id, skill_type_id))
       §    }
       §}
       §""".stripMargin('§')
  }.trim

  // TODO do something about these stupid names
  private final def genPoolPartsMakerImplPoolPartsMakerMakeFieldValidate(tt: Type): String = {
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
           §        ${genPoolPartsMakerImplPoolPartsMakerMakeFieldValidate(t.getBaseType)},
           §        _ => Err(SkillFail::internal(InternalFail::BadFieldType {
           §            expected: "${mapTypeToUser(t.getBaseType)}",
           §            found: format!("{}", **box_v)
           §        })),
           §    }
           §}""".stripMargin('§')
      case t: SingleBaseTypeContainer ⇒
        e"""${mapTypeToMagicMatch(t)} => {
           §    match **box_v {
           §        ${genPoolPartsMakerImplPoolPartsMakerMakeFieldValidate(t.getBaseType)},
           §        _ => Err(SkillFail::internal(InternalFail::BadFieldType {
           §            expected: "${mapTypeToUser(t.getBaseType)}",
           §            found: format!("{}", **box_v)
           §        })),
           §    }
           §}""".stripMargin('§')
      case t: MapType                 ⇒
        e"""${mapTypeToMagicMatch(t)} => {
           §    ${genPoolPartsMakerImplPoolPartsMakerMakeFieldValidateMap(t.getBaseTypes.asScala.toList)}
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
      case t: InterfaceType           ⇒
        t.getBaseType match {
          case _: UserType ⇒ e"""FieldType::User(ref object_reader) => {
                                §    object_readers.push(object_reader.clone());
                                §    Ok(())
                                §}
                                §""".stripMargin('§')
          case t           ⇒ e"""${mapTypeToMagicMatch(t)} => Ok(())
                                §""".stripMargin('§')
        }
      case _                          ⇒
        throw new GeneratorException("Unexpected field type")
    }
  }.trim

  private final def genPoolPartsMakerImplPoolPartsMakerMakeFieldValidateMap(tts: List[Type]): String = {
    val (key, remainder) = tts.splitAt(1)

    e"""match **key_box_v {
       §    ${genPoolPartsMakerImplPoolPartsMakerMakeFieldValidate(key.head)},
       §    _ => Err(SkillFail::internal(InternalFail::BadFieldType {
       §        expected: "${mapTypeToUser(key.head)}",
       §        found: format!("{}", **key_box_v)
       §    })),
       §}?;
       §match **box_v {
       §    ${
      if (remainder.size >= 2) {
        e"""FieldType::BuildIn(BuildInType::Tmap(ref key_box_v, ref box_v)) => {
           §    ${genPoolPartsMakerImplPoolPartsMakerMakeFieldValidateMap(remainder)}
           §}""".stripMargin('§')
      } else {
        genPoolPartsMakerImplPoolPartsMakerMakeFieldValidate(remainder.head)
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
       §${genPoolImplPoolProxy(base)}
       §""".stripMargin('§')
  }.trim

  private final def genPoolStruct(base: UserType): String = {
    e"""pub struct ${storagePool(base)} {
       §    pool: Pool,
       §    string_pool: Rc<RefCell<StringBlock>>,
       §}""".stripMargin('§')
  }

  private final def genPoolImpl(base: UserType): String = {
    e"""impl ${storagePool(base)} {
       §    pub(crate) fn new(
       §        string_pool: Rc<RefCell<StringBlock>>,
       §        name: Rc<SkillString>,
       §        type_id: usize,
       §    ) -> ${storagePool(base)} {
       §        ${storagePool(base)} {
       §            pool: Pool::new(
       §                name.clone(),
       §                type_id,
       §                Box::new(${poolPartsMaker(base)}::new(name))
       §            ),
       §            string_pool
       §        }
       §    }
       §
       §    pub fn get(&self, index: usize) -> Result<Ptr<${name(base)}>, SkillFail> {
       §        match self.pool.read_object(index) {
       §            Ok(obj) => {
       §                if obj.borrow().get_skill_id() == skill_object::DELETE {
       §                    return Err(SkillFail::user(UserFail::AccessDeleted));
       §                }
       §                match obj.cast::<${name(base)}>() {
       §                    Some(obj) => Ok(obj.clone()),
       §                    None => Err(SkillFail::user(UserFail::BadCastID { id:index })),
       §                }
       §            },
       §            Err(e) => Err(e),
       §        }
       §    }
       §
       §    pub fn add(&mut self) -> Ptr<${name(base)}> {
       §        let ret = Ptr::new(${name(base)}::new(0, self.pool.get_type_id()));
       §        self.pool.add(ret.clone());
       §        ret
       §    }
       §}""".stripMargin('§')
  }

  private final def genPoolImplPoolProxy(base: UserType): String = {
    e"""impl PoolProxy for ${storagePool(base)} {
       §    fn pool(&self) -> &Pool {
       §        &self.pool
       §    }
       §    fn pool_mut(&mut self) -> &mut Pool {
       §        &mut self.pool
       §    }
       §
       §    fn complete(&mut self, file: &SkillFileBuilder) {
       §        ${
      val fields = base.getFields.asScala.filterNot(_.isAuto).toList :::
                   allSuperInterfaces(base).flatMap(t ⇒ t.getFields.asScala).filterNot(_.isAuto)
      if (fields.nonEmpty) {
        e"""let mut set = HashSet::with_capacity(${fields.size});
           §let mut string_pool = self.string_pool.borrow_mut();
           §{
           §    let lit = string_pool.lit();
           §    ${
          (for (field ← fields) yield {
            e"""set.insert(lit.${name(field)});
               §""".stripMargin('§')
          }).mkString.trim()
        }
           §}
           §
           §for f in self.pool.fields().iter() {
           §    set.remove(f.borrow().name().as_str());
           §}
           §
           §${
          (for (ft ← fields) yield {
            e"""if set.contains(string_pool.lit().${name(ft)}) {
               §    let index = self.pool.fields().len() + 1;
               §    let name = string_pool.lit().${name(ft)};${
              "" // FIXME accessing the fields of lit will create _copies_! else this would be illegal
            }
               §    self.pool.fields_mut().push(Box::new(RefCell::new(
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
                    e"""{
                       §    // This is madness ...
                       §    let tmp = Rc::downgrade(file.${pool(ut)}.as_ref().unwrap());
                       §    tmp
                       §},
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
       §}
       §""".stripMargin('§')
  }.trim

  //----------------------------------------
  // FieldDeclaration
  //----------------------------------------
  private final def genFieldDeclaration(base: UserType): String = {
    val ret = new StringBuilder()

    for (field ← base.getFields.asScala.filterNot(_.isAuto).toList :::
                 allSuperInterfaces(base).flatMap(t ⇒ t.getFields.asScala).filterNot(_.isAuto)) {
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
       §    field_id: usize, // Index into the pool fields vector
       §    field_type: FieldType,
       §    chunks: Vec<FieldChunk>,${
      val userType = collectUserTypes(field.getType)
      if (userType.isEmpty) {
        ""
      } else {
        e"""
           §object_reader: Vec<Weak<RefCell<PoolProxy>>>,""".stripMargin('§')
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
         §        field_id: usize,
         §        field_type: FieldType,
         §    ) -> ${fieldDeclaration(base, field)} {
         §        ${fieldDeclaration(base, field)} {
         §            name,
         §            field_id,
         §            field_type,
         §            chunks: Vec::new(),
         §        }
         §    }
         §}""".stripMargin('§')
    } else {
      e"""impl ${fieldDeclaration(base, field)} {
         §    fn new(
         §        name: Rc<SkillString>,
         §        field_id: usize,
         §        field_type: FieldType,
         §        object_reader: Vec<Weak<RefCell<PoolProxy>>>
         §    ) -> ${fieldDeclaration(base, field)} {
         §        ${fieldDeclaration(base, field)} {
         §            name,
         §            field_id,
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
       §        block_reader: &Vec<FileReader>,
       §        string_pool: &StringBlock,
       §        blocks: &Vec<Block>,
       §        type_pools: &Vec<Rc<RefCell<PoolProxy>>>,
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
       §                    let mut reader = block_reader[block.block.block].rel_view(chunk.begin, chunk.end);
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
       §                                trace!(
       §                                    target:"SkillParsing",
       §                                    "Block:{:?} ObjectProper:{}",
       §                                    block,
       §                                    o + block.bpo,
       §                                );
       §                                o += 1;
       §                                ${
      if (f.isConstant) {
        e"""let val = ${
          genFieldDeclarationImplFieldDeclarationRead(f.getType, Stream.iterate(0)(_ + 1).iterator)
        };
           §if unsafe {
           §    std::mem::transmute::<${
          mapType(f.getType).replace('i', 'u')
        }, ${
          mapType(f.getType)
        }>(${f.constantValue()})
           §} != val {
           §    return Err(SkillFail::internal(InternalFail::BadConstantValue{
           §        field: self.name.string().clone(),
           §        expected: format!("{}", unsafe {
           §            std::mem::transmute::<${
          mapType(f.getType).replace('i', 'u')
        }, ${
          mapType(f.getType)
        }>(${f.constantValue()})
           §        }),
           §        found: format!("{}", val),
           §    }));
           §}""".stripMargin('§')
      } else {
        e"""match obj.cast::<${name(base)}>() {
           §    Some(obj) =>
           §        obj.borrow_mut().set_${name(f)}(${
          genFieldDeclarationImplFieldDeclarationRead(f.getType, Stream.iterate(0)(_ + 1).iterator)
        }),
           §    None => return Err(SkillFail::internal(InternalFail::BadCast)),
           §}""".stripMargin('§')
      }
    }
       §                            }
       §                        }
       §                    }
       §                },
       §                FieldChunk::Continuation(chunk) => {
       §                    let block = &blocks[block_index.block];
       §                    let mut reader = block_reader[block.block.block].rel_view(chunk.begin, chunk.end);
       §                    block_index += 1;
       §
       §                    if chunk.count > 0 {
       §                        let mut o = 0;
       §
       §                        for obj in instances.iter()
       §                            .skip(chunk.bpo)
       §                            .take(chunk.count)
       §                        {
       §                            trace!(
       §                                target:"SkillParsing",
       §                                "Block:{:?} ObjectProper:{}",
       §                                block,
       §                                o + chunk.bpo,
       §                            );
       §                            o += 1;
       §                            ${
      if (f.isConstant) {
        e"""let val = ${
          genFieldDeclarationImplFieldDeclarationRead(f.getType, Stream.iterate(0)(_ + 1).iterator)
        };
           §if unsafe {
           §    std::mem::transmute::<${
          mapType(f.getType).replace('i', 'u')
        }, ${
          mapType(f.getType)
        }>(${f.constantValue()})
           §} != val {
           §    return Err(SkillFail::internal(InternalFail::BadConstantValue{
           §        field: self.name.string().clone(),
           §        expected: format!("{}", unsafe {
           §            std::mem::transmute::<${
          mapType(f.getType).replace('i', 'u')
        }, ${
          mapType(f.getType)
        }>(${f.constantValue()})
           §        }),
           §        found: format!("{}", val),
           §    }));
           §}""".stripMargin('§')
      } else {
        e"""match obj.cast::<${name(base)}>() {
           §    Some(obj) =>
           §        obj.borrow_mut().set_${name(f)}(${
          genFieldDeclarationImplFieldDeclarationRead(f.getType, Stream.iterate(0)(_ + 1).iterator)
        }),
           §    None => return Err(SkillFail::internal(InternalFail::BadCast)),
           §}""".stripMargin('§')
      }
    }
       §                        }
       §                    }
       §                }
       §            }
       §        }
       §        Ok(())
       §    }
       §
       §    fn deserialize(
       §        &mut self,
       §        block_reader: &Vec<FileReader>,
       §        string_pool: &StringBlock,
       §        blocks: &Vec<Block>,
       §        type_pools: &Vec<Rc<RefCell<PoolProxy>>>,
       §        instances: &[Ptr<SkillObject>],
       §    ) -> Result<(), SkillFail> {
       §        Ok(())
       §    }
       §
       §    fn add_chunk(&mut self, chunk: FieldChunk) {
       §        self.chunks.push(chunk);
       §    }
       §    fn name(&self) -> &Rc<SkillString> {
       §        &self.name
       §    }
       §    fn field_id(&self) -> usize {
       §        self.field_id
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
       §    fn offset(&self, iter: dynamic_data::Iter) -> Result<usize, SkillFail> {
       §        ${genFieldDeclarationImplFieldDeclarationOffset(base, f)}
       §    }
       §    fn write_meta(&mut self, writer: &mut FileWriter, iter: dynamic_data::Iter, offset: usize) -> Result<usize, SkillFail> {
       §        debug!(
       §            target:"SkillWriting",
       §            "~~~~Write Field Meta Data for Field:{}",
       §            self.name.as_ref(),
       §        );
       §        writer.write_v64(self.field_id as i64)?;
       §        writer.write_v64(self.name.get_id() as i64)?;
       §        writer.write_field_type(&self.field_type)?;
       §        writer.write_i8(0)?; // TODO write restrictions
       §        let end_offset = offset + self.offset(iter.clone())?;
       §        debug!(
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
       §       debug!(
       §            target:"SkillWriting",
       §            "~~~~Write Field Data for Field:{}",
       §            self.name.as_ref(),
       §        );
       §        let mut writer = match self.chunks.first().unwrap() {
       §            FieldChunk::Declaration(ref chunk) => writer.rel_view(chunk.begin, chunk.end)?,
       §            FieldChunk::Continuation(_) => Err(SkillFail::internal(InternalFail::OnlyOneChunk))?,
       §        };
       §        for i in iter {
       §            let tmp = i.cast::<${name(base)}>().unwrap();
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
            e"""Ok(iter.count())
               §""".stripMargin('§')
          case "i16"         ⇒
            e"""Ok(2 * iter.count())
               §""".stripMargin('§')
          case "i32" | "f32" ⇒
            e"""Ok(4 * iter.count())
               §""".stripMargin('§')
          case "f64" | "i64" ⇒
            e"""Ok(8 * iter.count())
               §""".stripMargin('§')
          case "v64"         ⇒
            e"""let mut offset = 0;
               §for i in iter {
               §    let tmp = i.cast::<${name(base)}>().unwrap();
               §    let tmp = tmp.borrow(); // borrowing madness
               §    offset += bytes_v64(tmp.get_${field(f)}() as i64);
               §}
               §Ok(offset)
               §""".stripMargin('§')
          case "string"      ⇒
            e"""let mut offset = 0;
               §for i in iter {
               §    let tmp = i.cast::<${name(base)}>().unwrap();
               §    let tmp = tmp.borrow(); // borrowing madness
               §    if let Some(tmp) = tmp.get_${field(f)}() {
               §        offset += bytes_v64(tmp.get_id() as i64);
               §    } else {
               §        offset += 1;
               §    }
               §}
               §Ok(offset)
               §""".stripMargin('§')
          case "annotation"  ⇒
            e"""let mut offset = 0;
               §for i in iter {
               §    let tmp = i.cast::<${name(base)}>().unwrap();
               §    let tmp = tmp.borrow(); // borrowing madness
               §    offset += match tmp.get_${field(f)}() {
               §        Some(ref val) => match val.upgrade() {
               §            Some(val) => if val.borrow().to_delete() {
               §                2
               §            } else {
               §                bytes_v64((val.borrow().skill_type_id() - 31) as i64)
               §                    + bytes_v64(val.borrow().get_skill_id() as i64)
               §            },
               §            None => 2,
               §        },
               §        None => 2,
               §    };
               §}
               §Ok(offset)
               §""".stripMargin('§')
          case _             ⇒
            throw new GeneratorException(s"Unhandled type $ft")
        }
      case ft: ConstantLengthArrayType ⇒
        e"""let mut offset = 0;
           §for i in iter {
           §    let tmp = i.cast::<${name(base)}>().unwrap();
           §    let tmp = tmp.borrow(); // borrowing madness
           §    for val in tmp.get_${field(f)}().iter() {
           §        offset += ${genFieldDeclarationImplFieldDeclarationOffsetInner(ft.getBaseType)};
           §    }
           §}
           §Ok(offset)
           §""".stripMargin('§')
      case ft: SingleBaseTypeContainer ⇒
        e"""let mut offset = 0;
           §for i in iter {
           §    let tmp = i.cast::<${name(base)}>().unwrap();
           §    let tmp = tmp.borrow(); // borrowing madness
           §    offset += bytes_v64(tmp.get_${field(f)}().len() as i64);
           §    for val in tmp.get_${field(f)}().iter() {
           §        offset += ${genFieldDeclarationImplFieldDeclarationOffsetInner(ft.getBaseType)};
           §    }
           §}
           §Ok(offset)
           §""".stripMargin('§')
      case ft: MapType                 ⇒
        e"""let mut offset = 0;
           §for i in iter {
           §    let tmp = i.cast::<${name(base)}>().unwrap();
           §    let tmp = tmp.borrow(); // borrowing madness
           §    let val = tmp.get_${field(f)}();
           §    ${genFieldDeclarationImplFieldDeclarationOffsetMap(ft.getBaseTypes.asScala.toList)}
           §}
           §Ok(offset)
           §""".stripMargin('§')
      case _: UserType                 ⇒
        e"""let mut offset = 0;
           §for i in iter {
           §    let tmp = i.cast::<${name(base)}>().unwrap();
           §    let tmp = tmp.borrow(); // borrowing madness
           §    offset += match tmp.get_${field(f)}() {
           §        Some(ref val) => match val.upgrade() {
           §            Some(val) => if val.borrow().to_delete() {
           §                1
           §            } else {
           §                bytes_v64(val.borrow().get_skill_id() as i64)
           §            },
           §            None => 1,
           §        },
           §        None => 1,
           §    };
           §}
           §Ok(offset)
           §""".stripMargin('§')
      case t: InterfaceType            ⇒
        t.getBaseType match {
          case _: UserType ⇒ e"""let mut offset = 0;
                                §for i in iter {
                                §    let tmp = i.cast::<${traitName(base)}>().unwrap();
                                §    let tmp = tmp.borrow(); // borrowing madness
                                §    offset += match tmp.get_${field(f)}() {
                                §        Some(ref val) => match val.upgrade() {
                                §            Some(val) => if val.borrow().to_delete() {
                                §                1
                                §            } else {
                                §                bytes_v64(val.borrow().get_skill_id() as i64)
                                §            },
                                §            None => 1,
                                §        },
                                §        None => 1,
                                §    };
                                §}
                                §Ok(offset)
                                §""".stripMargin('§')
          case _           ⇒ e"""let mut offset = 0;
                                §for i in iter {
                                §    let tmp = i.cast::<${traitName(base)}>().unwrap();
                                §    let tmp = tmp.borrow(); // borrowing madness
                                §    offset += match tmp.get_${field(f)}() {
                                §        Some(ref val) =>  match val.upgrade() {
                                §            Some(val) => if val.borrow().to_delete() {
                                §                2
                                §            } else {
                                §                bytes_v64((val.borrow().skill_type_id() - 31) as i64)
                                §                    + bytes_v64(val.borrow().get_skill_id() as i64)
                                §            },
                                §            None => 2,
                                §        },
                                §        None => 2,
                                §    };
                                §}
                                §Ok(offset)
                                §""".stripMargin('§')
        }

      case ft ⇒
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
            e"""if let Some(val) = val {
               §    bytes_v64(val.get_id() as i64)
               §} else {
               §    1
               §}
               §""".stripMargin('§')
          case "annotation"  ⇒
            e"""match val {
               §    Some(ref val) =>  match val.upgrade() {
               §        Some(val) => if val.borrow().to_delete() {
               §            2
               §        } else {
               §            bytes_v64((val.borrow().skill_type_id() - 31) as i64)
               §                + bytes_v64(val.borrow().get_skill_id() as i64)
               §        },
               §        None => 2,
               §    },
               §    None => 2,
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
           §    Some(ref val) =>  match val.upgrade() {
           §        Some(val) => if val.borrow().to_delete() {
           §            1
           §        } else {
           §            bytes_v64(val.borrow().get_skill_id() as i64)
           §        },
           §        None => 1,
           §    },
           §    None => 1,
           §}
           §""".stripMargin('§')
      case t: InterfaceType           ⇒
        t.getBaseType match {
          case _: UserType ⇒ e"""match val {
                                §    Some(ref val) =>  match val.upgrade() {
                                §        Some(val) => if val.borrow().to_delete() {
                                §            1
                                §        } else {
                                §            bytes_v64(val.borrow().get_skill_id() as i64)
                                §        },
                                §        None => 1,
                                §    },
                                §    None => 1,
                                §}
                                §""".stripMargin('§')
          case _           ⇒ e"""match val {
                                §    Some(ref val) =>  match val.upgrade() {
                                §        Some(val) => if val.borrow().to_delete() {
                                §            2
                                §        } else {
                                §            bytes_v64((val.borrow().skill_type_id() - 31) as i64)
                                §                + bytes_v64(val.borrow().get_skill_id() as i64)
                                §        },
                                §        None => 2,
                                §    },
                                §    None => 2,
                                §}
                                §""".stripMargin('§')
        }
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
            e"""if let Some(val) = val {
               §    writer.write_v64(val.get_id() as i64)?;
               §} else {
               §   writer.write_i8(0)?;
               §}
               §""".stripMargin('§')
          case "annotation" ⇒
            e"""match val {
               §    Some(ref val) => match val.upgrade() {
               §        Some(val) => if val.borrow().to_delete() {
               §            writer.write_i8(0)?;
               §            writer.write_i8(0)?;
               §        } else {
               §            writer.write_v64((val.borrow().skill_type_id() - 31) as i64)?;
               §            writer.write_v64(val.borrow().get_skill_id() as i64)?;
               §        },
               §        None =>{
               §            writer.write_i8(0)?;
               §            writer.write_i8(0)?;
               §        },
               §    },
               §    None => {
               §        writer.write_i8(0)?;
               §        writer.write_i8(0)?;
               §    },
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
           §    Some(ref val) => match val.upgrade() {
           §        Some(val) => if val.borrow().to_delete() {
           §            writer.write_i8(0)?;
           §        } else {
           §            writer.write_v64(val.borrow().get_skill_id() as i64)?;
           §        },
           §        None =>{
           §            writer.write_i8(0)?;
           §        },
           §    },
           §    None => writer.write_i8(0)?,
           §}
           §""".stripMargin('§')
      case t: InterfaceType            ⇒
        t.getBaseType match {
          case _: UserType ⇒ e"""match val {
                                §    Some(ref val) => match val.upgrade() {
                                §        Some(val) => if val.borrow().to_delete() {
                                §            writer.write_i8(0)?;
                                §        } else {
                                §            writer.write_v64(val.borrow().get_skill_id() as i64)?;
                                §        },
                                §        None =>{
                                §            writer.write_i8(0)?;
                                §        },
                                §    },
                                §    None => writer.write_i8(0)?,
                                §}
                                §""".stripMargin('§')
          case _           ⇒ e"""match val {
                                §    Some(ref val) => match val.upgrade() {
                                §        Some(val) => if val.borrow().to_delete() {
                                §            writer.write_i8(0)?;
                                §            writer.write_i8(0)?;
                                §        } else {
                                §            writer.write_v64((val.borrow().skill_type_id() - 31) as i64)?;
                                §            writer.write_v64(val.borrow().get_skill_id() as i64)?;
                                §        },
                                §        None =>{
                                §            writer.write_i8(0)?;
                                §            writer.write_i8(0)?;
                                §        },
                                §    },
                                §    None => {
                                §        writer.write_i8(0)?;
                                §        writer.write_i8(0)?;
                                §    },
                                §}
                                §""".stripMargin('§')
        }
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
            e"""if let Some(val) = val {
               §    writer.write_v64(val.get_id() as i64)?;
               §} else {
               §    writer.write_i8(0)?;
               §}
               §""".stripMargin('§')
          case "annotation" ⇒
            e"""match val {
               §    Some(ref val) => match val.upgrade() {
               §        Some(val) => if val.borrow().to_delete() {
               §            writer.write_i8(0)?;
               §            writer.write_i8(0)?;
               §        } else {
               §            writer.write_v64((val.borrow().skill_type_id() - 31) as i64)?;
               §            writer.write_v64(val.borrow().get_skill_id() as i64)?;
               §        },
               §        None =>{
               §            writer.write_i8(0)?;
               §            writer.write_i8(0)?;
               §        },
               §    },
               §    None => {
               §        writer.write_i8(0)?;
               §        writer.write_i8(0)?;
               §    },
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
           §    Some(ref val) => match val.upgrade() {
           §        Some(val) => if val.borrow().to_delete() {
           §            writer.write_i8(0)?;
           §        } else {
           §            writer.write_v64(val.borrow().get_skill_id() as i64)?;
           §        },
           §        None =>{
           §            writer.write_i8(0)?;
           §        },
           §    },
           §    None => writer.write_i8(0)?,
           §}
           §""".stripMargin('§')
      case t: InterfaceType            ⇒
        t.getBaseType match {
          case _: UserType ⇒ e"""match val {
                                §    Some(ref val) => match val.upgrade() {
                                §        Some(val) => if val.borrow().to_delete() {
                                §            writer.write_i8(0)?;
                                §        } else {
                                §            writer.write_v64(val.borrow().get_skill_id() as i64)?;
                                §        },
                                §        None =>{
                                §            writer.write_i8(0)?;
                                §        },
                                §    },
                                §    None => writer.write_i8(0)?,
                                §}
                                §""".stripMargin('§')
          case _           ⇒ e"""match val {
                                §    Some(ref val) => match val.upgrade() {
                                §        Some(val) => if val.borrow().to_delete() {
                                §            writer.write_i8(0)?;
                                §            writer.write_i8(0)?;
                                §        } else {
                                §            writer.write_v64((val.borrow().skill_type_id() - 31) as i64)?;
                                §            writer.write_v64(val.borrow().get_skill_id() as i64)?;
                                §        },
                                §        None =>{
                                §            writer.write_i8(0)?;
                                §            writer.write_i8(0)?;
                                §        },
                                §    },
                                §    None => {
                                §        writer.write_i8(0)?;
                                §        writer.write_i8(0)?;
                                §    },
                                §}
                                §""".stripMargin('§')
        }
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
        e"""string_pool.get(reader.read_v64()? as usize)?
           §""".stripMargin('§')
      case t: GroundType
        if t.getName.lower().equals("annotation") ⇒
        e"""{
           §    let pool = reader.read_v64()? as usize;
           §    let object = reader.read_v64()? as usize;
           §    if pool != 0 && object != 0 {
           §        Ok(Some(type_pools[pool - 1]
           §            .borrow()
           §            .pool()
           §            .read_object(object)?.downgrade()))
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
           §            .upgrade()
           §            .unwrap()
           §            .borrow()
           §            .pool()
           §            .read_object(object)?
           §            .cast::<${name(t)}>()
           §        {
           §            Ok(Some(object.downgrade()))
           §        } else {
           §            return Err(SkillFail::internal(InternalFail::BadCast))
           §        }
           §    } else {
           §        Ok(None)
           §    }
           §}?
           §""".stripMargin('§').trim
      case t: InterfaceType                       ⇒
        t.getBaseType match {
          case _: UserType ⇒ e"""{
                                §    let object = reader.read_v64()? as usize;
                                §    if object != 0 {
                                §        if let Some(object) = self.object_reader[${user.next()}]
                                §            .upgrade()
                                §            .unwrap()
                                §            .borrow()
                                §            .pool()
                                §            .read_object(object)?
                                §            .cast::<${traitName(t)}>()
                                §        {
                                §            Ok(Some(object.downgrade()))
                                §        } else {
                                §            return Err(SkillFail::internal(InternalFail::BadCast))
                                §        }
                                §    } else {
                                §        Ok(None)
                                §    }
                                §}?
                                §""".stripMargin('§').trim
          case _           ⇒ e"""{
                                §    let pool = reader.read_v64()? as usize;
                                §    let object = reader.read_v64()? as usize;
                                §    if pool != 0 && object != 0 {
                                §        Ok(Some(type_pools[pool - 1]
                                §            .borrow()
                                §            .pool()
                                §            .read_object(object)?.downgrade()))
                                §    } else {
                                §        Ok(None)
                                §    }
                                §}?
                                §""".stripMargin('§').trim
        }
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
    case _: ConstantLengthArrayType                                ⇒
      s"FieldType::BuildIn(${mapTypeToMagic(t)}(length, ref box_v))"
    case _: MapType                                                ⇒
      s"FieldType::BuildIn(${mapTypeToMagic(t)}(ref key_box_v, ref box_v))"
    case _@(_: VariableLengthArrayType | _: ListType | _: SetType) ⇒
      s"FieldType::BuildIn(${mapTypeToMagic(t)}(ref box_v))"
    case _: UserType                                               ⇒
      e"""FieldType::User(ref pool)"""
    case t: InterfaceType                                          ⇒
      t.getBaseType match {
        case _: UserType ⇒ e"""FieldType::User(ref pool)"""
        case t           ⇒ e"""FieldType::BuildIn(BuildInType::Tannotation)"""
      }
    case _                                                         ⇒
      e"""FieldType::BuildIn(${mapTypeToMagic(t)})"""
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
      e"""// This is madness ...
         §FieldType::User({
         §    let tmp = Rc::downgrade(file.${pool(t)}.as_ref().unwrap());
         §    tmp
         §})""".stripMargin('§')
    case t: InterfaceType           ⇒
      t.getBaseType match {
        case _: UserType ⇒
          e"""// This is madness ...
             §FieldType::User({
             §    let tmp = Rc::downgrade(file.${pool(t)}.as_ref().unwrap());
             §    tmp
             §})""".stripMargin('§')
        case _           ⇒ e"FieldType::BuildIn(BuildInType::Tannotation)"
      }
    case _                          ⇒
      e"FieldType::BuildIn(${mapTypeToMagic(t)})"
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

  private final def collectUserTypes(t: Type): List[Type] = t match {
    case t: MapType                 ⇒
      collectUserTypesMap(t, t.getBaseTypes.asScala.toList)
    case t: SingleBaseTypeContainer ⇒
      collectUserTypes(t.getBaseType)
    case t: UserType                ⇒
      List[Type](t)
    case t: InterfaceType           ⇒
      t.getBaseType match {
        case _: UserType ⇒ List[Type](t)
        case _           ⇒ List()
      }
    case _                          ⇒
      List()
  }

  private final def collectUserTypesMap(t: Type, tts: List[Type]): List[Type] = {
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
    case _: InterfaceType           ⇒ s"InterfaceType"

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

  private final def gatherCustomUses(base: WithFields): Seq[String] = {
    gatherCustoms(base).flatMap {
      case null ⇒ ArrayBuffer[String]()
      case c    ⇒ val inc = c.getOptions.get("use")
        if (null != inc) {
          inc.asScala
        } else {
          ArrayBuffer[String]()
        }
    }
  }
}
