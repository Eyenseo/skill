/*  ___ _  ___ _ _                                                            *\
** / __| |/ (_) | |       The SKilL Generator                                 **
** \__ \ ' <| | | |__     (c) 2013-18 University of Stuttgart                 **
** |___/_|\_\_|_|____|    see LICENSE                                         **
\*                                                                            */
package de.ust.skill.generator.scala

import java.io.BufferedWriter
import java.io.File
import java.io.FileOutputStream
import java.io.OutputStreamWriter
import java.io.PrintWriter

import scala.collection.JavaConversions.asScalaBuffer

import de.ust.skill.generator.common.Generator
import de.ust.skill.ir.Field
import de.ust.skill.ir.FieldLike
import de.ust.skill.ir.InterfaceType
import de.ust.skill.ir.Type
import de.ust.skill.ir.TypeContext
import de.ust.skill.ir.UserType

/**
 * The parent class for all output makers.
 *
 * @author Timm Felden
 */
trait GeneralOutputMaker extends Generator {

  // remove special stuff
  final def setTC(tc : TypeContext) {
    this.types = tc
    this.IR = tc.removeTypedefs.removeEnums.getUsertypes.to
    this.IRInterfaces = tc.removeTypedefs.removeEnums.getInterfaces.to
    // set large specification mode; leave some spare parameters
    largeSpecificationMode = IR.size > 200
  }
  var types : TypeContext = _
  var IR : Array[UserType] = _
  var IRInterfaces : Array[InterfaceType] = _

  /**
   * This flag is set iff the specification is too large to be passed as parameter list
   */
  var largeSpecificationMode = false

  override def getLanguageName : String = "scala";

  // options
  /**
   * if set to true, the generated binding will reveal the values of skill IDs.
   */
  protected var revealSkillID = false;

  val ArrayTypeName = "scala.collection.mutable.ArrayBuffer"
  val VarArrayTypeName = "scala.collection.mutable.ArrayBuffer"
  val ListTypeName = "scala.collection.mutable.ListBuffer"
  val SetTypeName = "scala.collection.mutable.HashSet"
  val MapTypeName = "scala.collection.mutable.HashMap"

  /**
   * Assume the existence of a translation function for types.
   */
  protected def mapType(t : Type) : String

  /**
   * creates argument list of a constructor call, not including potential skillID or braces
   */
  protected def makeConstructorArguments(t : UserType) : String
  /**
   * creates argument list of a constructor call, including a trailing comma for insertion into an argument list
   */
  protected def appendConstructorArguments(t : UserType) : String

  /**
   * turns a declaration and a field into a string writing that field into an outStream
   * @note the used iterator is "outData"
   * @note the used target OutStream is "dataChunk"
   */
  protected def writeField(d : UserType, f : Field) : String

  /**
   * Translation of a type to its representation in the source code
   */
  protected def name(t : Type) : String = escaped(t.getName.capital)
  protected def storagePool(t : Type) : String = escaped(t.getName.capital + "Pool")
  protected def subPool(t : Type) : String = escaped(t.getName.capital + "SubPool")

  protected def name(f : FieldLike) : String = escaped(f.getName.camel)
  protected def knownField(f : Field) : String =
    escaped(s"F_${f.getDeclaredIn.getName.capital()}_${f.getName.camel()}")

  /**
   * Assume a package prefix provider.
   */
  protected def packagePrefix() : String
  protected def packageName = packagePrefix.substring(0, packagePrefix.length - 1)
}
