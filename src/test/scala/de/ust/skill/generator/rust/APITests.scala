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
import org.json.{JSONArray, JSONObject}
import org.junit.runner.RunWith
import org.scalatest.junit.JUnitRunner

import scala.collection.JavaConverters._

@RunWith(classOf[JUnitRunner])
class APITests extends common.GenericAPITests {

  override val language = "rust"

  var gen = new Main

  class Test(val name: String, val location: String) {}

  var generatedTests = new Array[Test](0)

  override def deleteOutDir(out: String): Unit = {
    // TODO is this the intended behaviour? Why are the folders shared?
  }

  override def callMainFor(name: String, source: String, options: Seq[String]) {
    // TODO remove / extend
    if (!name.equals("age") && !name.equals("number")) {
      println("mAPI Skip: " + name)
      return
    }

    CommandLine.main(Array[String](source,
                                   "--debug-header",
                                   "-c",
                                   "-L", "rust",
                                   "-p", name,
                                   "-o", "testsuites/rust/" + name) ++ options)
  }

  override def finalizeTests {
    val pw = new PrintWriter(new File("testsuites/rust/Cargo.toml"))
    // FIXME hardcoded path
    // TODO move to separate file and share with other tests
    pw.write(
              """[workspace]
                |members = [""".stripMargin
            )
    for (test: Test <- generatedTests) {
      pw.write(e""""${test.name}", """.stripMargin
              )
    }
    pw.write("]")
    pw.close()
  }

  override def newTestFile(packagePath: String, name: String): PrintWriter = {
    val packageName = packagePath.split("/").map(EscapeFunction.apply).mkString("::")
    gen = new Main
    gen.setPackage(List(packagePath))

    generatedTests :+= new Test(packagePath, s"$packagePath/tests/generic${name}Test.rs")

    val f = new File(s"testsuites/rust/$packagePath/tests/generic${name}Test.rs")
    f.getParentFile.mkdirs
    if (f.exists)
      f.delete
    f.createNewFile

    val rval = new PrintWriter(new BufferedWriter(new OutputStreamWriter(new FileOutputStream(f), "UTF-8")))
    rval.write(
                e"""#![feature(test)]
                   |
                  |extern crate skill_common;
                   |extern crate skill_tests;
                   |
                  |#[cfg(test)]
                   |#[allow(non_snake_case)]
                  |#[allow(unused_must_use)]
                  |mod tests {
                  |    use skill_common::SkillFile as SkillFileTrait;
                  |    use skill_tests::$packageName::skill_file::SkillFile;""".stripMargin)
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
    // TODO skip tests
    out.write(
               s"""
TEST(${name.capitalize}_APITest, ${gen.escaped(kind)}_skipped_${gen.escaped(testName)}) {${
                 if (accept) ""
                 else
                   """
    GTEST_FAIL() << "The test was skipped by the test generator.";"""
               }
}
""")
  }

  override def makeRegularTest(out: PrintWriter, kind: String, name: String, testName: String, accept: Boolean,
                               IR: TypeContext, obj: JSONObject) {
    val tc = IR.removeSpecialDeclarations()
    out.write(
               // FIXME hardcoded tmp file
               e"""
                  |
                  |    #[test]${if (!accept) "\n#[should_panic]" else ""}
                  |    fn ${name.capitalize}_APITest_${if (accept) "Acc" else "Fail"}_${gen.escaped(testName)}() {
                  |        let mut sf = SkillFile::create("/tmp/foo.sf");
                  |
                  |        // create objects
                  |        ${createObjects(obj, tc, name)}
                  |        // set fields
                  |        ${setFields(obj, tc)}
                  |
                  |        sf.close();
                  |    }""".stripMargin)
  }

  private def typ(tc: TypeContext, name: String): String = {
    val n = name.toLowerCase()
    try {
      gen.escaped((tc.getUsertypes.asScala ++ tc.getInterfaces.asScala).filter(_.getSkillName.equals(n)).head.getName
                                                                       .capital())
    } catch {
      case e: NoSuchElementException ⇒ fail(s"Type '$n' does not exist, fix your test description!")
    }
  }

  private def field(tc: TypeContext, typ: String, field: String) = {
    val tn = typ.toLowerCase()
    val t = tc.getUsertypes.asScala.find(_.getSkillName.equals(tn)).get
    val fn = field.toLowerCase()
    try {
      t.getAllFields.asScala.find(_.getSkillName.equals(fn)).get
    } catch {
      case e: NoSuchElementException ⇒ fail(s"Field '$fn' does not exist, fix your test description!")
    }
  }

  private def value(v: Any, f: Field): String = value(v, f.getType)

  private def value(v: Any, t: Type): String = t match {
    case t: GroundType ⇒
      t.getSkillName match {
        case "string" if null != v ⇒ s"""sf.strings.add("${v.toString}")"""
        case "i8"                  ⇒ v.toString + " as i8"
        case "i16"                 ⇒ v.toString + " as i16"
        case "f32"                 ⇒ v.toString + " as f32"
        case "f64"                 ⇒ v.toString + " as f64"
        case "v64" | "i64"         ⇒ v.toString + " as i64"
        case _                     ⇒
          if (null == v || v.toString.equals("null"))
            "std::ptr::null" // TODO check
          else
            v.toString
      }

    // TODO container
    case t: SingleBaseTypeContainer ⇒
      locally {
                var rval = t match {
                  case t: SetType ⇒ s"set<${gen.mapType(t.getBaseType)}>()"
                  case _          ⇒ s"array<${gen.mapType(t.getBaseType)}>()"
                }
                for (x ← v.asInstanceOf[JSONArray].iterator().asScala) {
                  rval = s"put<${gen.mapType(t.getBaseType)}>($rval, ${value(x, t.getBaseType)})"
                }
                rval
              }
    // TODO container map
    case t: MapType if v != null ⇒ valueMap(v.asInstanceOf[JSONObject], t.getBaseTypes.asScala.toList)

    case _ ⇒
      if (null == v || v.toString().equals("null"))
        "std::ptr::null" // TODO check
      else
        v.toString
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
      val obj = v.asInstanceOf[JSONObject]

      for (name ← JSONObject.getNames(obj)) {
        rval = s"put<${gen.mapType(ts.head)}, ${
          ts.tail match {
            case t if t.size >= 2 ⇒ t.map(gen.mapType).reduceRight((k, v) ⇒ s"::skill::api::Map<$k, $v>*")
            case t                ⇒ gen.mapType(t.head)
          }
        }>($rval, ${value(name, ts.head)}, ${valueMap(obj.get(name), ts.tail)})"
      }

      rval;
    }
  }

  private def createObjects(obj: JSONObject, tc: TypeContext, packagePath: String): String = {
    if (null == JSONObject.getNames(obj)) {
      ""
    } else {
      val rval = for (name ← JSONObject.getNames(obj)) yield {
        val x = obj.getJSONObject(name)
        val t = JSONObject.getNames(x).head

        val typeName = typ(tc, t)

        s"let mut $name = sf.$typeName.add();\n"
      }

      rval.mkString
    }
  }

  private def setFields(obj: JSONObject, tc: TypeContext): String = {
    if (null == JSONObject.getNames(obj)) {
      ""
    } else {

      val rval = for (name ← JSONObject.getNames(obj)) yield {
        val x = obj.getJSONObject(name)
        val t = JSONObject.getNames(x).head
        val fs = x.getJSONObject(t)

        if (null == JSONObject.getNames(fs))
          ""
        else {
          val assignments = for (fieldName ← JSONObject.getNames(fs).toSeq) yield {
            val f = field(tc, t, fieldName)
            val setter = gen.escaped("set" + f.getName.capital())
            s"$name.$setter(${value(fs.get(fieldName), f)});\n"
          }

          assignments.mkString
        }
      }
      val rstr = rval.mkString
      if (rstr.endsWith("\n"))
        return rstr.substring(0, rstr.length - 1)
      rstr
    }
  }
}
