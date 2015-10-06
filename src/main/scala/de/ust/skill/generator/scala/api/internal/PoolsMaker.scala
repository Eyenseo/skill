/*  ___ _  ___ _ _                                                            *\
** / __| |/ (_) | |       The SKilL Generator                                 **
** \__ \ ' <| | | |__     (c) 2013-15 University of Stuttgart                 **
** |___/_|\_\_|_|____|    see LICENSE                                         **
\*                                                                            */
package de.ust.skill.generator.scala.api.internal

import scala.collection.JavaConversions.asScalaBuffer
import de.ust.skill.generator.scala.GeneralOutputMaker
import de.ust.skill.ir.View
import de.ust.skill.ir.restriction.SingletonRestriction
import de.ust.skill.ir.ReferenceType
import de.ust.skill.ir.ContainerType

trait PoolsMaker extends GeneralOutputMaker {
  abstract override def make {
    super.make
    val out = open("api/internal/Pools.scala")
    //package
    out.write(s"""package ${packagePrefix}api.internal""")
    out.write(s"""

import scala.collection.mutable.ArrayBuffer
import scala.collection.mutable.HashMap
import scala.collection.mutable.HashSet
import scala.collection.mutable.ListBuffer
import scala.collection.mutable.WrappedArray
import scala.reflect.Manifest

import de.ust.skill.common.jvm.streams.InStream

import de.ust.skill.common.scala.SkillID
import de.ust.skill.common.scala.api.SkillObject
import de.ust.skill.common.scala.api.TypeMissmatchError
import de.ust.skill.common.scala.internal.BasePool
import de.ust.skill.common.scala.internal.FieldDeclaration
import de.ust.skill.common.scala.internal.StoragePool
import de.ust.skill.common.scala.internal.SubPool
import de.ust.skill.common.scala.internal.fieldTypes.AnnotationType
import de.ust.skill.common.scala.internal.fieldTypes.FieldType
import de.ust.skill.common.scala.internal.fieldTypes.StringType
import de.ust.skill.common.scala.internal.restrictions.FieldRestriction

import _root_.${packagePrefix}api._
""")

    for (t ← IR) {
      val typeName = "_root_."+packagePrefix + name(t)
      val isSingleton = !t.getRestrictions.collect { case r : SingletonRestriction ⇒ r }.isEmpty
      val fields = t.getFields.filterNot(_.isInstanceOf[View])

      out.write(s"""
final class ${storagePool(t)}(poolIndex : Int${
        if (t.getSuperType == null) ""
        else s",\nsuperPool: ${storagePool(t.getSuperType)}"
      })
    extends ${
        if (t.getSuperType == null) s"BasePool[$typeName]"
        else s"SubPool[$typeName, ${packagePrefix}${t.getBaseType.getName.capital}]"
      }(
      poolIndex,
      "${t.getSkillName}"${
        if (t.getSuperType == null) ""
        else ",\nsuperPool"
      },
      Set(${
        if (fields.isEmpty) ""
        else (for (f ← fields) yield s"""\n        "${f.getSkillName}"""").mkString("", ",", "\n      ")
      })
    ) {
  override def getInstanceClass: Class[$typeName] = classOf[$typeName]

  override def addField[T : Manifest](ID : Int, t : FieldType[T], name : String,
                           restrictions : HashSet[FieldRestriction]) : FieldDeclaration[T, $typeName] = {
    val f = (name match {${
        (for (f ← fields)
          yield s"""
      case "${f.getSkillName}" ⇒ new ${knownField(f)}(${
          if (f.isAuto()) ""
          else "ID, "
        }this${
          if (f.getType.isInstanceOf[ReferenceType] || f.getType.isInstanceOf[ContainerType])
            s""", t.asInstanceOf[FieldType[${mapType(f.getType)}]]"""
          else ""
        })"""
        ).mkString("")
      }
      case _      ⇒ return super.addField(ID, t, name, restrictions)
    }).asInstanceOf[FieldDeclaration[T, $typeName]]

    //check type
    if (t != f.t)
      throw new TypeMissmatchError(t, f.t.toString, f.name, name)

    restrictions.foreach(f.addRestriction(_))
    dataFields += f
    return f
  }
/*  override def addKnownField[T](name : String, mkType : FieldType[T] ⇒ FieldType[T]) {${
        if (fields.isEmpty) "/* no known fields */"
        else s"""
    val f = (name match {
${
          (for (f ← fields)
            yield s"""      case "${f.getSkillName}" ⇒ new ${knownField(f)}(${
            if (f.isAuto()) ""
            else "fields.size, "
          }this${
            val t = f.getType.getSkillName
            if ("annotation" == t || "string" == t) s""",
  _type : FieldType[${mapType(f.getType)}]"""
            else ""
          })"""
          ).mkString("\n")
        }
    }).asInstanceOf[FieldDeclaration[T, $typeName]]
    f.t = mkType(f.t)
    fields += f
  """
      }} */

  override def reflectiveAllocateInstance: $typeName = {
    val r = new $typeName(-1)
    this.newObjects.append(r)
    r
  }

  override def makeSubPool(name : String, poolIndex : Int) = ${
        if (isSingleton) s"""throw new NoSuchMethodError("${t.getName.capital} is a Singleton and can therefore not have any subtypes.")"""
        else s"new ${subPool(t)}(poolIndex, name, this)"
      }${
        if (null == t.getSuperType) s"""

  override def allocateData : Unit = data = new Array[$typeName](cachedSize)"""
        else""
      }

  override def allocateInstances {
    for (b ← blocks.par) {
      var i : SkillID = b.bpo
      val last = i + b.staticCount
      while (i < last) {
        data(i) = new $typeName(i + 1)
        i += 1
      }
    }
  }

${
        if (isSingleton)
          s"""  lazy val theInstance = if (staticInstances.hasNext) {
    staticInstances.next
  } else {
    val r = new $typeName(-1)
    newObjects.append(r)
    r
  }
  def get = theInstance"""
        else
          s"""  def make(${makeConstructorArguments(t)}) = {
    val r = new $typeName(-1${appendConstructorArguments(t)})
    newObjects.append(r)
    r
  }"""
      }
}
""")

      if (!isSingleton) {
        // create a sub pool
        out.write(s"""
final class ${subPool(t)}(poolIndex : Int, name : String, superPool : StoragePool[_ >: $typeName.UnknownSubType <: $typeName, _root_.${packagePrefix}${t.getBaseType.getName.capital}])
    extends SubPool[$typeName.UnknownSubType, _root_.${packagePrefix}${name(t.getBaseType)}](
      poolIndex,
      name,
      superPool,
      StoragePool.noKnownFields
    ) {
  override def getInstanceClass : Class[$typeName.UnknownSubType] = classOf[$typeName.UnknownSubType]

  override def makeSubPool(name : String, poolIndex : Int) = new ${subPool(t)}(poolIndex, name, this)

  override def allocateInstances {
      for (b ← blocks.par) {
        var i : SkillID = b.bpo
        val last = i + b.staticCount
        while (i < last) {
          data(i) = new $typeName.UnknownSubType(i + 1, this)
          i += 1
        }
      }
    }

    def reflectiveAllocateInstance : $typeName.UnknownSubType = {
      val r = new $typeName.UnknownSubType(-1, this)
      this.newObjects.append(r)
      r
    }
}
""")
      }
    }

    //class prefix
    out.close()
  }
}
