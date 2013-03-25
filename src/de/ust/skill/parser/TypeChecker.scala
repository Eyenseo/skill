package de.ust.skill.parser

import scala.collection.mutable.HashMap

/**
 * Can type check a list of definitions.
 *
 * @author Timm Felden
 */
class TypeChecker {
  private val builtInTypes = Set[String]("any", "annotation", "bool", "i1", "i8", "i16", "i32", "i64", "v64", "string",
    "f64", "f32")
  private var knownTypes: HashMap[String, Definition] = null;

  def check(defs: List[Definition]) {
    assert(defs.size > 0, "There are no definitions to check!")

    knownTypes = new HashMap
    builtInTypes.foreach(s => knownTypes.put(s, null))
    // ensure that type names are unique and do not overwrite built in types
    defs.foreach(d => {
      assert(!knownTypes.contains(d.name), "duplicate type definition " + d.name)
      knownTypes.put(d.name, d)
    })

    defs.foreach(d =>
      d.body.foreach(f => {
        // ensure that all field types are known
        f.t match {
          case t: MapType => t.args.foreach(t => assert(knownTypes.contains(t.name)))
          case t: SetType => assert(knownTypes.contains(t.baseType.name),
            "in " + d.name + "::" + f.name + ":> unknown base type " + t.baseType)
          case t: ListType => assert(knownTypes.contains(t.baseType.name),
            "in " + d.name + "::" + f.name + ":> unknown base type " + t.baseType)
          case t: ArrayType => assert(knownTypes.contains(t.baseType.name),
            "in " + d.name + "::" + f.name + ":> unknown base type " + t.baseType)
          case t: GroundType ⇒ assert(knownTypes.contains(t.name),
            "in " + d.name + "::" + f.name + ":> unknown base type " + t.name)
        }
        // ensure that field names are unique inside the definition
        assert(d.body.count(o => f.name.equals(o.name)) == 1)

        // ensure that array types are well-formed
        f.t match {
          case t: ConstantArrayType => assert(t.length > 0)
          case t: DependentArrayType => {
            assert(!t.lengthFieldName.equals(f.name),
              "in " + d.name + "::" + f.name + ":> the array length is self dependent")
            assert(d.body.count(o => t.lengthFieldName.equals(o.name)) > 0,
              "in " + d.name + "::" + f.name + ":> the array length " + t.lengthFieldName +
                " does not refer to another field\n" + ASTPrettyPrinter.printNode(d))
          }
          case _ => ()
        }
        // ensure that constants are well-formed
        f match {
          case c: Constant => assert(c.t.isInstanceOf[GroundType] &&
            (c.t.asInstanceOf[GroundType].name.startsWith("i") ||
              c.t.asInstanceOf[GroundType].name.startsWith("v")) &&
              builtInTypes.contains(c.t.asInstanceOf[GroundType].name),
            "in " + d.name + "::" + f.name + ":> a constant must have an integer type!")
          case _ => ()
        }
      }))
  }
}