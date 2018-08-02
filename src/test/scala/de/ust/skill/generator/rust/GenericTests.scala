/*  ___ _  ___ _ _                                                            *\
** / __| |/ (_) | |       The SKilL Generator                                 **
** \__ \ ' <| | | |__     (c) 2013-18 University of Stuttgart                 **
** |___/_|\_\_|_|____|    see LICENSE                                         **
\*                                                                            */
package de.ust.skill.generator.rust

import java.io._

import de.ust.skill.generator.common
import de.ust.skill.generator.common.IndenterLaw._
import de.ust.skill.main.CommandLine
import org.junit.runner.RunWith
import org.scalatest.junit.JUnitRunner


@RunWith(classOf[JUnitRunner])
class GenericTests extends common.GenericTests {
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
                              // "emptyFile",
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
                              // "twoTypes", // FIXME
                              // "unicode-reference",

                              "restrictionsAll",
                              "nullInNonNullNode",
                              "",
                            )
  var generatedTests = new Array[String](0)

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

    val pkgEsc = escSnakeCase(out.split("/").mkString("_"))
    Directory(new File("testsuites/rust/", pkgEsc)).deleteRecursively
  }

  override def callMainFor(name: String,
                           source: String,
                           options: Seq[String]) {
    if (skipGeneration.contains(name)) {
      println("Generic Skip: " + name)
      return
    }
    val pkgEsc = escSnakeCase(name.split("/").mkString("_"))

    CommandLine.main(Array[String](source,
                                    "--debug-header",
                                    "-L", "rust",
                                    "-p", name,
                                    "-o", "testsuites/rust/" + pkgEsc) ++ options)
  }

  override def finalizeTests() {
    val pw = new PrintWriter(new File("testsuites/rust/Cargo.toml"))
    // FIXME hardcoded path
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

  def newTestFile(packagePath: String, name: String): PrintWriter = {
    val pkgEsc = escSnakeCase(packagePath.split("/").mkString("_"))

    generatedTests :+= pkgEsc

    val f = new File(s"testsuites/rust/$pkgEsc/tests/generic_${escSnakeCase(name)}.rs")

    f.getParentFile.mkdirs
    if (f.exists) {
      // TODO is this ... "ok"
      f.delete
    }
    f.createNewFile

    val rval = new PrintWriter(new BufferedWriter(new OutputStreamWriter(new FileOutputStream(f), "UTF-8")))
    rval.write(
                e"""#![feature(test)]
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
                   §    use $pkgEsc::common::error::*;
                   §    use $pkgEsc::skill_file::SkillFile;
                   §
                   §    use self::failure::Fail;""".stripMargin('§')
              )
    rval
  }

  def closeTestFile(out: java.io.PrintWriter) {
    out.write(
               """
                 §}
                 §""".stripMargin('§'))
    out.close()
  }


  def testString(testName: String, file: File, reject: Boolean): String = {
    val escFilePath = file.getPath.replaceAll("\\\\", "\\\\\\\\")
    val testCase = file.getName.replaceAll("\\..*$", "")

    if (skipTestCases.contains(testCase)) {
      return ""
    }

    e"""
       §
       §    #[test]${if (reject) "\n#[should_panic]" else ""}
       §    fn generic_${escSnakeCase(testName)}_${if (reject) "reject" else "accept"}_${
      escSnakeCase(testCase.replaceAll("_|-", ""))
    }() {
       §        let _logger = env_logger::try_init();
       §
       §        match SkillFile::open("../../../$escFilePath") {
       §            Ok(sf) => match || -> Result<(), SkillFail> {
       §                sf.check()?;
       §                Ok(())
       §            }() {
       §                Ok(_) => (),
       §                Err(e) => if let Some(bt) = e.backtrace() {
       §                    panic!("{}\n{}", e, bt)
       §                } else {
       §                    panic!("{}", e)
       §                },
       §            },
       §            Err(e) => if let Some(bt) = e.backtrace() {
       §                panic!("{}\n{}", e, bt)
       §            } else {
       §                panic!("{}", e)
       §            },
       §        }
       §    }""".stripMargin('§')
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
