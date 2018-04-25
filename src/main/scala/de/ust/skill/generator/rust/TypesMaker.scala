/*  ___ _  ___ _ _                                                            *\
** / __| |/ (_) | |       The SKilL Generator                                 **
** \__ \ ' <| | | |__     (c) 2013-18 University of Stuttgart                 **
** |___/_|\_\_|_|____|    see LICENSE                                         **
\*                                                                            */
package de.ust.skill.generator.rust

import de.ust.skill.ir.{Field, UserType}

import scala.collection.JavaConverters._
import scala.collection.mutable.ArrayBuffer

/**
  * creates header and implementation for all type definitions
  *
  * @author Timm Felden
  */
trait TypesMaker extends GeneralOutputMaker {

  abstract override def make {
    super.make

    // TODO add stuff
    //makeSource
  }

  @inline private final def fieldName(implicit f: Field): String = escaped(f.getName.capital())

  @inline private final def localFieldName(implicit f: Field): String = internalName(f)


  private final def makeSource {

    // one file per base type
    for (base <- IR if null == base.getSuperType) {
      val out = files.open(s"TypesOf${name(base)}.rs")
      out.write(
                 s"""#include "File.h"
#include "TypesOf${name(base)}.h"${
                   (for (t <- IR if base == t.getBaseType) yield
                     s"""

const char *const $packageName::${name(t)}::typeName = "${t.getSkillName}";
const char *$packageName::${name(t)}_UnknownSubType::skillName() const {
    return owner->name->c_str();
}${
                       if (visited.contains(t.getSkillName))
                         s"""
void $packageName::${name(t)}::accept($packageName::api::Visitor *v) {
    v->visit(this);
}"""
                       else ""
                     }""").mkString
                 }
""")
      out.close()
    }
  }

  private def gatherCustomIncludes(t: UserType): Seq[String] = {
    val x = t.getCustomizations.asScala.filter(_.language.equals("rust"))
            .flatMap {
                       case null ⇒ ArrayBuffer[String]()
                       case c    ⇒ val inc = c.getOptions.get("include")
                         if (null != inc) inc.asScala
                         else ArrayBuffer[String]()
                     }
    x ++ t.getSubTypes.asScala.flatMap(gatherCustomIncludes)
  }
}
