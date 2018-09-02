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
                   §
                   §use std::collections::HashMap;
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
      map += (t → makeInheritanceMap(t))
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

  private final def genLookUpMap(): String = {
    val maps = makeInheritanceMap()

    e"""lazy_static! {
       §    pub(crate) static ref VALID_CASTS: HashMap<::std::any::TypeId, HashMap<std::any::TypeId, VTable>> = {
       §        use std::any::TypeId;
       §        use std::mem::transmute;
       §        use std::ptr::null;
       §
       §        let mut map = HashMap::with_capacity(${IR.size + 1});
       §        map.insert(
       §            TypeId::of::<foreign::Foreign>(),
       §            unsafe {
       §                let mut map = HashMap::with_capacity(2);
       §                map.insert(
       §                    TypeId::of::<foreign::Foreign>(),
       §                    VTable::none(),
       §                );
       §                map.insert(
       §                    TypeId::of::<SkillObject>(),
       §                    transmute::<_, TraitObject>(
       §                        null::<foreign::Foreign>() as *const SkillObject
       §                    ).vtable,
       §                );
       §                map
       §            }
       §        );
       §        ${
      (for ((base, map) ← maps) yield {
        e"""map.insert(
           §    TypeId::of::<${name(base)}>(),
           §    unsafe {
           §        let mut map = HashMap::with_capacity(${map.size * 2 + 2});
           §        map.insert(
           §            TypeId::of::<foreign::Foreign>(),
           §            VTable::none(),
           §        );
           §        map.insert(
           §            TypeId::of::<SkillObject>(),
           §            transmute::<_, TraitObject>(
           §                null::<${name(base)}>() as *const SkillObject
           §            ).vtable,
           §        );
           §        ${
          (for (t ← map) yield {
            if (t.isInstanceOf[InterfaceType]) {
              e"""map.insert(
                 §    TypeId::of::<${traitName(t)}>(),
                 §    transmute::<_, TraitObject>(
                 §        null::<${name(base)}>() as *const ${traitName(t)}
                 §    ).vtable,
                 §);
                 §""".stripMargin('§')
            } else {
              e"""map.insert(
                 §    TypeId::of::<${name(t)}>(),
                 §    VTable::none(),
                 §);
                 §map.insert(
                 §    TypeId::of::<${traitName(t)}>(),
                 §    transmute::<_, TraitObject>(
                 §        null::<${name(t)}>() as *const ${traitName(t)}
                 §    ).vtable,
                 §);
                 §""".stripMargin('§')
            }
          }).mkString.trim
        }
           §        map
           §    }
           §);
           §""".stripMargin('§')
      }).mkString.trim
    }
       §        map
       §    };
       §}
       §""".stripMargin('§')
  }
}
