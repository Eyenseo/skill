/*  ___ _  ___ _ _                                                            *\
** / __| |/ (_) | |       The SKilL Generator                                 **
** \__ \ ' <| | | |__     (c) 2013-18 University of Stuttgart                 **
** |___/_|\_\_|_|____|    see LICENSE                                         **
\*                                                                            */
package de.ust.skill.generator.rust

import java.io._

import de.ust.skill.generator.common
import de.ust.skill.generator.common.Indenter._
import de.ust.skill.main.CommandLine
import org.junit.runner.RunWith
import org.scalatest.junit.JUnitRunner

import scala.reflect.io.Path.jfile2path


@RunWith(classOf[JUnitRunner])
class GenericTests extends common.GenericTests {

  class Test(val name: String, val location: String)

  // FIXME remove
  var skipTestCases  = Array(
                              // "age",
                              // "age16",
                              // "ageUnrestricted",
                              // "aircraft",
                              // "annotationNull",
                              // "annotationString",
                              // "annotationTest",
                              // "coloredNodes",
                              // "container",
                              // "crossNodes",
                              // "date",
                              // "duplicateDefinition",
                              // "duplicateDefinitionMixed",
                              // "duplicateDefinitionSecondBlock",
                              // "emptyBlocks",
                              // "fourColoredNodes",
                              // "illegalStringPoolOffsets",
                              // "illegalTypeID",
                              // "localBasePoolOffset",
                              // "missingUserType",
                              // "noFieldRegressionTest",
                              // "nodeFirstBlockOnly",
                              // "nullAsFieldName",
                              // "partial", // TODO - is this really successful?
                              // "trivialType",
                              // "twoNodeBlocks",
                              // "twoTypes",
                              // "unicode_reference",

                              "emptyFile",
                              "restrictionsAll",
                              "nullInNonNullNode",
                            )
  var generatedTests = new Array[Test](0)

  val skipGeneration = Array(
                              // "age",
                              // "annotation",
                              // "basicTypes",
                              // "constants",
                              // "container",
                              // "custom",
                              // "floats",
                              // "hintsAll", // TODO - is this really successful?
                              // "number",
                              // "unicode",
                              // "user",

                              "auto", // FIXME Broken generator
                              "empty", // FIXME Broken generator
                              "enums", // FIXME Broken generator
                              "escaping", // FIXME Broken generator
                              "fancy", // FIXME Broken generator
                              "graph", // FIXME Broken generator
                              "graphInterface", // FIXME Broken generator
                              "map3", // FIXME Broken generator
                              "restrictionsAll", // FIXME Broken generator
                              "restrictionsCore", // FIXME Broken generator
                              "subtypes", // FIXME Broken generator
                              "unknown", // FIXME Broken generator
                            )


  override def language: String = "rust"

  override def deleteOutDir(out: String) {
    import scala.reflect.io.Directory
    Directory(new File("testsuites/rust/", out)).deleteRecursively
  }

  override def callMainFor(name: String,
                           source: String,
                           options: Seq[String]) {
    // TODO remove / extend
    if (skipGeneration.contains(name)) {
      println("Generic Skip: " + name)
      return
    }
    CommandLine.main(Array[String](source,
                                    "--debug-header",
                                    "-L", "rust",
                                    "-p", name,
                                    "-Orust:revealSkillID=true",
                                    "-o", "testsuites/rust/" + name) ++ options)
  }

  override def finalizeTests() {
    val pw = new PrintWriter(new File("testsuites/rust/Cargo.toml"))
    // FIXME hardcoded path
    // TODO move to separate file and share with other tests
    pw.write(
              """[workspace]
                |members = [""".stripMargin
            )
    for (test: Test <- generatedTests) {
      pw.write(
                e""""${test.name}", """.stripMargin
              )
    }
    pw.write("]")
    pw.close()
  }

  /**
    * TODO remove duplicates
    *
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

  def newTestFile(packagePath: String, name: String): PrintWriter = {
    val packageName = packagePath.split("/").map(EscapeFunction.apply).mkString("::")

    generatedTests :+= new Test(packagePath, s"$packagePath/tests/generic${name}Test.rs")

    val f = new File(s"testsuites/rust/$packageName/tests/generic_${snakeCase(packageName)}_${snakeCase(name)}_test.rs")
    f.getParentFile.mkdirs
    if (f.exists) {
      f.delete
    }
    f.createNewFile

    val rval = new PrintWriter(new BufferedWriter(new OutputStreamWriter(new FileOutputStream(f), "UTF-8")))
    rval.write(
                e"""#![feature(test)]
                   |
                   |extern crate $packageName;
                   |
                   |#[cfg(test)]
                   |#[allow(non_snake_case)]
                   |#[allow(unused_must_use)]
                   |mod tests {
                   |    extern crate env_logger;
                   |
                   |    use $packageName::common::SkillFile as SkillFileTrait;
                   |
                   |    use $packageName::skill_file::SkillFile;""".stripMargin
              )
    rval
  }

  def closeTestFile(out: java.io.PrintWriter) {
    out.write(
               """
                 |}
               """.stripMargin)
    out.close()
  }


  def testString(testName: String, file: File, reject: Boolean): String = {
    val escFileName = file.getName.replaceAll("\\W", "_")
    val escFilePath = file.getPath.replaceAll("\\\\", "\\\\\\\\")

    if (skipTestCases.contains(escFileName.replace("_sf", ""))) {
      return ""
    }

    e"""
       |
       |    #[test]${if (reject) "\n#[should_panic]" else ""}
       |    fn ${testName.capitalize}_Parser_${if (reject) "Reject" else "Accept"}_$escFileName() {
       |        let _ = env_logger::try_init();
       |
       |        match SkillFile::open("../../../$escFilePath") {
       |            Ok(sf) => match sf.check() {
       |                Ok(_) => (),
       |                Err(e) => panic!("{}", e)
       |            },
       |            Err(e) => panic!("{}", e),
       |        }
       |    }""".stripMargin
  }

  override def makeTests(name: String) {
    // TODO remove / extend
    if (skipGeneration.contains(name)) {
      return
    }


    val (accept, reject) = collectBinaries(name)

    // generate read tests
    locally {
      val out = newTestFile(name, "Read")

      for (f ← accept) {
        out.write(testString(name, f, false))
      }

      for (f ← reject) {
        out.write(testString(name, f, true))
      }
      closeTestFile(out)
    }
  }
}
