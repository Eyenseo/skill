/*  ___ _  ___ _ _                                                            *\
 * / __| |/ (_) | |       Your SKilL Scala Binding                            *
 * \__ \ ' <| | | |__     generated: 06.12.2016                               *
 * |___/_|\_\_|_|____|    by: feldentm                                        *
\*                                                                            */
package de.ust.skill.sir.api.internal

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
import de.ust.skill.common.scala.internal.SkillState
import de.ust.skill.common.scala.internal.SingletonStoragePool
import de.ust.skill.common.scala.internal.StoragePool
import de.ust.skill.common.scala.internal.SubPool
import de.ust.skill.common.scala.internal.fieldTypes._
import de.ust.skill.common.scala.internal.restrictions.FieldRestriction

import _root_.de.ust.skill.sir.api._

final class SingleBaseTypeContainerPool(poolIndex : Int,
superPool: BuiltinTypePool)
    extends SubPool[_root_.de.ust.skill.sir.SingleBaseTypeContainer, de.ust.skill.sir.Type](
      poolIndex,
      "singlebasetypecontainer",
superPool
    ) {
  override def getInstanceClass: Class[_root_.de.ust.skill.sir.SingleBaseTypeContainer] = classOf[_root_.de.ust.skill.sir.SingleBaseTypeContainer]

  override def addField[T : Manifest](ID : Int, t : FieldType[T], name : String,
                           restrictions : HashSet[FieldRestriction]) : FieldDeclaration[T, _root_.de.ust.skill.sir.SingleBaseTypeContainer] = {
    val f = (name match {
      case "base" ⇒ new KnownField_SingleBaseTypeContainer_base(ID, this, t.asInstanceOf[FieldType[_root_.de.ust.skill.sir.GroundType]])
      case "kind" ⇒ new KnownField_SingleBaseTypeContainer_kind(ID, this, t.asInstanceOf[FieldType[java.lang.String]])
      case _      ⇒ return super.addField(ID, t, name, restrictions)
    }).asInstanceOf[FieldDeclaration[T, _root_.de.ust.skill.sir.SingleBaseTypeContainer]]

    //check type
    if (t != f.t)
      throw new TypeMissmatchError(t, f.t.toString, f.name, name)

    restrictions.foreach(f.addRestriction(_))
    dataFields += f
    return f
  }
  override def ensureKnownFields(st : SkillState) {
    val state = st.asInstanceOf[SkillFile]
    // data fields
    val Clsbase = classOf[KnownField_SingleBaseTypeContainer_base]
    val Clskind = classOf[KnownField_SingleBaseTypeContainer_kind]

    val fields = HashSet[Class[_ <: FieldDeclaration[_, _root_.de.ust.skill.sir.SingleBaseTypeContainer]]](Clsbase,Clskind)
    var dfi = dataFields.size
    while (dfi != 0) {
      dfi -= 1
      fields.remove(dataFields(dfi).getClass)
    }
    if(fields.contains(Clsbase))
        dataFields += new KnownField_SingleBaseTypeContainer_base(dataFields.size + 1, this, state.GroundType)
    if(fields.contains(Clskind))
        dataFields += new KnownField_SingleBaseTypeContainer_kind(dataFields.size + 1, this, state.String)
    // no auto fields


    for(f <- dataFields ++ autoFields)
      f.createKnownRestrictions
  }

  override def makeSubPool(name : String, poolIndex : Int) = new SingleBaseTypeContainerSubPool(poolIndex, name, this)
  override def reflectiveAllocateInstance: _root_.de.ust.skill.sir.SingleBaseTypeContainer = {
    val r = new _root_.de.ust.skill.sir.SingleBaseTypeContainer(-1)
    this.newObjects.append(r)
    r
  }

  override def allocateInstances {
    for (b ← blocks.par) {
      var i : SkillID = b.bpo
      val last = i + b.staticCount
      while (i < last) {
        data(i) = new _root_.de.ust.skill.sir.SingleBaseTypeContainer(i + 1)
        i += 1
      }
    }
  }

  def make(base : _root_.de.ust.skill.sir.GroundType = null, kind : java.lang.String = null) = {
    val r = new _root_.de.ust.skill.sir.SingleBaseTypeContainer(-1 - newObjects.size, base : _root_.de.ust.skill.sir.GroundType, kind : java.lang.String)
    newObjects.append(r)
    r
  }
}

final class SingleBaseTypeContainerSubPool(poolIndex : Int, name : String, superPool : StoragePool[_ >: _root_.de.ust.skill.sir.SingleBaseTypeContainer.UnknownSubType <: _root_.de.ust.skill.sir.SingleBaseTypeContainer, _root_.de.ust.skill.sir.Type])
    extends SubPool[_root_.de.ust.skill.sir.SingleBaseTypeContainer.UnknownSubType, _root_.de.ust.skill.sir.Type](
      poolIndex,
      name,
      superPool
    ) {
  override def getInstanceClass : Class[_root_.de.ust.skill.sir.SingleBaseTypeContainer.UnknownSubType] = classOf[_root_.de.ust.skill.sir.SingleBaseTypeContainer.UnknownSubType]

  override def makeSubPool(name : String, poolIndex : Int) = new SingleBaseTypeContainerSubPool(poolIndex, name, this)

  override def ensureKnownFields(st : SkillState) {}

  override def allocateInstances {
      for (b ← blocks.par) {
        var i : SkillID = b.bpo
        val last = i + b.staticCount
        while (i < last) {
          data(i) = new _root_.de.ust.skill.sir.SingleBaseTypeContainer.UnknownSubType(i + 1, this)
          i += 1
        }
      }
    }

    def reflectiveAllocateInstance : _root_.de.ust.skill.sir.SingleBaseTypeContainer.UnknownSubType = {
      val r = new _root_.de.ust.skill.sir.SingleBaseTypeContainer.UnknownSubType(-1, this)
      this.newObjects.append(r)
      r
    }
}
