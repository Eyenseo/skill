/*  ___ _  ___ _ _                                                            *\
** / __| |/ (_) | |       The SKilL Generator                                 **
** \__ \ ' <| | | |__     (c) 2013-18 University of Stuttgart                 **
** |___/_|\_\_|_|____|    see LICENSE                                         **
\*                                                                            */
package de.ust.skill.generator.rust

import de.ust.skill.generator.common.IndenterLaw._
import de.ust.skill.ir.{Type, UserType}

trait PtrMaker extends GeneralOutputMaker {

  abstract override def make: Unit = {
    super.make

    val out = files.open("src/ptr.rs")

    out.write(
               e"""${genUsage()}
                  §
                  §${genCasts()}
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
                e"""use common::Ptr;
                   §use common::internal::SkillObject;
                   §use common::internal::foreign;
                   §
                   §use std::any::TypeId;
                   §
                   §""".stripMargin('§')
              )

    for (base ← IR) {
      val mod = snakeCase(storagePool(base))

      ret.append(
                  e"""use $mod::${name(base)};
                     §use $mod::${foreignName(base)};
                     §use $mod::${traitName(base)};
                     §""".stripMargin('§')
                )
    }
    ret.mkString.trim
  }

  //----------------------------------------
  // Casts
  //----------------------------------------
  def genCasts(): String = {
    e"""ptr_cast_able!(SkillObject =
       §    ${
      (for (t ← IR) yield {
        e"""${genNucastTraitInner(t, foreign = false)}
           §${genNucastTraitInner(t, foreign = true)}
           §""".stripMargin('§')
      }).mkString.trim
    }
       §    foreign::ObjectProper: {
       §        SkillObject,
       §        foreign::Object,
       §    },
       §);
       §
       §ptr_cast_able!(foreign::ObjectProper = {
       §    SkillObject,
       §    foreign::Object,
       §});
       §ptr_cast_able!(foreign::Object =
       §    ${
      (for (t ← IR) yield {
        e"""${genNucastTraitInner(t, foreign = true)}
           §""".stripMargin('§')
      }).mkString.trim
    }
       §    foreign::ObjectProper: {
       §        SkillObject,
       §        foreign::Object,
       §    },
       §);
       §
       §${
      (for (base ← IR) yield {
        e"""${genNucast(base)}
           §
           §""".stripMargin('§')
      }).mkString.trim
    }
       §""".stripMargin('§').trim
  }

  def genNucast(base: UserType): String = {
    e"""${genNucastStruct(base)}
       §${genNucastTrait(base)}
       §""".stripMargin('§')
  }.trim

  def genNucastStruct(base: UserType): String = {
    e"""ptr_cast_able!(${name(base)} = {
       §    SkillObject,
       §    ${
      (for (sub ← getAllSuperTypes(base)) yield {
        e"""${traitName(sub)},
           §""".stripMargin('§')
      }).mkString.trim
    }
       §});
       §ptr_cast_able!(${foreignName(base)} = {
       §    SkillObject,
       §    ${
      (for (sub ← getAllSuperTypes(base)) yield {
        e"""${traitName(sub)},
           §""".stripMargin('§')
      }).mkString.trim
    }
       §    foreign::Object,
       §});
       §""".stripMargin('§')
  }.trim

  def genNucastTrait(base: UserType): String = {
    e"""ptr_cast_able!(${traitName(base)} =
       §    ${
      (for (t ← (getAllSuperTypes(base) ::: getAllSubTypes(base)).distinct) yield {
        e"""${genNucastTraitInner(t, foreign = false)}
           §${genNucastTraitInner(t, foreign = true)}
           §""".stripMargin('§')
      }).mkString.trim
    }
       §);
       §""".stripMargin('§')
  }.trim

  def genNucastTraitInner(base: Type, foreign: Boolean): String = {
    val t = {
      val t = IR.filter(u ⇒ u == base)
      if (t.size != 1) {
        throw new GeneratorException(s"Didn't find unique user type: ${base.getName} aka: ${name(base)}")
      }
      t.head
    }

    e"""${if (foreign) foreignName(base) else name(base)}: {
       §    SkillObject,
       §    ${
      (for (base ← getAllSuperTypes(t)) yield {
        e"""${traitName(base)},
           §""".stripMargin('§')
      }).mkString.trim
    }${
      if (foreign) {
        "\nforeign::Object,"
      } else {
        ""
      }
    }
       §},
       §""".stripMargin('§')
  }.trim
}