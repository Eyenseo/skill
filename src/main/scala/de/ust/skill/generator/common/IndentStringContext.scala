
package de.ust.skill.generator.common

/**
  * This class allows to conveniently indent substrings in string interpolation
  * by prepending the current indention to all lines in the interpolation
  *
  * @note Adapted from https://stackoverflow.com/a/11426477
  * @author Roland Jaeger
  */
class IndentStringContext(sc: StringContext, margin: Char) {
  def e(args: Any*): String = {
    val sb = new StringBuilder()

    for ((s, a) <- sc.parts.zip(args)) {
      sb.append(s)

      val ind = getIndent(sb, s)
      if (ind.length > 0) {
        sb.append(a.toString.replaceAll("\n", "\n" + ind))
      } else {
        sb.append(a.toString)
      }
    }
    if (sc.parts.size > args.size) {
      sb.append(sc.parts.last)
    }

    sb.toString()
  }

  // get white indent after the last new line, if any
  private final def getIndent(sb: StringBuilder, str: String): String = {
    val lastnl = str.lastIndexOf("\n")

    if (lastnl == -1) {
      val lastnl = sb.lastIndexOf("\n")

      if (lastnl == -1) {
        ""
      } else {
        extractIndent(sb.substring(lastnl + 1).stripMargin(margin))
      }
    } else {
      extractIndent(str.substring(lastnl + 1).stripMargin(margin))
    }
  }

  private final def extractIndent(str: String): String = {
    if (str.trim.isEmpty) {
      str // ind is all whitespace. Use this
    } else {
      val pattern = "^(\\s*).*".r
      str match {
        case pattern(ind) ⇒ ind
        case _            ⇒ ""
      }
    }
  }
}

// TODO find a better way to pass the margin char
object Indenter {
  // top level implicit defs allowed only in 2.10 and above
  implicit def toISC(sc: StringContext): IndentStringContext = new IndentStringContext(sc, '|')
}

object IndenterLaw {
  // top level implicit defs allowed only in 2.10 and above
  implicit def toISC(sc: StringContext): IndentStringContext = new IndentStringContext(sc, '§')
}
