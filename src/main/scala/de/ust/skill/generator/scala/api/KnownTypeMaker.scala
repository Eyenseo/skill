package de.ust.skill.generator.scala.api

import de.ust.skill.generator.scala.GeneralOutputMaker

trait KnownTypeMaker extends GeneralOutputMaker {
  override def make {
    super.make
    val out = open("api/KnownType.scala")
    //package & imports
    out.write(s"""package ${packagePrefix}api

/**
 * The top of the known types hierarchy.
 *
 * @author Timm Felden
 */
trait KnownType extends SkillType;
""")

    out.close()
  }
}
