/*  ___ _  ___ _ _                                                            *\
** / __| |/ (_) | |       The SKilL Generator                                 **
** \__ \ ' <| | | |__     (c) 2013-18 University of Stuttgart                 **
** |___/_|\_\_|_|____|    see LICENSE                                         **
\*                                                                            */
package de.ust.skill.generator.rust

import de.ust.skill.ir._
import de.ust.skill.main.HeaderInfo

import scala.collection.JavaConverters._
import scala.collection.mutable

/**
  * Fake Main implementation required to make trait stacking work.
  */
abstract class FakeMain extends GeneralOutputMaker {
  def make {}
}

final class Main extends FakeMain
                 with SkillFileMaker
                 with PoolsMaker
                 with LibMaker
                 with PtrMaker
                 with DependenciesMaker
                 with LiteralKeeper {
  lineLength = 100

  /**
    * Tries to escape a string without decreasing the usability of the generated identifier.
    */
  private val escapeCache            = new mutable.HashMap[String, String]()
  private var _packagePrefix: String = _

  override def comment(d: Declaration): String = d.getComment
                                                 .format("", "/// ", lineLength, "")
                                                 .trim

  override def comment(f: FieldLike): String = f.getComment
                                               // NOTE 4 spaces indent
                                               .format("", "/// ", lineLength - 4, "")
                                               .trim

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

      case "string"     ⇒ "Rc<SkillString>"
      case "annotation" ⇒ "Option<Ptr<SkillObject>>"

      case _ ⇒ throw new GeneratorException(s"Unhandled type $t")
    }

    case t: ConstantLengthArrayType ⇒ s"Vec<${mapType(t.getBaseType)}>"
    case t: VariableLengthArrayType ⇒ s"Vec<${mapType(t.getBaseType)}>"
    case t: ListType                ⇒ s"LinkedList<${mapType(t.getBaseType)}>"
    case t: SetType                 ⇒ s"HashSet<${mapType(t.getBaseType)}>"
    case t: MapType                 ⇒ t.getBaseTypes.asScala.map(mapType).reduceRight((k, v) ⇒ s"HashMap<$k, $v>")

    case t: UserType ⇒ s"Option<Ptr<${traitName(t)}>>"

    case _ ⇒ throw new GeneratorException(s"Unknown type $t")
  }

  override def makeHeader(headerInfo: HeaderInfo): String = headerInfo
                                                            .format(this, "/*", "*\\", " *", "* ", "\\*", "*/")

  override def setPackage(names: List[String]) {
    _packagePrefix = names.foldRight("")(_ + "." + _)
  }

  override def setOption(option: String, value: String) {
    option match {
      case "interfacechecks" ⇒ interfaceChecks = "true".equals(value)
      case unknown           ⇒ throw new GeneratorException(s"unknown Argument: $unknown")
    }
  }

  override def helpText: String =
    """
      §interfaceChecks   true/false  if set to true, the generated API will contain is[[interface]] methods
      §""".stripMargin('§')

  override def customFieldManual: String =
    """
      §!include string+    Argument strings are added to the head of the generated file and included using
      §                    <> around the strings content.""".stripMargin('§')

  override def escaped(target: String): String = escapeCache.getOrElse(target, {
    val result = EscapeFunction(target)
    escapeCache(target) = result
    result
  })

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
        case "string"                             ⇒ "Rc::default()"
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

  protected def filterInterfacesFromIR() {
    // find implementers
    val ts = types.removeTypedefs()
    for (t ← ts.getUsertypes.asScala) {
      val is: mutable.HashSet[InterfaceType] = t.getSuperInterfaces.asScala
                                               .flatMap(recursiveSuperInterfaces(_, new mutable.HashSet[InterfaceType]))
                                               .to
      interfaceCheckImplementations(t.getSkillName) = is.map(insertInterface(_, t))
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
    for (s ← i.getSuperInterfaces.asScala) {
      recursiveSuperInterfaces(s, r)
    }
    r
  }
}

object EscapeFunction {
  def apply(target: String): String = target match {
    // keywords get a suffix "Z_" -- just "_" doesn't work in rust as that will cause a warning
    // for bad snake case -- because that way at least auto-completion will work almost as expected
    case
      // Used throughout the generator
      "Ptr" | "Rc" | "RefCell"
      // Prelude
      // https://doc.rust-lang.org/std/prelude/
      | "AsMut" | "AsRef" | "Box" | "Clone" | "Copy" | "Default" | "DoubleEndedIterator" | "Drop"
      | "Eq" | "ExactSizeIterator" | "Extend" | "Fn" | "FnMut" | "FnOnce" | "From" | "Into"
      | "IntoIterator" | "Iterator" | "Option" | "Ord" | "PartialEq" | "PartialOrd" | "Result"
      | "Send" | "Sized" | "SliceConcatExt" | "String" | "Sync" | "ToOwned" | "ToString" | "Vec"
      | "clone" | "drop"
      // Data types
      // https://doc.rust-lang.org/book/second-edition/ch03-02-data-types.html
      | "bool" | "char" | "f32" | "f64" | "i16" | "i32" | "i64" | "i8" | "isize" | "str" | "u16"
      | "u32" | "u64" | "u8" | "usize"
      // Keywords + reserved
      // https://doc.rust-lang.org/beta/reference/keywords.html
      // https://doc.rust-lang.org/book/second-edition/appendix-01-keywords.html
      | "Self" | "abstract" | "alignof" | "as" | "become" | "box" | "break" | "const" | "continue"
      | "crate" | "do" | "else" | "enum" | "extern" | "false" | "final" | "fn" | "for" | "if"
      | "impl" | "in" | "let" | "loop" | "macro" | "match" | "mod" | "move" | "mut" | "offsetof"
      | "override" | "priv" | "proc" | "pub" | "pure" | "ref" | "return" | "self" | "sizeof"
      | "static" | "struct" | "super" | "trait" | "true" | "type" | "typeof" | "unsafe" | "unsized"
      | "use" | "virtual" | "where" | "while" | "yield"  ⇒ s"Z_$target"
    case t if t.forall(c ⇒ Character.isLetterOrDigit(c)) ⇒ t
    case _                                               ⇒ target.map {
      case 'Z'           ⇒ "ZZ"
      case c if '_' == c ⇒ "Z" + c
      case c             ⇒ f"Z$c%04X"
    }.mkString
  }
}
