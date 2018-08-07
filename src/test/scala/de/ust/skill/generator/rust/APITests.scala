/*  ___ _  ___ _ _                                                            *\
** / __| |/ (_) | |       The SKilL Generator                                 **
** \__ \ ' <| | | |__     (c) 2013-18 University of Stuttgart                 **
** |___/_|\_\_|_|____|    see LICENSE                                         **
\*                                                                            */
package de.ust.skill.generator.rust

import java.io._
import java.nio.file.Files

import de.ust.skill.generator.common
import de.ust.skill.generator.common.IndenterLaw._
import de.ust.skill.ir._
import de.ust.skill.main.CommandLine
import org.json.{JSONArray, JSONObject}
import org.junit.runner.RunWith
import org.scalatest.junit.JUnitRunner

import scala.collection.JavaConverters._

@RunWith(classOf[JUnitRunner])
class APITests extends common.GenericAPITests {

  override val language = "rust"

  var gen = new Main

  var generatedTests = new Array[String](0)

  class ExtraTest(test: File, deps: Array[String]) {
    def dependencies(): Array[String] = {
      deps
    }

    def file(): File = {
      test
    }
  }

  // NOTE cyclic dependencies are not allowed
  var extraTests = Array(
                          new ExtraTest(
                                         new File("deps/rust/tests/undefined_read_write_read.rs"),
                                         Array(
                                                "unknown",
                                              )
                                       ),
                          new ExtraTest(
                                         new File("deps/rust/tests/subtypes_undefined_subtypes.rs"),
                                         Array(
                                                "unknown",
                                                "subtypes",
                                              )
                                       ),
                        )

  var skipTestCases  = Array(
                              "restr", // FIXME restrictions are not implemented
                              "fail_long_array", // NOTE this would fail to compile!
                              "fail_short_array", // NOTE this would fail to compile!
                              "polyFail", // NOTE this would fail to compile!
                              "poly_fail_1", // NOTE this would fail to compile!
                              "poly_fail_2", // NOTE this would fail to compile!
                              "boolean", // NOTE this would fail to compile!
                              "bool", // NOTE this would fail to compile!
                            )
  val skipGeneration = Array(
                              // "age",
                              // "annotation",
                              // "auto",
                              // "basicTypes",
                              // "container",
                              // "custom", // TODO -  is this really successful?
                              // "empty",
                              // "escaping",
                              // "fancy",
                              // "floats",
                              // "graph",
                              // "graphInterface",
                              // "hintsAll", // TODO -  is this really successful?
                              // "map3",
                              // "number",
                              // "subtypes",
                              // "unicode",
                              // "user",
                              // "unknown", // NOTE in this test there happens nothing "unknown"

                              // FIXME currently a setter is generated that is needed for the reader thus constant fields are not constant
                              // FIXME the test pass so the generator has to be adjusted
                              "constants",
                              "enums", // FIXME test fail
                              "restrictionsAll", // FIXME broken generation
                              "restrictionsCore", // FIXME broken generation
                              "",
                            )

  override def deleteOutDir(out: String): Unit = {
    import scala.reflect.io.Directory

    val pkgEsc = escSnakeCase(out.split("/").map(EscapeFunction.apply).mkString("_"))
    Directory(new File(s"testsuites/rust/$pkgEsc", out)).deleteRecursively
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
    pw.write(
              """[workspace]
                §members = [""".stripMargin('§')
            )
    for (test ← generatedTests) {
      pw.write(
                e""""$test", """.stripMargin('§')
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

    for (
      test ← extraTests.filter(t ⇒ t.dependencies().toList.head.equals(packagePath))) {
      val out = new File(s"testsuites/rust/$pkgEsc/tests/${test.file().getName}")

      if (out.exists()) {
        out.delete()
      }
      out.getParentFile.mkdirs()

      if (test.dependencies().forall(d ⇒ !skipGeneration.contains(d))) {
        Files.copy(test.file().toPath, out.toPath)

        val fw = new FileWriter(s"testsuites/rust/$pkgEsc/Cargo.toml", true)
        try {
          for (d ← test.dependencies().filterNot(s ⇒ s.equals(packagePath))) {
            fw.write(
                      e"""skill_$d = { path = "../$d" }
                         §""".stripMargin('§'))
          }
        } finally {
          fw.close()
        }
      }
    }

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
                   §#![feature(nll)]
                   §
                   §extern crate $pkgEsc;
                   §
                   §#[cfg(test)]
                   §#[allow(non_snake_case)]
                   §#[allow(unused_imports)]
                   §#[allow(unused_variables)]
                   §mod tests {
                   §    extern crate env_logger;
                   §    extern crate failure;
                   §
                   §    use $pkgEsc::common::*;
                   §    use $pkgEsc::common::error::*;
                   §    use $pkgEsc::common::internal::SkillObject;
                   §    use $pkgEsc::skill_file::*;
                   §    use $pkgEsc::*;
                   §
                   §    use self::failure::Fail;
                   §
                   §    use std::collections::HashSet;
                   §    use std::collections::HashMap;
                   §    use std::collections::LinkedList;
                   §    use std::rc::Rc;
                   §""".stripMargin('§').trim
              )
    rval
  }

  override def closeTestFile(out: java.io.PrintWriter) {
    out.write(
               """
                 §}
                 §""".stripMargin('§'))
    out.close()
  }

  override def makeSkipTest(out: PrintWriter, kind: String, name: String, testName: String, accept: Boolean) {
    throw new GeneratorException("SKIP is not implemented for Rust")
  }

  override def makeRegularTest(out: PrintWriter, kind: String, name: String, testName: String, accept: Boolean,
                               IR: TypeContext, root: JSONObject) {
    if (skipTestCases.contains(testName.toLowerCase)) {
      return
    }

    val tc = IR.removeSpecialDeclarations()
    val uuid = java.util.UUID.randomUUID.toString
    val funName = e"api_${escSnakeCase(name)}_${if (accept) "accept" else "reject"}_${
      escSnakeCase(testName.replaceAll("_|-", ""))
    }"

    out.write(
               // FIXME hardcoded tmp file
               e"""
                  §    struct Cleanup${gen.camelCase(funName.capitalize)};
                  §
                  §    impl Drop for Cleanup${gen.camelCase(funName.capitalize)} {
                  §        fn drop(&mut self) {
                  §            let _ignore = ::std::fs::remove_file("/tmp/${funName}_$uuid.sf");
                  §        }
                  §    }
                  §
                  §    #[test]${if (!accept) "\n#[should_panic]" else ""}
                  §    fn $funName() {
                  §        let _logger = env_logger::try_init();
                  §        let _cleanup = Cleanup${gen.camelCase(funName.capitalize)};
                  §
                  §         ${objectIDs(root, tc)}
                  §
                  §        match SkillFile::create("/tmp/${funName}_$uuid.sf") {
                  §            Ok(sf) => match || -> Result<(), SkillFail> {
                  §                sf.check()?;
                  §                // create objects
                  §                ${createObjects(root, tc, name)}
                  §                // set fields
                  §                ${setFields(root, tc)}
                  §                // serialize
                  §                sf.close()?;
                  §                // remember object IDs - type hierarchy makes them difficult to calculate for the generator
                  §                ${rememberObjectIDs(root, tc)}
                  §                Ok(())
                  §            }() {
                  §                Ok(_) => {},
                  §                Err(e) => if let Some(bt) = e.backtrace() {
                  §                    panic!("{}\n{}", e, bt)
                  §                } else {
                  §                    panic!("{}", e)
                  §                }
                  §            },
                  §            Err(e) => if let Some(bt) = e.backtrace() {
                  §                panic!("{}\n{}", e, bt)
                  §            } else {
                  §                panic!("{}", e)
                  §            },
                  §        };
                  §
                  §        match SkillFile::open("/tmp/${funName}_$uuid.sf") {
                  §            Ok(sf) => match sf.check() {
                  §                Ok(_) => {
                  §                    // get objects
                  §                    ${readObjects(root, tc, name)}
                  §                    // assert fields
                  §                    ${assertFields(root, tc)}
                  §                },
                  §                Err(e) => if let Some(bt) = e.backtrace() {
                  §                    panic!("{}\n{}", e, bt)
                  §                } else {
                  §                    panic!("{}", e)
                  §                }
                  §            },
                  §            Err(e) => if let Some(bt) = e.backtrace() {
                  §                panic!("{}\n{}", e, bt)
                  §            } else {
                  §                panic!("{}", e)
                  §            },
                  §        };
                  §    }""".stripMargin('§'))
  }

  private def getType(tc: TypeContext, name: String) = {
    val n = name.toLowerCase()

    try {
      (tc.getUsertypes.asScala ++ tc.getInterfaces.asScala).find(e ⇒ e.getSkillName.equals(n)).get
    } catch {
      case _: NoSuchElementException ⇒ fail(s"Type '$n' does not exist, fix your test description!")
    }
  }

  private def getField(tc: TypeContext, typ: String, field: String) = {
    val t = getType(tc, typ)
    val fn = field.toLowerCase()
    try {
      t.getAllFields.asScala.find(e ⇒ e.getSkillName.equals(fn)).get
    } catch {
      case _: NoSuchElementException ⇒ fail(s"Field '$fn' does not exist, fix your test description!")
    }
  }

  private def value(v: Any, f: Field): String = value(v, f.getType)

  private def value(v: Any, t: Type): String = t match {
    case t: GroundType              ⇒
      t.getSkillName match {
        case "bool"        ⇒ v.toString
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
          // NOTE currently NLL doesn't fix this mess
          s"""{
             §    let mut sp = sf.strings.borrow_mut();
             §    let s = sp.add("${v.toString}");
             §    s
             §}""".stripMargin('§')

        case _ ⇒
          if (null == v || v.toString.equals("null")) {
            "None"
          } else {
            // NOTE all objects are read back so these names have to be valid
            // NOTE unwrapping is done to trigger a panic in case the cast ist illegal
            e"Some(${v.toString}.clone().nucast::<SkillObject>().unwrap())"
          }
      }
    case t: ConstantLengthArrayType ⇒
      e"""{
         §    let mut arr: ${gen.mapType(t)} = ${gen.defaultValue(t)};
         §    ${
        (for ((x, i) ← v.asInstanceOf[JSONArray].iterator().asScala.zipWithIndex) yield {
          e"""arr[$i] = ${value(x, t.getBaseType)};
             §""".stripMargin('§')
        }).mkString.trim
      }
         §    arr
         §}""".stripMargin('§')
    case t: VariableLengthArrayType ⇒
      e"""{
         §    let mut vec: ${gen.mapType(t)} = ${gen.defaultValue(t)};
         §    vec.reserve(${v.asInstanceOf[JSONArray].length()});
         §    ${
        (for (x ← v.asInstanceOf[JSONArray].iterator().asScala) yield {
          e"""vec.push(${value(x, t.getBaseType)});
             §""".stripMargin('§')
        }).mkString.trim
      }
         §    vec
         §}""".stripMargin('§')
    case t: SetType                 ⇒
      e"""{
         §    let mut set: ${gen.mapType(t)} = ${gen.defaultValue(t)};
         §    set.reserve(${v.asInstanceOf[JSONArray].length()});
         §    ${
        (for (x ← v.asInstanceOf[JSONArray].iterator().asScala) yield {
          e"""set.insert(${value(x, t.getBaseType)});
             §""".stripMargin('§')
        }).mkString.trim
      }
         §    set
         §}""".stripMargin('§')

    case t: ListType             ⇒
      e"""{
         §    let mut list: ${gen.mapType(t)} = ${gen.defaultValue(t)};
         §    ${
        (for (x ← v.asInstanceOf[JSONArray].iterator().asScala) yield {
          e"""list.push_back(${value(x, t.getBaseType)});
             §""".stripMargin('§')
        }).mkString.trim
      }
         §    list
         §}""".stripMargin('§')
    case t: MapType if v != null ⇒
      valueMap(v.asInstanceOf[JSONObject], t.getBaseTypes.asScala.toList)
    case _: UserType             ⇒
      if (null == v || v.toString.equals("null")) {
        "None"
      } else {
        // NOTE all objects are read back so these names have to be valid
        // NOTE unwrapping is done to trigger a panic in case the cast ist illegal
        e"Some(${v.toString}.clone().nucast::<${gen.traitName(t)}>().unwrap())"
      }
    case _                       ⇒
      throw new GeneratorException("Unknown Type")
  }

  private def valueMap(v: Any, tts: List[Type]): String = {
    val (key, remainder) = tts.splitAt(1)

    if (remainder.size > 1) {
      e"""{
         §    let mut map: ${gen.mapMapTypes(tts)} = HashMap::default();
         §    map.reserve(${v.asInstanceOf[JSONObject].length()});
         §    ${
        val root = v.asInstanceOf[JSONObject]
        (for (name ← JSONObject.getNames(root)) yield {
          e"""map.insert(${value(name, key.head)}, ${valueMap(root.get(name), remainder)});
             §""".stripMargin('§')
        }).mkString.trim
      }
         §    map
         §}""".stripMargin('§')
    } else {
      e"""{
         §    let mut map: ${gen.mapMapTypes(tts)} = HashMap::default();
         §    map.reserve(${v.asInstanceOf[JSONObject].length()});
         §    ${
        val root = v.asInstanceOf[JSONObject]
        (for (name ← JSONObject.getNames(root)) yield {
          e"""map.insert(${value(name, key.head)}, ${value(root.get(name), remainder.head)});
             §""".stripMargin('§')
        }).mkString.trim
      }
         §    map
         §}""".stripMargin('§')
    }
  }

  private def objectIDs(root: JSONObject, tc: TypeContext): String = {
    if (null == JSONObject.getNames(root)) {
      ""
    } else {
      (for (name ← JSONObject.getNames(root)) yield {
        e"""let mut ${name}_id = 0;
           §""".stripMargin('§')
      }).mkString
    }
  }.trim

  private def rememberObjectIDs(root: JSONObject, tc: TypeContext): String = {
    if (null == JSONObject.getNames(root)) {
      ""
    } else {
      (for (name ← JSONObject.getNames(root)) yield {
        e"""${name}_id = $name.borrow().get_skill_id();
           §""".stripMargin('§')
      }).mkString
    }
  }.trim


  private def createObjects(root: JSONObject,
                            tc: TypeContext,
                            packagePath: String): String = {
    if (null == JSONObject.getNames(root)) {
      ""
    } else {
      (for (name ← JSONObject.getNames(root)) yield {
        val obj = root.getJSONObject(name)
        val objType = getType(tc, JSONObject.getNames(obj).head)
        val pool = snakeCase(gen.escaped(objType.getName.camel()))

        e"""let $name = sf.$pool.borrow_mut().add();
           §""".stripMargin('§')
      }).mkString
    }
  }.trim

  private def readObjects(root: JSONObject, tc: TypeContext, packagePath: String): String = {
    if (null == JSONObject.getNames(root)) {
      ""
    } else {
      (for (name ← JSONObject.getNames(root)) yield {
        val obj = root.getJSONObject(name)
        val objType = getType(tc, JSONObject.getNames(obj).head)
        val pool = snakeCase(gen.escaped(objType.getName.camel()))

        e"""let $name = match sf.$pool.borrow().get(${name}_id) {
           §    Ok(ptr) => ptr,
           §    Err(e) => panic!("Object $name was not retrieved because:{}", e),
           §};
           §""".stripMargin('§')
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
               §""".stripMargin('§')
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

            // In case the field is an auto field we expect the default value
            def value(v: Any, f: Field): String = {
              if (f.isAuto) {
                gen.defaultValue(f.getType)
              } else {
                this.value(v, f.getType)
              }
            }

            def ptrUser(): String = {
              val expected = value(objFieldNames.get(fieldName), field)
              if (expected.equals("None")) {
                e"""assert_eq!($name.borrow_mut().$getter().is_none(), true);
                   §""".stripMargin('§')
              } else {
                e"""assert_eq!(
                   §    $name.borrow_mut().$getter().is_some(), true);
                   §assert_eq!(
                   §    $name.borrow_mut().$getter().as_ref().unwrap().nucast::<SkillObject>(),
                   §    $expected.unwrap().nucast::<SkillObject>(),
                   §);
                   §""".stripMargin('§')
              }
            }

            field.getType match {
              case _: UserType                                             ⇒
                ptrUser()
              case t: GroundType if t.getName.camel().equals("annotation") ⇒
                ptrUser()
              case _: ReferenceType                                        ⇒
                e"""assert_eq!(*$name.borrow_mut().$getter(), ${value(objFieldNames.get(fieldName), field)});
                   §""".stripMargin('§')
              case _: GroundType                                           ⇒
                e"""assert_eq!($name.borrow_mut().$getter(), ${value(objFieldNames.get(fieldName), field)});
                   §""".stripMargin('§')
              case _                                                       ⇒
                e"""assert_eq!(*$name.borrow_mut().$getter(), ${value(objFieldNames.get(fieldName), field)});
                   §""".stripMargin('§')
            }
          }).mkString
        }
      }).mkString
    }
  }.trim
}
