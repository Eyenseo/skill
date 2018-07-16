/*  ___ _  ___ _ _                                                            *\
** / __| |/ (_) | |       The SKilL Generator                                 **
** \__ \ ' <| | | |__     (c) 2013-18 University of Stuttgart                 **
** |___/_|\_\_|_|____|    see LICENSE                                         **
\*                                                                            */
package de.ust.skill.generator.rust

import de.ust.skill.generator.common.Generator
import de.ust.skill.ir._
import de.ust.skill.ir.restriction.CodingRestriction

import scala.collection.JavaConversions.`deprecated asScalaBuffer`
import scala.collection.mutable

trait GeneralOutputMaker extends Generator {
  /**
    * Takes a camel cased identifier name and returns an underscore separated
    * name
    *
    * Example:
    * camelToUnderscores("thisIsA1Test") == "this_is_a_1_test"
    *
    * https://gist.github.com/sidharthkuruvila/3154845
    */
  def snakeCase(text: String): String = {
    text.drop(1).foldLeft(
                           text.headOption.map(_.toLower + "")
                           getOrElse "") {
                                           case (acc, c) if c.isUpper => acc + "_" + c.toLower
                                           case (acc, c)              => acc + c
                                         }
  }

  final protected val endGuard    : String                     =
    """
#endif"""
  protected lazy  val packageParts: Array[String]              = packagePrefix().split("\\.").map(escaped)
  protected lazy  val packageName : String                     = packageParts.mkString("_")
  /**
    * all string literals used in type and field names
    *
    * @note first set are strings whose representations exist as type names
    * @note second set are strings that will be created and unified by the skill
    *       file.
    */
  protected lazy  val allStrings  : (Set[String], Set[String]) = {
    val types = IR.map(_.getSkillName).toSet
    val fields =
      IR.flatMap(_.getFields).map(_.getSkillName).toSet ++
      IR.flatMap(_.getFields).flatMap(_.getRestrictions).collect { case f: CodingRestriction ⇒ f }.map(_.getValue)
      .toSet --
      types

    (types, fields)
  }
  /**
    * If interfaceChecks then skillName -> Name of sub-interfaces
    *
    * @note the same interface can be sub and super, iff the type is a base type;
    *       in that case, super wins!
    */
  protected       val interfaceCheckMethods                    = new mutable.HashMap[String, mutable.HashSet[String]]

  // options
  /**
    * If interfaceChecks then skillName -> Name of super-interfaces
    */
  protected val interfaceCheckImplementations = new mutable.HashMap[String, mutable.HashSet[String]]
  var types: TypeContext     = _
  var IR   : List[UserType]  = _
  /**
    * This flag is set iff the specification is too large to be passed as parameter list
    */
  var largeSpecificationMode = false
  /**
    * If set to true, the generated binding will reveal the values of skill IDs.
    */
  protected var revealSkillID   = false
  /**
    * If set to true, the generated API will contain is_interface methods.
    * These methods return true iff the type implements that interface.
    * These methods exist for direct super types of interfaces.
    * For rootless interfaces, they exist in base types.
    */
  protected var interfaceChecks = false

  // remove special stuff
  final def setTC(tc: TypeContext) {
    this.types = tc
    this.IR = tc.removeSpecialDeclarations().getUsertypes.to
    // set large specification mode; leave some spare parameters
    largeSpecificationMode = IR.size > 200

    // filter implemented interfaces from original IR
    if (interfaceChecks) {
      filterIntarfacesFromIR()
    }
  }

  override def getLanguageName: String = "rust"

  protected def filterIntarfacesFromIR()

  /**
    * Assume the existence of a translation function for types.
    */
  protected def mapType(t: Type): String

  /**
    * Returns the selector required to turn a box into a useful type.
    *
    * @note does not include . or -> to allow usage in both cases
    */
  protected def unbox(t: Type): String

  /**
    * creates argument list of a constructor call, not including potential skillID or braces
    */
  protected def makeConstructorArguments(t: UserType): String

  /**
    * creates argument list of a constructor call, including a trailing comma for insertion into an argument list
    */
  protected def appendConstructorArguments(t: UserType): String

  /**
    * turns a declaration and a field into a string writing that field into an outStream
    *
    * @note the used iterator is "outData"
    * @note the used target OutStream is "dataChunk"
    */
  protected def writeField(d: UserType, f: Field): String

  // FIXME use this ...
  protected def storagePool(t: Type): String = escaped(t.getName.capital + "Pool")

  protected def subPool(t: Type): String = escaped(t.getName.capital + "SubPool")

  protected def name(f: Field): String = escaped(f.getName.camel)

  protected def internalName(f: Field): String = escaped("_" + f.getName.camel())

  protected def knownField(f: Field): String = escaped(s"KnownField_${f.getDeclaredIn.getName.capital()}_${
    f.getName.camel()
  }")

  protected def name(f: LanguageCustomization): String = escaped(f.getName.camel)

  /**
    * Assume a package prefix provider.
    */
  protected def packagePrefix(): String

  /**
    * start a guard word for a file
    */
  final protected def beginGuard(t: Type): String = beginGuard(escaped(name(t)))

  /**
    * Translation of a type to its representation in the source code
    */
  protected def name(t: Type): String = escaped(t.getName.capital)

  final protected def beginGuard(word: String): String = ""
}
