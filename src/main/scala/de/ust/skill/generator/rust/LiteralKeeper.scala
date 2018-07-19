/*  ___ _  ___ _ _                                                            *\
** / __| |/ (_) | |       The SKilL Generator                                 **
** \__ \ ' <| | | |__     (c) 2013-18 University of Stuttgart                 **
** |___/_|\_\_|_|____|    see LICENSE                                         **
\*                                                                            */
package de.ust.skill.generator.rust

import de.ust.skill.generator.common.Indenter._

trait LiteralKeeper extends GeneralOutputMaker {
  abstract override def make {
    super.make

    makeSource()
  }

  private def makeSource() {
    val out = files.open("src/common/internal/literal_keeper.rs")

    out.write(
               e"""${genUsage()}
                  |
                  |${genLiteralKeeper()}
                  |""".stripMargin
             )

    out.close()
  }

  //----------------------------------------
  // Usage
  //----------------------------------------
  private final def genUsage(): String = {
    e"""use std::borrow::Cow;
       |use std::collections::HashSet;
       |use std::rc::Rc;
       |""".stripMargin
  }.trim

  //----------------------------------------
  // LiteralKeeper
  //----------------------------------------
  private final def genLiteralKeeper(): String = {
    e"""//----------------------------------------
       |// LiteralKeeper
       |//----------------------------------------
       |${genLiteralKeeperStruct()}
       |
       |${genLiteralKeeperImpl()}
       |
       |${genLiteralKeeperImplDefault()}
       |""".stripMargin
  }.trim

  def genLiteralKeeperStruct(): String = {
    e"""#[derive(Debug)]
       |pub struct LiteralKeeper {
       |   ${
      (for (s ← allStrings._1; name = literal_field(s)) yield {
        e"""pub $name: &'static str,
           |""".mkString
      }).mkString
    }${
      (for (s ← allStrings._2; name = literal_field(s)) yield {
        e"""pub $name: &'static str,
           |""".mkString
      }).mkString.trim
    }
       |   set: HashSet<Rc<String>>,
       |}
       |""".stripMargin
  }.trim

  def genLiteralKeeperImpl(): String = {
    e"""impl LiteralKeeper {
       |    pub fn get(&mut self, lit: &str) -> Option<Rc<String>> {
       |        self.set.take(&String::from(Cow::Borrowed(lit)))
       |    }
       |
       |    pub fn get_rest(&mut self) -> Vec<Rc<String>> {
       |        self.set.drain().collect()
       |    }
       |}
       |""".stripMargin
  }.trim

  def genLiteralKeeperImplDefault(): String = {
    e"""impl Default for LiteralKeeper {
       |    fn default() -> LiteralKeeper {
       |        let mut lit = LiteralKeeper {
       |            ${
      (for (s ← allStrings._1; name = literal_field(s)) yield {
        e"""$name: "$s",
           |""".mkString
      }).mkString
    }${
      (for (s ← allStrings._2; name = literal_field(s)) yield {
        e"""$name: "$s",
           |""".mkString
      }).mkString.trim
    }
       |            set: HashSet::with_capacity(${allStrings._1.size + allStrings._2.size}),
       |        };
       |        ${
      (for (s ← allStrings._1; name = literal_field(s)) yield {
        e"""lit.set.insert(Rc::new(String::from(Cow::Borrowed(lit.$name))));
           |""".mkString
      }).mkString
    }${
      (for (s ← allStrings._2; name = literal_field(s)) yield {
        e"""lit.set.insert(Rc::new(String::from(Cow::Borrowed(lit.$name))));
           |""".mkString
      }).mkString.trim
    }
       |        lit
       |    }
       |}
       |""".mkString
  }.trim
}
