/*  ___ _  ___ _ _                                                            *\
** / __| |/ (_) | |       The SKilL Generator                                 **
** \__ \ ' <| | | |__     (c) 2013-18 University of Stuttgart                 **
** |___/_|\_\_|_|____|    see LICENSE                                         **
\*                                                                            */
package de.ust.skill.generator.rust

import de.ust.skill.ir._
import de.ust.skill.main.HeaderInfo

import scala.collection.JavaConversions.asScalaBuffer
import scala.collection.mutable

/**
  * Fake Main implementation required to make trait stacking work.
  */
abstract class FakeMain extends GeneralOutputMaker {
  def make {}
}

final class Main extends FakeMain
                 // with FieldDeclarationsMaker
                 with SkillFileMaker
                 // with StringKeeperMaker
                 with PoolsMaker
                 with LibMaker
                 // with TypesMaker
                 with PtrMaker
                 with DependenciesMaker {
  lineLength = 100

  /**
    * Tries to escape a string without decreasing the usability of the generated identifier.
    */
  private val escapeCache            = new mutable.HashMap[String, String]()
  private var _packagePrefix: String = _

  override def comment(d: Declaration): String = d.getComment.format("/**\n", "     * ", lineLength, "     */\n    ")

  override def comment(f: FieldLike): String = f.getComment
                                               .format("/**\n", "         * ", lineLength, "         */\n        ")

  override def packageDependentPathPostfix: String = ""


  override def defaultCleanMode: String = "file"

  override def mapType(t: Type): String = t match {
    case t: GroundType ⇒ t.getName.lower match {
      case "bool" ⇒ "bool"

      case "i8"  ⇒ "i8"
      case "i16" ⇒ "i16"
      case "i32" ⇒ "i32"
      case "i64" ⇒ "i64"
      case "v64" ⇒ "i64"

      case "f32" ⇒ "f32"
      case "f64" ⇒ "f64"

      case "string"     ⇒ "Rc<String>"
      case "annotation" ⇒ "Option<Ptr<SkillObject>>"

      case _ ⇒ throw new GeneratorException(s"Unhandled type $t")
    }

    case t: ConstantLengthArrayType ⇒ s"Vec<${mapType(t.getBaseType)}>"
    case t: VariableLengthArrayType ⇒ s"Vec<${mapType(t.getBaseType)}>"
    case t: ListType                ⇒ s"LinkedList<${mapType(t.getBaseType)}>"
    case t: SetType                 ⇒ s"HashSet<${mapType(t.getBaseType)}>"
    case t: MapType                 ⇒ t.getBaseTypes.map(mapType).reduceRight((k, v) ⇒ s"HashMap<$k, $v>")

    case t: UserType ⇒ s"Option<Ptr<${t.getName.capital()}T>>" // TODO are we able to infer Struct vs Type?

    case _ ⇒ throw new GeneratorException(s"Unknown type $t")
  }

  override def makeHeader(headerInfo: HeaderInfo): String = headerInfo
                                                            .format(this, "/*", "*\\", " *", "* ", "\\*", "*/")

  override def setPackage(names: List[String]) {
    _packagePrefix = names.foldRight("")(_ + "." + _)
  }

  override def setOption(option: String, value: String) {
    option match {
      case "revealskillid"   ⇒ revealSkillID = "true".equals(value)
      case "interfacechecks" ⇒ interfaceChecks = "true".equals(value)
      case unknown           ⇒ throw new GeneratorException(s"unknown Argument: $unknown")
    }
  }

  override def helpText: String =
    """
      |revealSkillID     true/false  if set to true, the generated binding will reveal SKilL IDs in the API
      |interfaceChecks   true/false  if set to true, the generated API will contain is[[interface]] methods
      |""".stripMargin

  override def customFieldManual: String =
    """
      |!include string+    Argument strings are added to the head of the generated file and included using
      |                    <> around the strings content.""".stripMargin

  override def escaped(target: String): String = escapeCache.getOrElse(target, {
    val result = EscapeFunction(target)
    escapeCache(target) = result
    result
  })

  override protected def unbox(t: Type): String = t match {
    case t: GroundType ⇒ t.getName.lower

    case _: ConstantLengthArrayType ⇒ "array"
    case _: VariableLengthArrayType ⇒ "list"
    case _: ListType                ⇒ "list"
    case _: SetType                 ⇒ "set"
    case _: MapType                 ⇒ "map"

    //    case _: Declaration ⇒ "annotation"

    case _ ⇒ throw new GeneratorException(s"Unknown type $t")
  }

  /**
    * creates argument list of a constructor call, not including potential skillID or braces
    */
  override protected def makeConstructorArguments(t: UserType): String =
    (for (f ← t.getAllFields if !(f.isConstant || f.isIgnored)) yield {
      s"${escaped(f.getName.camel)} : ${mapType(f.getType)}"
    }).mkString(", ")

  override protected def appendConstructorArguments(t: UserType): String = {
    val r = t.getAllFields.filterNot { f ⇒ f.isConstant || f.isIgnored }
    if (r.isEmpty) {
      ""
    } else {
      r.map({ f ⇒ s"${escaped(f.getName.camel)} : ${mapType(f.getType)}" }).mkString(", ", ", ", "")
    }
  }

  /**
    * provides the package prefix
    */
  override protected def packagePrefix(): String = _packagePrefix

  override protected def defaultValue(f: Field): String =
    f.getType match {
      case t: GroundType ⇒ t.getSkillName match {
        case "i8" | "i16" | "i32" | "i64" | "v64" ⇒ "0"
        case "f32" | "f64"                        ⇒ "0.0"
        case "bool"                               ⇒ "false"
        case "string"                             ⇒ "Rc::default()" // FIXME string
        case "annotation"                         ⇒ "None"
        case _                                    ⇒ throw new GeneratorException(s"Unhandled type $t")
      }


      case _: ConstantLengthArrayType ⇒ "Vec::default()"
      case _: VariableLengthArrayType ⇒ "Vec::default()"
      case _: ListType                ⇒ "LinkedList::default()"
      case _: SetType                 ⇒ "HashSet::default()"
      case _: MapType                 ⇒ "HashMap::default()"

      case _: UserType ⇒ "None"

      case t ⇒ throw new GeneratorException(s"Unknown type $t")
    }

  protected def filterIntarfacesFromIR() {
    // find implementers
    val ts = types.removeTypedefs()
    for (t ← ts.getUsertypes) {
      val is: mutable.HashSet[InterfaceType] = t.getSuperInterfaces
                                               .flatMap(recursiveSuperInterfaces(_, new mutable.HashSet[InterfaceType]))
                                               .to
      interfaceCheckImplementations(t.getSkillName) = is.map(insertInterface(_, t))
    }
  }

  protected def writeField(d: UserType, f: Field): String = {
    val fName = escaped(f.getName.camel)

    if (f.isConstant) {
      "// constants do not write individual field data"
    } else {
      f.getType match {
        case t: GroundType ⇒ t.getSkillName match {
          case "annotation" | "string" ⇒ s"for(i ← outData) ${f.getType.getSkillName}(i.$fName, dataChunk)"
          case _                       ⇒ s"for(i ← outData) dataChunk.${f.getType.getSkillName}(i.$fName)"

        }

        case _: Declaration ⇒ s"""for(i ← outData) userRef(i.$fName, dataChunk)"""

        case t: ConstantLengthArrayType ⇒ s"for(i ← outData) writeConstArray(${
          t.getBaseType match {
            case t: Declaration ⇒ s"userRef[${mapType(t)}]"
            case b              ⇒ b.getSkillName
          }
        })(i.$fName, dataChunk)"
        case t: VariableLengthArrayType ⇒ s"for(i ← outData) writeVarArray(${
          t.getBaseType match {
            case t: Declaration ⇒ s"userRef[${mapType(t)}]"
            case b              ⇒ b.getSkillName
          }
        })(i.$fName, dataChunk)"
        case t: SetType                 ⇒ s"for(i ← outData) writeSet(${
          t.getBaseType match {
            case t: Declaration ⇒ s"userRef[${mapType(t)}]"
            case b              ⇒ b.getSkillName
          }
        })(i.$fName, dataChunk)"
        case t: ListType                ⇒ s"for(i ← outData) writeList(${
          t.getBaseType match {
            case t: Declaration ⇒ s"userRef[${mapType(t)}]"
            case b              ⇒ b.getSkillName
          }
        })(i.$fName, dataChunk)"

        case t: MapType ⇒ locally {
          s"for(i ← outData) ${
            t.getBaseTypes.map {
              case t: Declaration ⇒ s"userRef[${mapType(t)}]"
              case b              ⇒ b.getSkillName
            }.reduceRight { (t, v) ⇒
              s"writeMap($t, $v)"
            }
          }(i.$fName, dataChunk)"
        }
      }
    }
  }

  private def insertInterface(i: InterfaceType, target: UserType): String = {
    // register a potential implementation for the target type and interface
    i.getBaseType match {
      case b: UserType ⇒
        interfaceCheckMethods.getOrElseUpdate(b.getSkillName, new mutable.HashSet[String]) += i.getName.capital()
      case _           ⇒
        interfaceCheckMethods.getOrElseUpdate(target.getBaseType.getSkillName, new mutable.HashSet[String]) +=
        i.getName.capital()
    }
    // return the name to be used
    i.getName.capital
  }

  private def recursiveSuperInterfaces(i: InterfaceType,
                                       r: mutable.HashSet[InterfaceType]): mutable.HashSet[InterfaceType] = {
    r += i
    for (s ← i.getSuperInterfaces) {
      recursiveSuperInterfaces(s, r)
    }
    r
  }
}

object EscapeFunction {
  def apply(target: String): String = target match {
    //keywords get a suffix "_", because that way at least auto-completion will work as expected
    case "auto" | "const" | "double" | "float" | "int" | "short" | "struct" | "unsigned" | "break" | "continue"
         | "else" | "for" | "long" | "signed" | "switch" | "void" | "case" | "default" | "enum" | "goto" | "register"
         | "sizeof" | "typedef" | "volatile" | "char" | "do" | "extern" | "if" | "return" | "static" | "union" | "while"
         | "asm" | "dynamic_cast" | "namespace" | "reinterpret_cast" | "try" | "bool" | "explicit" | "new" |
         "static_cast"
         | "typeid" | "catch" | "false" | "operator" | "template" | "typename" | "class" | "friend" | "private" | "this"
         | "using" | "const_cast" | "inline" | "public" | "throw" | "virtual" | "delete" | "mutable" | "protected"
         | "true" | "wchar_t" | "and" | "bitand" | "compl" | "not_eq" | "or_eq" | "xor_eq" | "and_eq" | "bitor" | "not"
         | "or" | "xor" | "cin" | "endl" | "INT_MIN" | "iomanip" | "main" | "npos" | "std" | "cout" | "include"
         | "INT_MAX" | "iostream" | "MAX_RAND" | "NULL" | "string" ⇒ s"_$target"

    case t if t.forall(c ⇒ '_' == c || Character.isLetterOrDigit(c)) ⇒ t

    case _ ⇒ target.map {
      case 'Z'                                           ⇒ "ZZ"
      case c if '_' == c || Character.isLetterOrDigit(c) ⇒ "" + c
      case c                                             ⇒ "Z" + c.formatted("%04X")
    }.mkString
  }
}
