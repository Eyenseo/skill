/*  ___ _  ___ _ _                                                            *\
** / __| |/ (_) | |       The SKilL Generator                                 **
** \__ \ ' <| | | |__     (c) 2013-18 University of Stuttgart                 **
** |___/_|\_\_|_|____|    see LICENSE                                         **
\*                                                                            */
package de.ust.skill.generator.rust

import de.ust.skill.generator.common.IndenterLaw._

import scala.collection.JavaConverters._

trait LiteralKeeper extends GeneralOutputMaker {
  abstract override def make {
    super.make

    makeSource()
  }

  private def makeSource() {
    val out = files.open("src/common/internal/literal_keeper.rs")

    out.write(
               e"""${genUsage()}
                  §
                  §${genLiteralKeeper()}
                  §""".stripMargin('§')
             )

    out.close()
  }

  //----------------------------------------
  // Usage
  //----------------------------------------
  private final def genUsage(): String = {
    e"""use common::SkillString;
       §
       §use std::borrow::Cow;
       §use std::collections::HashSet;
       §use std::rc::Rc;
       §""".stripMargin('§')
  }.trim

  //----------------------------------------
  // LiteralKeeper
  //----------------------------------------
  private final def genLiteralKeeper(): String = {
    e"""//----------------------------------------
       §// LiteralKeeper
       §//----------------------------------------
       §${genLiteralKeeperStruct()}
       §
       §${genLiteralKeeperImpl()}
       §
       §${genLiteralKeeperImplDefault()}
       §""".stripMargin('§')
  }.trim

  def genLiteralKeeperStruct(): String = {
    e"""#[derive(Debug)]
       §pub struct LiteralKeeper {
       §   ${
      (for (s ← allStrings._1; name = getName(s)) yield {
        e"""pub $name: &'static str,
           §""".stripMargin('§')
      }).mkString
    }${
      (for (s ← allStrings._2; name = getName(s)) yield {
        e"""pub $name: &'static str,
           §""".stripMargin('§')
      }).mkString.trim
    }
       §   set: HashSet<Rc<SkillString>>,
       §}
       §""".stripMargin('§')
  }.trim

  def genLiteralKeeperImpl(): String = {
    e"""impl LiteralKeeper {
       §    pub fn get(&mut self, lit: &Rc<SkillString>) -> Option<Rc<SkillString>> {
       §        self.set.take(lit)
       §    }
       §
       §    pub fn get_rest(&mut self) -> Vec<Rc<SkillString>> {
       §        self.set.drain().collect()
       §    }
       §}
       §""".stripMargin('§')
  }.trim

  def genLiteralKeeperImplDefault(): String = {
    e"""impl Default for LiteralKeeper {
       §    fn default() -> LiteralKeeper {
       §        let mut lit = LiteralKeeper {
       §            ${
      (for (s ← allStrings._1; name = getName(s)) yield {
        e"""$name: "$s",
           §""".stripMargin('§')
      }).mkString
    }${
      (for (s ← allStrings._2; name = getName(s)) yield {
        e"""$name: "$s",
           §""".stripMargin('§')
      }).mkString.trim
    }
       §            set: HashSet::with_capacity(${allStrings._1.size + allStrings._2.size}),
       §        };
       §        ${
      (for (s ← allStrings._1; name = getName(s)) yield {
        e"""lit.set.insert(Rc::new(SkillString::from(Cow::from(lit.$name))));
           §""".stripMargin('§')
      }).mkString
    }${
      (for (s ← allStrings._2; name = getName(s)) yield {
        e"""lit.set.insert(Rc::new(SkillString::from(Cow::from(lit.$name))));
           §""".stripMargin('§')
      }).mkString.trim
    }
       §        lit
       §    }
       §}
       §""".stripMargin('§')
  }.trim

  private final def getName(name: String): String = {
    // TODO this shouldn't be needed and the names should be provided not in string from

    (IR ::: IRInterfaces).find(u ⇒ u.getSkillName.equals(name)) match {
      case Some(t) ⇒
        field(t.getName.camel())
      case None    ⇒
        (IR ::: IRInterfaces).flatMap(u ⇒ u.getAllFields.asScala).find(k ⇒ k.getSkillName.equals(name)) match {
          case Some(f) ⇒
            field(f.getName.camel())
          case None    ⇒
            // If we cant fnd a field with the name it has to be a string literal
            field(name)
        }
    }
  }
}
