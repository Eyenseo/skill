/*  ___ _  ___ _ _                                                            *\
** / __| |/ (_) | |       The SKilL Generator                                 **
** \__ \ ' <| | | |__     (c) 2013-18 University of Stuttgart                 **
** |___/_|\_\_|_|____|    see LICENSE                                         **
\*                                                                            */
package de.ust.skill.generator.rust

import de.ust.skill.generator.common.IndenterLaw._
import de.ust.skill.ir._

import scala.collection.mutable.ArrayBuffer

trait PtrMaker extends GeneralOutputMaker {

  abstract override def make: Unit = {
    super.make

    val out = files.open("src/ptr.rs")

    out.write(
               e"""${genUsage()}
                  §
                  §${genLookUpMap()}
                  §${genCastAbles()}
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
                e"""use common::*;
                   §use common::internal::foreign;
                   §use common::ptr::VTable;
                   §use common::ptr::TraitObject;
                   §use common::ptr::CastAble;
                   §
                   §use std::any::TypeId;
                   §use std::mem::transmute;
                   §use std::ptr::null;
                   §
                   §""".stripMargin('§')
              )

    for (base ← IR) {
      val mod = snakeCase(storagePool(base))

      ret.append(
                  e"""use $mod::${name(base)};
                     §use $mod::${traitName(base)};
                     §""".stripMargin('§')
                )
    }
    for (base ← IRInterfaces) {
      val mod = snakeCase(interface(base))

      ret.append(
                  e"""use $mod::${traitName(base)};
                     §""".stripMargin('§')
                )
    }
    ret.mkString.trim
  }

  //----------------------------------------
  // LookUpMap
  //----------------------------------------
  private final def makeInheritanceMap():
  // Scala Generic suck ...
  Map[Declaration with WithInheritance, List[Declaration with WithInheritance]] = {
    var map = ArrayBuffer[(Declaration with WithInheritance, List[Declaration with WithInheritance])]()

    for (t ← IR) {
      map.append(t → makeInheritanceMap(t))
    }
    map.toMap
  }

  private final def makeInheritanceMap(base: Declaration with WithInheritance):
  // Scala Generic suck ...
  List[Declaration with WithInheritance] = {
    var map = ArrayBuffer[Declaration with WithInheritance]()

    for (t ← getAllSuperTypes(base)) {
      map += t
    }
    for (t ← allSuperInterfaces(base)) {
      map += t
    }
    map += base

    map.toList.distinct
  }

  private final def makeLookupArray(base: Declaration, valids: List[Declaration]): String = {
    val arr = ArrayBuffer.fill(IR.size + IRInterfaces.size + 2)(
                                                                 e"""None,
                                                                    §""".stripMargin('§'))

    arr.update(IR.size,
                e"""Some(unsafe {transmute::<_, TraitObject>(
                   §    null::<foreign::Foreign>() as *const foreign::ForeignObject
                   §).vtable }),
                   §""".stripMargin('§'))
    arr.update(IR.size + IRInterfaces.size + 1,
                e"""Some(unsafe {transmute::<_, TraitObject>(
                   §    null::<foreign::Foreign>() as *const SkillObject
                   §).vtable }),
                   §""".stripMargin('§'))

    for (t ← valids) yield {
      if (t.isInstanceOf[InterfaceType]) {
        arr.update(genTypeId(t),
                    e"""Some(unsafe {transmute::<_, TraitObject>(
                       §    null::<${name(base)}>() as *const ${traitName(t)}
                       §).vtable }),
                       §""".stripMargin('§'))
      } else {
        arr.update(genTypeId(t),
                    e"""Some(unsafe { transmute::<_, TraitObject>(
                       §    null::<${name(t)}>() as *const ${traitName(t)}
                       §).vtable }),
                       §""".stripMargin('§'))
      }
    }
    arr.mkString.trim
  }

  private final def genLookUpMap(): String = {
    val maps = makeInheritanceMap()

    e"""lazy_static! {
       §    pub(crate) static ref VALID_CASTS: [[Option<VTable>; ${
      IR.size + IRInterfaces.size + 2
    }]; ${
      IR.size + 1
    }] = [
       §        ${
      (for (base ← IR) yield {
        val map = maps(base)
        e"""[
           §    // ${name(base)}
           §    ${makeLookupArray(base, map)}
           §],
           §""".stripMargin('§')
      }).mkString.trim
    }
       §        [
       §            // Foreign
       §            ${makeLookupArray(null, List())}
       §        ],
       §    ];
       §}
       §""".stripMargin('§')
  }

  //----------------------------------------
  // CastAbles
  //----------------------------------------
  private final def genCastAbles(): String = {
    e"""impl CastAble for foreign::Foreign {
       §    fn cast_id() -> usize {
       §        ${IR.size}
       §    }
       §}
       § §impl CastAble for SkillObject {
       §    fn cast_id() -> usize {
       §        ${IR.size + IRInterfaces.size + 1}
       §    }
       §}
       §
       §""".stripMargin('§') +
    (for (base ← IR) yield {
      e"""impl CastAble for ${name(base)} {
         §    fn cast_id() -> usize {
         §        ${genTypeId(base)}
         §    }
         §}
         §impl CastAble for ${traitName(base)} {
         §    fn cast_id() -> usize {
         §        ${genTypeId(base)}
         §    }
         §}
         §
         §""".stripMargin('§')
    }).mkString +
    (for (base ← IRInterfaces) yield {
      e"""impl CastAble for ${traitName(base)} {
         §    fn cast_id() -> usize {
         §        ${genTypeId(base)}
         §    }
         §}
         §
         §""".stripMargin('§')
    }).mkString
  }.trim
}
