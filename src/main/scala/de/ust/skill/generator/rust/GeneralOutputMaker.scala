/*  ___ _  ___ _ _                                                            *\
** / __| |/ (_) | |       The SKilL Generator                                 **
** \__ \ ' <| | | |__     (c) 2013-18 University of Stuttgart                 **
** |___/_|\_\_|_|____|    see LICENSE                                         **
\*                                                                            */
package de.ust.skill.generator.rust

import de.ust.skill.generator.common.Generator
import de.ust.skill.ir._
import de.ust.skill.ir.restriction.CodingRestriction

import scala.annotation.tailrec
import scala.collection.JavaConverters._
import scala.collection.mutable
import scala.collection.mutable.ArrayBuffer

/**
  * @author Roland Jaeger
  */
object GeneralOutputMaker {
  /**
    * Takes a camel cased identifier name and returns an underscore separated
    * name
    *
    * Example:
    * camelToUnderscores("thisIsA1Test") == "this_is_a_1_test"
    *
    * https://gist.github.com/sidharthkuruvila/3154845
    */
  final def snakeCase(text: String): String = {
    @tailrec
    def go(accDone: List[Char], acc: List[Char]): List[Char] = acc match {
      case Nil                                                        =>
        accDone
      case a :: tail if a == '-'                                      =>
        go(accDone ++ List('_'), tail)
      case a :: b :: c :: tail if a.isUpper && b.isUpper && c.isLower =>
        go(accDone ++ List(a, '_', b, c), tail)
      case a :: b :: tail if a.isLower && b.isUpper                   =>
        go(accDone ++ List(a, '_', b), tail)
      case a :: tail                                                  =>
        go(accDone :+ a, tail)
    }

    go(Nil, text.toList).mkString.toLowerCase
  }

  /**
    * Takes a snake cased identifier name and returns camel cased name
    *
    * Example:
    * camelCase("this_is_a_1_test") == "thisIsA1Test"
    */
  final def camelCase(text: String): String = {
    @tailrec
    def go(accDone: List[Char], acc: List[Char]): List[Char] = acc match {
      case Nil                                      =>
        accDone
      case a :: b :: tail if a == '_' && b.isLetter =>
        go(accDone ++ List(b.toUpper), tail)
      case a :: b :: tail if a == '_'               =>
        go(accDone ++ List(b), tail)
      case a :: tail                                =>
        go(accDone :+ a, tail)
    }

    go(Nil, text.toList).mkString
  }
}

trait GeneralOutputMaker extends Generator {

  protected lazy val packageParts: Array[String] = packagePrefix().split("\\.").map(escaped)
  protected lazy val packageName : String        = packageParts.mkString("_")

  /**
    * all string literals used in type and field names
    *
    * @note first set are strings whose representations exist as type names
    * @note second set are strings that will be created and unified by the skill
    *       file.
    */
  protected lazy val allStrings: (Set[String], Set[String]) = {
    val types = (IR ::: IRInterfaces).map(_.getSkillName).toSet
    val fields =
      (IR ::: IRInterfaces).flatMap(_.getFields.asScala)
                           .map(_.getSkillName).toSet ++
      (IR ::: IRInterfaces).flatMap(_.getFields.asScala)
                           .flatMap(_.getRestrictions.asScala)
                           .collect { case f: CodingRestriction ⇒ f }
                           .map(_.getValue)
                           .toSet ++
      IR.flatMap(gatherCustoms(_))
      .map(_.getSkillName).toSet --
      types

    (types, fields)
  }

  /**
    * If interfaceChecks then skillName -> Name of sub-interfaces
    *
    * @note the same interface can be sub and super, iff the type is a base type;
    *       in that case, super wins!
    */
  protected val interfaceCheckMethods = new mutable.HashMap[String, mutable.HashSet[String]]

  // options
  var types       : TypeContext         = _
  var IR          : List[UserType]      = _
  var IRInterfaces: List[InterfaceType] = _
  /**
    * This flag is set iff the specification is too large to be passed as parameter list
    */
  var largeSpecificationMode            = false

  // remove special stuff
  final def setTC(tc: TypeContext) {
    this.types = tc
    val flat = tc.removeTypedefs().removeEnums()
    this.IR = flat.getUsertypes.asScala.to
    this.IRInterfaces = flat.getInterfaces.asScala.to
    // set large specification mode; leave some spare parameters
    largeSpecificationMode = IR.size > 200

    validateCustomOptions()
  }

  def validateCustomOptions(): Unit = {
    for (c ← (IR ::: IRInterfaces).toArray.flatMap(gatherCustoms)) {
      val ops = collection.mutable.Map() ++= c.getOptions.asScala.toMap

      ops.get("init") match {
        case Some(lst) ⇒ if (lst.size() != 1) {
          throw new GeneratorException(
                                        s"There has to be exactly one initialisation for each custom field but ${
                                          c
                                          .getName
                                        } had ${lst.size()}"
                                      )
        }
        case _         ⇒ throw new GeneratorException(
                                                       s"There has to be one initialisation (init) for each custom field!"
                                                     )
      }
      ops.remove("use")
      ops.remove("init")
      if (ops.nonEmpty) {
        throw new GeneratorException(
                                      s"""The Rust generator only supports two custom field options; 'init' and 'use'. Additionally given were "${
                                        ops.keys.mkString(", ")
                                      }"""")
      }
      c.getOptions
    }
  }

  override def getLanguageName: String = "rust"

  /**
    * Assume the existence of a translation function for types.
    */
  protected def mapType(t: Type): String

  protected def storagePool(t: Type): String = escaped(t.getName.capital + "Pool")

  protected def poolProxy(t: Type): String = escaped(t.getName.capital + "Proxy")

  protected def poolPartsMaker(t: Type): String = escaped(t.getName.capital + "PartsMaker")

  protected def interface(t: Type): String = escaped(t.getName.capital)

  protected def fieldDeclaration(t: Type, f: Field): String = escaped(t.getName.capital + f.getName.capital()) +
                                                              "FieldDeclaration"

  protected def readName(t: Type): String = t match {
    case t: GroundType ⇒ t.getName.lower

    case _: ConstantLengthArrayType ⇒ "array"
    case _: VariableLengthArrayType ⇒ "list"
    case _: ListType                ⇒ "list"
    case _: SetType                 ⇒ "set"
    case _: MapType                 ⇒ "map"

    case _ ⇒ throw new GeneratorException(s"Type '$t' is not supported for reading")
  }

  final def field(s: String): String = escaped(snakeCase(s)).toLowerCase

  final def field(f: Field): String = field(f.getName.camel())

  final def field(t: Type): String = field(t.getName.camel())

  final def pool(t: Type): String = t match {
    case t: InterfaceType ⇒ field(t.getBaseType.getName.camel())
    case _                ⇒ field(t.getName.camel())
  }

  final def traitName(t: Type): String = name(t) + "Object"

  final def name(t: Type): String = escaped(t.getName.capital)

  final def name(f: Field): String = escaped(snakeCase(f.getName.camel)).toLowerCase

  final def name(f: LanguageCustomization): String = escaped(f.getName.camel)

  final def snakeCase(str: String): String = GeneralOutputMaker.snakeCase(str)

  final def camelCase(str: String): String = GeneralOutputMaker.camelCase(str)

  /**
    * @param t Type to get the list of super types for
    * @return A list of all super types a given type t has
    */
  protected final def getAllSuperTypes(t: Declaration with WithInheritance): List[Declaration with WithInheritance] = {
    if (t == null) {
      List()
    } else if (t.getSuperType != null && t.getSuperType.isInstanceOf[Declaration with WithInheritance]) {
      getAllSuperTypes(t.getSuperType.asInstanceOf[Declaration with WithInheritance]) :::
      List(t)
    } else {
      List(t)
    }
  }.distinct

  protected final def getAllSupers(t: Declaration with WithInheritance): List[Declaration with WithInheritance] = {
    if (t == null) {
      return List()
    }
    var buff = ArrayBuffer[Declaration with WithInheritance]()

    if (t.getSuperType != null && t.getSuperType.isInstanceOf[Declaration with WithInheritance]) {
      buff = buff ++ getAllSupers(t.getSuperType.asInstanceOf[Declaration with WithInheritance])
    }

    for (i ← t.getSuperInterfaces.asScala) {
      buff = buff ++ getAllSupers(i).toArray
    }

    buff.append(t)

    buff.distinct.toList
  }

  /**
    * @param t Type to get the list of super types for
    * @return A list of all super types a given type t has
    */
  protected final def getAllSubTypes(t: UserType): List[Type] = {
    if (t.getSubTypes != null) {
      return List[UserType](t) ::: t.getSubTypes.asScala.toList.flatMap(t ⇒ getAllSubTypes(t))
    } else {
      List[UserType](t)
    }
  }.distinct

  /**
    * Assume a package prefix provider.
    */
  protected def packagePrefix(): String

  /**
    * @param base type to get all super interfaces for
    * @return a List of all super interfaces of the given type
    */
  protected final def allSuperInterfaces(base: Declaration with WithInheritance): List[InterfaceType] = {
    var ret: List[InterfaceType] = base.getSuperInterfaces.asScala.toList
    for (i ← base.getSuperInterfaces.asScala) {
      ret = ret ::: allSuperInterfaces(i)
    }
    ret.distinct
  }

  /**
    * @param base type to get all customisations for
    * @return a List of all customisations of the given type
    */
  protected final def gatherCustoms(base: WithFields): Seq[LanguageCustomization] = {
    if (base != null && base.getCustomizations != null) {
      val x = base.getCustomizations.asScala.filter(c ⇒ c.language.equals("rust")).flatMap {
        case null ⇒ ArrayBuffer[LanguageCustomization]()
        case c    ⇒ ArrayBuffer[LanguageCustomization](c)
      }
      base match {
        case base: UserType ⇒
          x ++
          getAllSuperTypes(base.getSuperType)
          .filter(_.isInstanceOf[WithFields])
          .map(_.asInstanceOf[WithFields])
          .flatMap(gatherCustoms)
        case _              ⇒ x
      }
    } else {
      ArrayBuffer[LanguageCustomization]()
    }
  }

  /**
    * @param base type to generate the id for
    * @return type id, used for the vtable lookup table
    */
  protected final def genTypeId(base: Declaration): Int = {
    val id = IR.indexOf(base)
    if (id == -1) {
      val id = IRInterfaces.indexOf(base)
      if (id == -1) {
        throw new GeneratorException("Couldn't find " + base.getName)
      }
      IR.size + 1 + id
    } else {
      id
    }
  }
}
