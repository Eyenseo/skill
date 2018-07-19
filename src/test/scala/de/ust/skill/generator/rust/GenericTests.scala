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
                              "",
                            )
  var generatedTests = new Array[Test](0)

  val skipGeneration = Array(
                              // "age",
                              // "annotation",
                              // "auto", // TODO - is this really successful?
                              // "basicTypes",
                              // "constants",
                              // "container",
                              // "custom",
                              // "empty",
                              // "enums", // TODO - is this really successful?
                              // "escaping",
                              // "fancy",
                              // "floats",
                              // "graph",
                              // "graphInterface", // TODO - is this really successful?
                              // "hintsAll", // TODO - is this really successful?
                              // "map3",
                              // "number",
                              // "restrictionsAll", // TODO - is this really successful?
                              // "restrictionsCore", // TODO - is this really successful?
                              // "subtypes",
                              // "unicode",
                              // "unknown",
                              // "user",
                              "",
                            )


  override def language: String = "rust"

  override def deleteOutDir(out: String) {
    import scala.reflect.io.Directory
    Directory(new File("testsuites/rust/", out)).deleteRecursively
  }

  override def callMainFor(name: String,
                           source: String,
                           options: Seq[String]) {
    if (skipGeneration.contains(name)) {
      println("Generic Skip: " + name)
      return
    }
    CommandLine.main(Array[String](source,
                                    "--debug-header",
                                    "-L", "rust",
                                    "-p", name,
                                    "-o", "testsuites/rust/" + name) ++ options)
  }

  override def finalizeTests() {
    val pw = new PrintWriter(new File("testsuites/rust/Cargo.toml"))
    // FIXME hardcoded path
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

  def snakeCase(str: String): String = GeneralOutputMaker.snakeCase(str)

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
                   |extern crate ${snakeCase(packageName)};
                   |
                   |#[cfg(test)]
                   |#[allow(non_snake_case)]
                   |#[allow(unused_must_use)]
                   |mod tests {
                   |    extern crate env_logger;
                   |
                   |    use ${snakeCase(packageName)}::common::SkillFile as SkillFileTrait;
                   |
                   |    use ${snakeCase(packageName)}::skill_file::SkillFile;""".stripMargin
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
