/*  ___ _  ___ _ _                                                            *\
** / __| |/ (_) | |       The SKilL Generator                                 **
** \__ \ ' <| | | |__     (c) 2013 University of Stuttgart                    **
** |___/_|\_\_|_|____|    see LICENSE                                         **
\*                                                                            */
package de.ust.skill.generator.ada

import de.ust.skill.ir._
import java.io.File
import java.io.PrintWriter
import java.io.BufferedWriter
import java.io.OutputStreamWriter
import java.io.FileOutputStream
import scala.collection.mutable.MutableList
import de.ust.skill.generator.common.Generator

import scala.collection.JavaConversions._

/**
 * The parent class for all output makers.
 *
 * @author Timm Felden, Dennis Przytarski
 */
trait GeneralOutputMaker extends Generator {

  override def getLanguageName = "ada";

  private[ada] def header : String

  // remove special stuff for now
  final def setTC(tc : TypeContext) = this.IR = tc.removeSpecialDeclarations.getUsertypes.to
  var IR : List[UserType] = _

  /**
   * Creates the correct PrintWriter for the argument file.
   */
  override protected def open(path : String) = {
    val f = new File(s"$outPath$packagePath/$path")
    f.getParentFile.mkdirs
    f.createNewFile
    val rval = new PrintWriter(new BufferedWriter(new OutputStreamWriter(
      new FileOutputStream(f), "UTF-8")))
    rval.write(header)
    rval
  }

  /**
   * Assume the existence of a translation function for the types.
   */
  protected def mapTypeToId(t : Type, f : Field) : String
  protected def mapType(t : Type, d : Declaration, f : Field) : String

  /**
   * Assume the existence of a translation function for the fields.
   */
  protected def mapFileReader(d : UserType, f : Field) : String
  protected def mapFileWriter(d : UserType, f : Field) : String

  /**
   * Assume the existence of inheritance information functions for the types.
   */
  protected def getSuperTypes(d : UserType) : MutableList[Type]
  protected def getSubTypes(d : UserType) : MutableList[Type]

  /**
   * Assume the existence of the get field parameters function.
   */
  protected def printParameters(d : UserType) : String

  /**
   * Assume a package prefix provider.
   */
  protected def packagePrefix : String

  /**
   * Tries to escape a string without decreasing the usability of the generated identifier.
   */
  protected def escaped(target : Name) : String = escaped(target.ada)

  private lazy val packagePath = if (packagePrefix.length > 0) {
    packagePrefix.replace(".", "/")
  } else {
    ""
  }
}
