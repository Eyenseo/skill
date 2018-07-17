/*  ___ _  ___ _ _                                                            *\
** / __| |/ (_) | |       The SKilL Generator                                 **
** \__ \ ' <| | | |__     (c) 2013-18 University of Stuttgart                 **
** |___/_|\_\_|_|____|    see LICENSE                                         **
\*                                                                            */
package de.ust.skill.generator.rust

import de.ust.skill.generator.common.Indenter._
import de.ust.skill.ir
import de.ust.skill.ir.{Type, UserType}

import scala.collection.JavaConverters._


trait PtrMaker extends GeneralOutputMaker {

  // TODO move somewhere else / Main?
  private final def getAllSuperTypes(t: UserType): List[Type] = {
    if (t.getSuperType != null) {
      getAllSuperTypes(t.getSuperType) ::: List[UserType](t)
    } else {
      List[UserType](t)
    }
  }.distinct


  abstract override def make: Unit = {
    super.make

    val out = files.open("src/ptr.rs")

    out.write(
               e"""${genUsage()}
                  |
                  |${genCasts()}
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
                e"""use common::Ptr;
                   |use common::internal::SkillObject;
                   |
                   |use std::any::TypeId;
                   |
                   |""".stripMargin
              )

    for (base ← IR) {
      val mod = snakeCase(storagePool(base))

      ret.append(
                  e"""use $mod::${name(base)};
                     |use $mod::${traitName(base)};
                     |""".stripMargin
                )
    }
    ret.mkString.trim
  }

  //----------------------------------------
  // Casts
  //----------------------------------------
  // TODO check that all casts are present and working
  // TODO benchmark against nucast
  def genCasts(): String = {
    e"""ptr_cast_able!(SkillObject =
       |    ${
      (for (t ← IR) yield {
        genNucastTraitInner(t)
      }).mkString.trim
    }
       |);
       |
       |${
      (for (base ← IR) yield {
        e"""${genNucast(base)}
           |
           |""".stripMargin
      }
      ).mkString.trim
    }
       |""".stripMargin.trim
  }

  def genNucast(base: UserType): String = {
    e"""${genNucastStruct(base)}
       |${genNucastTrait(base)}
       |""".stripMargin
  }

  def genNucastStruct(base: UserType): String = {
    e"""ptr_cast_able!(${name(base)} = {
       |    SkillObject,
       |    ${
      (for (sub ← getAllSuperTypes(base)) yield {
        e"""${traitName(sub)},
           |""".stripMargin
      }).mkString.trim
    }
       |});
       |""".stripMargin.trim
  }

  def genNucastTrait(base: UserType): String = {
    e"""ptr_cast_able!(${traitName(base)} =
       |    ${
      (for (t ← getAllSuperTypes(base) ::: base.getSubTypes.asScala.toList) yield {
        genNucastTraitInner(t)
      }).mkString.trim
    }
       |);
       |""".stripMargin.trim
  }

  def genNucastTraitInner(base: Type): String = {
    val t = {
      val t = IR.filter(u ⇒ u == base)
      if (t.size != 1) {
        throw new GeneratorException(s"Didn't find unique user type: ${base.getName} aka: ${name(base)}")
      }
      t.head
    }

    e"""${name(base)}: {
       |    SkillObject,
       |    ${
      (for (base ← getAllSuperTypes(t)) yield {
        e"""${traitName(base)},
           |""".stripMargin
      }).mkString.trim
    }
       |},
       |""".stripMargin
  }

  def genToCasts(base: ir.UserType, baseIsStruct: Boolean): String = {
    val ret = new StringBuilder()

    val low_base = base.getName.lower()
    val cap_base = base.getName.capital()

    // NOTE Basically useless ...
    ret.append(
                e"""pub fn to_${low_base}_t(from: &Ptr<$cap_base>) -> Ptr<${cap_base}T> {
                   |    from.clone()
                   |}
                   |""".stripMargin)

    for (to ← base.getAllSuperTypes.asScala) {
      val low_to = to.getName.lower()
      val cap_to = to.getName.capital()

      // NOTE Basically useless ...
      ret.append(
                  e"""pub fn to_${low_to}_t(from: &Ptr<$cap_base>) -> Ptr<${cap_to}T> {
                     |    from.clone()
                     |}
                     |""".stripMargin)


      ret.append(
                  e"""pub fn to_${low_to}_t(from: &Ptr<${cap_base}T>) -> Ptr<${cap_to}T> {
                     |    from.cast::<$cap_to>()
                     |}
                     |""".stripMargin)

    }

    ret.mkString.trim
  }

  def genAsCasts(base: ir.UserType, baseIsStruct: Boolean): String = {
    val ret = new StringBuilder()

    val low_base = base.getName.lower()
    val cap_base = base.getName.capital()

    ret.append(
                e"""pub fn as_$low_base(from: &Ptr<${cap_base}T>) -> Option<Ptr<$cap_base>> {
                   |    if from.type_id() == TypeId::of::<$cap_base>() {
                   |        Some(from.cast::<$cap_base>())
                   |    } else {
                   |        None
                   |    }
                   |}
                   |""".stripMargin)


    for (as ← base.getSubTypes.asScala) {
      val low_as = as.getName.lower()
      val cap_as = as.getName.capital()

      ret.append(
                  e"""pub fn as_${low_as}_t(from: &Ptr<$cap_base>) -> Option<Ptr<${cap_as}T>> {
                     |    ${
                    (for (t ← base.getSubTypes.asScala) yield {
                      e"""if from.type_id() == TypeId::of::<${t.getName.camel()}>() {
                         |    Some(self.cast::<${t.getName.capital()}>())
                         |} else """.stripMargin
                    }).mkString
                  }{
                     |        None
                     |    }
                     |}
                     |""".stripMargin)

      ret.append(
                  e"""pub fn as_${low_as}_t(from: &Ptr<${cap_base}T>) -> Option<Ptr<${cap_as}T>> {
                     |    ${
                    (for (t ← base.getSubTypes.asScala) yield {
                      e"""if from.type_id() == TypeId::of::<${t.getName.camel()}>() {
                         |    Some(self.cast::<${t.getName.capital()}>())
                         |} else """.stripMargin
                    }).mkString
                  }{
                     |        None
                     |    }
                     |}
                     |""".stripMargin)

      ret.append(
                  e"""pub fn as_$low_as(from: &Ptr<${cap_base}T>) -> Option<Ptr<$cap_as>> {
                     |    if from.type_id() == TypeId::of::<$cap_as>() {
                     |        Some(self.cast::<$cap_as>())
                     |    } else {
                     |        None
                     |    }
                     |}
                     |""".stripMargin)
    }

    ret.mkString.trim
  }
}
