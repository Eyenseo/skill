/*  ___ _  ___ _ _                                                            *\
** / __| |/ (_) | |       The SKilL Generator                                 **
** \__ \ ' <| | | |__     (c) 2013-16 University of Stuttgart                 **
** |___/_|\_\_|_|____|    see LICENSE                                         **
\*                                                                            */
package de.ust.skill.generator.java

import java.io.BufferedWriter
import java.io.File
import java.io.FileOutputStream
import java.io.OutputStreamWriter
import java.io.PrintWriter

import scala.collection.JavaConversions.asScalaBuffer

import de.ust.skill.generator.common.Generator
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
    val flat = tc.removeTypedefs.removeEnums
    this.IR = flat.getUsertypes.to
    this.interfaces = flat.getInterfaces.to
    // set large specification mode; leave some spare parameters
    largeSpecificationMode = IR.size > 200
  }
  var types : TypeContext = _
  var IR : List[UserType] = _
  var interfaces : List[InterfaceType] = _

  /**
   * This flag is set iff the specification is too large to be passed as parameter list
   */
  var largeSpecificationMode = false

  override def getLanguageName : String = "java";

  override def clean {
    deleteRecursively(new File(outPath + "/" + packagePath))
  }

  // options
  /**
   * if set to true, the generated binding will reveal the values of skill IDs.
   */
  protected var revealSkillID = false;

  /**
   * if set to true, the generated binding will also contain visitors for each
   * base type
   */
  protected var createVisitors = false;

  val ArrayTypeName = "java.util.ArrayList"
  val VarArrayTypeName = "java.util.ArrayList"
  val ListTypeName = "java.util.LinkedList"
  val SetTypeName = "java.util.HashSet"
  val MapTypeName = "java.util.HashMap"

  private[java] def header : String

  /**
   * Creates the correct PrintWriter for the argument file.
   *
   * @note the used path uses maven/sbt source placement convention
   */
  override protected def open(path : String) = {
    val f = simpleOpenDirtyPathString(s"$outPath/$packagePath$path")

    val rval = new PrintWriter(new BufferedWriter(new OutputStreamWriter(
      new FileOutputStream(f), "UTF-8")))
    rval.write(header)
    rval
  }

  /**
   * Assume the existence of a translation function for types.
   */
  protected def mapType(t : Type, boxed : Boolean = false) : String

  /**
   * Assume the existence of a translation function for types that creates
   * variant container types.
   */
  protected def mapVariantType(t : Type) : String

  /**
   * the name of an interface field type that acts as its pool
   */
  protected final def interfacePool(t : InterfaceType) : String =
    if (t.getSuperType.getSkillName.equals("annotation"))
      s"de.ust.skill.common.java.internal.UnrootedInterfacePool<${mapType(t)}>"
    else
      s"de.ust.skill.common.java.internal.InterfacePool<${mapType(t)}, ${mapType(t.getBaseType)}>"

  /**
   * creates argument list of a constructor call, not including potential skillID or braces
   */
  protected def makeConstructorArguments(t : UserType) : String
  /**
   * creates argument list of a constructor call, including a trailing comma for insertion into an argument list
   */
  protected def appendConstructorArguments(t : UserType, prependTypes : Boolean = true) : String

  /**
   * Translation of a type to its representation in the source code
   */
  protected def name(t : Type) : String = escaped(t.getName.capital)
  /**
   * Translation of a field to its representation in the source code
   */
  protected def name(f : FieldLike) : String = escaped(f.getName.camel)

  /**
   * Class name of the representation of a known field
   */
  protected def knownField(f : FieldLike) : String = s"F_${name(f.getDeclaredIn)}_${name(f)}"

  /**
   * Assume a package prefix provider.
   */
  protected def packagePrefix() : String
  protected def packageName = packagePrefix.substring(0, packagePrefix.length - 1)

  private lazy val packagePath = if (packagePrefix.length > 0) {
    packagePrefix.replace(".", "/")
  } else {
    ""
  }

  /**
   * this string may contain a "@SuppressWarnings("all")\n", in order to suppress warnings in generated code;
   * the option can be enabled by "-O@java:SuppressWarnings=true"
   */
  protected var suppressWarnings = "";
}
