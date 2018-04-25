/*  ___ _  ___ _ _                                                            *\
** / __| |/ (_) | |       The SKilL Generator                                 **
** \__ \ ' <| | | |__     (c) 2013-18 University of Stuttgart                 **
** |___/_|\_\_|_|____|    see LICENSE                                         **
\*                                                                            */
package de.ust.skill.generator.rust

import java.io.File
import java.nio.file.Files


trait DependenciesMaker extends GeneralOutputMaker {

  final private val depsSrc = new File("deps/rust/src")

  final private def gatherFiles(dir: File): List[File] = {
    val these = dir.listFiles.toList
    these ++ these.filter(_.isDirectory).flatMap(gatherFiles)
  }

  abstract override def make {
    super.make

    // safe unnecessary overwrites that cause race conditions on parallel builds anyway
    if (!skipDependencies)
      this.getClass.synchronized(
        {
          if (!(depsSrc.exists && depsSrc.isDirectory)) {
            new GeneratorException("The directory " + depsSrc.getAbsolutePath + " apparently does not exist.")
          }

          val files: List[File] = gatherFiles(depsSrc)

          for (file ← files) {
            val out = new File(depsPath, "src/common" + file.getPath.replaceFirst(depsSrc.getPath, ""))

            out.getParentFile.mkdirs()
            Files.deleteIfExists(out.toPath)
            Files.copy(file.toPath, out.toPath)
          }
        })
  }
}
