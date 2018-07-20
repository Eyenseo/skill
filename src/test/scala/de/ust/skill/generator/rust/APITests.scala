/*  ___ _  ___ _ _                                                            *\
** / __| |/ (_) | |       The SKilL Generator                                 **
** \__ \ ' <| | | |__     (c) 2013-18 University of Stuttgart                 **
** |___/_|\_\_|_|____|    see LICENSE                                         **
\*                                                                            */
package de.ust.skill.generator.rust

import java.io._

import de.ust.skill.generator.common
import de.ust.skill.generator.common.Indenter._
import de.ust.skill.ir._
import de.ust.skill.main.CommandLine
import org.json.JSONObject
import org.junit.runner.RunWith
import org.scalatest.junit.JUnitRunner

import scala.collection.JavaConverters._

@RunWith(classOf[JUnitRunner])
class APITests extends common.GenericAPITests {

  override val language = "rust"

  var gen = new Main

  var generatedTests = new Array[String](0)

  val skipGeneration = Array(
                              // "age",
                              // "floats",
                              // "number",
                              // "unicode",

                              "annotation",
                              "auto", // TODO - is this really successful?
                              "basicTypes",
                              "constants",
                              "container",
                              "custom",
                              "empty",
                              "enums", // TODO - is this really successful?
                              "escaping",
                              "fancy",
                              "graph",
                              "graphInterface", // TODO - is this really successful?
                              "hintsAll", // TODO - is this really successful?
                              "map3",
                              "restrictionsAll", // TODO - is this really successful?
                              "restrictionsCore", // TODO - is this really successful?
                              "subtypes",
                              "unknown",
                              "user",
                              "",
                            )

  override def deleteOutDir(out: String): Unit = {
    import scala.reflect.io.Directory

    val pkgEsc = escSnakeCase(out.split("/").map(EscapeFunction.apply).mkString("_"))
    Directory(new File("testsuites/rust/", out)).deleteRecursively
  }

  override def callMainFor(name: String, source: String, options: Seq[String]) {
    if (skipGeneration.contains(name)) {
      println("API Skip: " + name)
      return
    }
    val pkgEsc = escSnakeCase(name.split("/").map(EscapeFunction.apply).mkString("_"))

    CommandLine.main(Array[String](source,
                                    "--debug-header",
                                    "-c",
                                    "-L", "rust",
                                    "-p", name,
                                    "-o", "testsuites/rust/" + pkgEsc) ++ options)
  }

  override def finalizeTests() {
    val pw = new PrintWriter(new File("testsuites/rust/Cargo.toml"))
    // FIXME hardcoded path
    pw.write(
              """[workspace]
                |members = [""".stripMargin
            )
    for (test ← generatedTests) {
      pw.write(
                e""""$test", """.stripMargin
              )
    }
    pw.write("]")
    pw.close()
  }

  def snakeCase(str: String): String = GeneralOutputMaker.snakeCase(str)

  def escSnakeCase(str: String): String = snakeCase(EscapeFunction.apply(str))

  override def newTestFile(packagePath: String, name: String): PrintWriter = {
    if (skipGeneration.contains(packagePath)) {
      return null
    }

    gen = new Main
    gen.setPackage(List(packagePath))

    val pkgEsc = escSnakeCase(packagePath.split("/").map(EscapeFunction.apply).mkString("_"))

    generatedTests :+= pkgEsc

    val f = if (name.toLowerCase.equals("api")) {
      new File(s"testsuites/rust/$pkgEsc/tests/api.rs")
    } else {
      new File(s"testsuites/rust/$pkgEsc/tests/api_${escSnakeCase(name)}.rs")
    }


    f.getParentFile.mkdirs
    if (f.exists) {
      // TODO is this ... "ok"
      f.delete
    }
    f.createNewFile

    val rval = new PrintWriter(new BufferedWriter(new OutputStreamWriter(new FileOutputStream(f), "UTF-8")))
    rval.write(
                e"""#![feature(test)]
                   |
                   |extern crate $pkgEsc;
                   |
                   |#[cfg(test)]
                   |#[allow(non_snake_case)]
                   |#[allow(unused_must_use)]
                   |mod tests {
                   |    extern crate env_logger;
                   |
                   |    use $pkgEsc::common::SkillFile as SkillFileTrait;
                   |
                   |    use $pkgEsc::skill_file::SkillFile;""".stripMargin
              )
    rval
  }

  override def closeTestFile(out: java.io.PrintWriter) {
    out.write(
               """
                 |}
               """.stripMargin)
    out.close()
  }

  override def makeSkipTest(out: PrintWriter, kind: String, name: String, testName: String, accept: Boolean) {
    throw new GeneratorException("SKIP is not implemented for Rust")
  }

  override def makeRegularTest(out: PrintWriter, kind: String, name: String, testName: String, accept: Boolean,
                               IR: TypeContext, root: JSONObject) {
    val tc = IR.removeSpecialDeclarations()
    val uuid = java.util.UUID.randomUUID.toString
    val funName = e"api_${escSnakeCase(name)}_${if (accept) "accept" else "reject"}_${
      escSnakeCase(testName.replaceAll("_|-", ""))
    }"

    out.write(
               // FIXME hardcoded tmp file
               e"""
                  |
                  |    #[test]${if (!accept) "\n#[should_panic]" else ""}
                  |    fn $funName() {
                  |        match SkillFile::create("/tmp/${funName}_$uuid.sf") {
                  |            Ok(sf) => match sf.check() {
                  |                Ok(_) => {
                  |                    // create objects
                  |                    ${createObjects(root, tc, name)}
                  |                    // set fields
                  |                    ${setFields(root, tc)}
                  |
                  |                    sf.close();
                  |                },
                  |                Err(e) => panic!("{}", e)
                  |            },
                  |            Err(e) => panic!("{}", e),
                  |        };
                  |
                  |        match SkillFile::open("/tmp/${funName}_$uuid.sf") {
                  |            Ok(sf) => match sf.check() {
                  |                Ok(_) => {
                  |                    // get objects
                  |                    ${readObjects(root, tc, name)}
                  |                    // assert fields
                  |                    ${assertFields(root, tc)}
                  |                },
                  |                Err(e) => panic!("{}", e)
                  |            },
                  |            Err(e) => panic!("{}", e),
                  |        };
                  |    }""".stripMargin)
    // TODO add writing, reading and verifying results
  }

  private def getType(tc: TypeContext, name: String) = {
    val n = name.toLowerCase()

    try {
      (tc.getUsertypes.asScala ++ tc.getInterfaces.asScala).find(_.getSkillName.equals(n)).get
    } catch {
      case e: NoSuchElementException ⇒ fail(s"Type '$n' does not exist, fix your test description!")
    }
  }

  private def getField(tc: TypeContext, typ: String, field: String) = {
    val t = getType(tc, typ)
    val fn = field.toLowerCase()
    try {
      t.getAllFields.asScala.find(_.getSkillName.equals(fn)).get
    } catch {
      case e: NoSuchElementException ⇒ fail(s"Field '$fn' does not exist, fix your test description!")
    }
  }

  private def getPoolName(tc: TypeContext, name: String): String = {
    val n = name.toLowerCase()
    try {
      snakeCase(
                 gen.escaped(
                              (tc.getUsertypes.asScala ++ tc.getInterfaces.asScala)
                                .filter(_.getSkillName.equals(n))
                                .head
                                .getName
                                .camel()
                            )
               )
    } catch {
      case e: NoSuchElementException ⇒ fail(s"Type '$n' does not exist, fix your test description!")
    }
  }

  private def value(v: Any, f: Field): String = value(v, f.getType)

  private def value(v: Any, t: Type): String = t match {
    case t: GroundType ⇒
      t.getSkillName match {
        case "i8"          ⇒ v.toString + " as i8"
        case "u8"          ⇒ v.toString + " as u8"
        case "i16"         ⇒ v.toString + " as i16"
        case "u16"         ⇒ v.toString + " as u16"
        case "i32"         ⇒ v.toString + " as i32"
        case "u32"         ⇒ v.toString + " as u32"
        case "u64"         ⇒ v.toString + " as u64"
        case "v64" | "i64" ⇒ v.toString + " as i64"
        case "f32"         ⇒ v.toString + " as f32"
        case "f64"         ⇒ v.toString + " as f64"

        case "string" if null != v ⇒
          s"""sf.strings.add("${v.toString}")"""

        case _ ⇒
          if (null == v || v.toString.equals("null")) {
            "None"
          } else {
            // v.toString
            throw new GeneratorException("to be implemented")
          }
      }

    // TODO container
    case t: SingleBaseTypeContainer ⇒
      throw new GeneratorException("to be implemented")
    // locally {
    //   var rval = t match {
    //     case t: SetType ⇒ s"set<${gen.mapType(t.getBaseType)}>()"
    //     case _          ⇒ s"array<${gen.mapType(t.getBaseType)}>()"
    //   }
    //   for (x ← v.asInstanceOf[JSONArray].iterator().asScala) {
    //     rval = s"put<${gen.mapType(t.getBaseType)}>($rval, ${value(x, t.getBaseType)})"
    //   }
    //   rval
    // }
    // TODO container map
    case t: MapType if v != null ⇒
      throw new GeneratorException("to be implemented")
    // valueMap(v.asInstanceOf[JSONObject], t.getBaseTypes.asScala.toList)

    // TODO user types
    case t: UserType ⇒
      throw new GeneratorException("to be implemented")
    // if (null == v || v.toString().equals("null")) {
    //   "std::ptr::null"
    // } // TODO check
    // else {
    //   v.toString
    // }
    case _ ⇒
      throw new GeneratorException("Unknown Type")
  }

  private def valueMap(v: Any, ts: List[Type]): String = {
    if (1 == ts.length) {
      value(v, ts.head)
    } else {
      var rval = s"map<${gen.mapType(ts.head)}, ${
        ts.tail match {
          case t if t.size >= 2 ⇒ t.map(gen.mapType).reduceRight((k, v) ⇒ s"::skill::api::Map<$k, $v>*")
          case t                ⇒ gen.mapType(t.head)
        }
      }>()"
      val root = v.asInstanceOf[JSONObject]

      for (name ← JSONObject.getNames(root)) {
        rval = s"put<${gen.mapType(ts.head)}, ${
          ts.tail match {
            case t if t.size >= 2 ⇒ t.map(gen.mapType).reduceRight((k, v) ⇒ s"::skill::api::Map<$k, $v>*")
            case t                ⇒ gen.mapType(t.head)
          }
        }>($rval, ${value(name, ts.head)}, ${valueMap(root.get(name), ts.tail)})"
      }

      rval
    }
  }

  private def createObjects(root: JSONObject, tc: TypeContext, packagePath: String): String = {
    if (null == JSONObject.getNames(root)) {
      ""
    } else {
      (for (name ← JSONObject.getNames(root)) yield {
        val obj = root.getJSONObject(name)
        val objType = getType(tc, JSONObject.getNames(obj).head)
        val pool = snakeCase(gen.escaped(objType.getName.camel()))

        e"""let $name = sf.$pool.add();
           |""".stripMargin
      }).mkString
    }
  }.trim

  private def readObjects(root: JSONObject, tc: TypeContext, packagePath: String): String = {
    if (null == JSONObject.getNames(root)) {
      ""
    } else {
      (for ((name, i) ← JSONObject.getNames(root).zipWithIndex) yield {
        val obj = root.getJSONObject(name)
        val objType = getType(tc, JSONObject.getNames(obj).head)
        val pool = snakeCase(gen.escaped(objType.getName.camel()))

        e"""let $name = sf.$pool.get(${i + 1});
           |""".stripMargin
      }).mkString
    }
  }.trim

  private def setFields(root: JSONObject, tc: TypeContext): String = {
    if (null == JSONObject.getNames(root)) {
      ""
    } else {
      (for (name ← JSONObject.getNames(root)) yield {
        val obj = root.getJSONObject(name)
        val objTypeName = JSONObject.getNames(obj).head
        val objFieldNames = obj.getJSONObject(objTypeName)

        if (null == JSONObject.getNames(objFieldNames)) {
          ""
        } else {
          (for (fieldName ← JSONObject.getNames(objFieldNames).toSeq) yield {
            val field = getField(tc, objTypeName, fieldName)
            val setter = "set_" + snakeCase(gen.escaped(field.getName.camel()))

            e"""$name.borrow_mut().$setter(${value(objFieldNames.get(fieldName), field)});
               |""".stripMargin
          }).mkString
        }
      }).mkString
    }
  }.trim

  private def assertFields(root: JSONObject, tc: TypeContext): String = {
    if (null == JSONObject.getNames(root)) {
      ""
    } else {
      (for (name ← JSONObject.getNames(root)) yield {
        val obj = root.getJSONObject(name)
        val objTypeName = JSONObject.getNames(obj).head
        val objFieldNames = obj.getJSONObject(objTypeName)

        if (null == JSONObject.getNames(objFieldNames)) {
          ""
        } else {
          (for (fieldName ← JSONObject.getNames(objFieldNames).toSeq) yield {
            val field = getField(tc, objTypeName, fieldName)
            val getter = "get_" + snakeCase(gen.escaped(field.getName.camel()))

            e"""assert_eq!($name.borrow_mut().$getter(), ${value(objFieldNames.get(fieldName), field)});
               |""".stripMargin
          }).mkString
        }
      }).mkString
    }
  }.trim
}
