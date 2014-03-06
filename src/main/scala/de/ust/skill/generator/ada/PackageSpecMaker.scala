/*  ___ _  ___ _ _                                                            *\
** / __| |/ (_) | |       The SKilL Generator                                 **
** \__ \ ' <| | | |__     (c) 2013 University of Stuttgart                    **
** |___/_|\_\_|_|____|    see LICENSE                                         **
\*                                                                            */
package de.ust.skill.generator.ada

import scala.collection.JavaConversions._

trait PackageSpecMaker extends GeneralOutputMaker {
  abstract override def make {
    super.make
    val out = open(s"""${packagePrefix}.ads""")

    out.write(s"""
with Ada.Containers.Indefinite_Hashed_Maps;
with Ada.Containers.Indefinite_Vectors;
with Ada.Strings.Hash;
with Ada.Strings.Unbounded;

with Ada.Text_IO;

package ${packagePrefix.capitalize} is

   package SU renames Ada.Strings.Unbounded;

   -------------
   --  TYPES  --
   -------------
   --  Short_Short_Integer
   subtype i8 is Short_Short_Integer'Base range -(2**7) .. +(2**7 - 1);

   --  Short_Integer
   subtype Short is Short_Integer'Base range -(2**15) .. +(2**15 - 1);
   subtype i16 is Short;

   --  Integer
   subtype i32 is Integer'Base range -(2**31) .. +(2**31 - 1);

   --  Long_Integer
   subtype Long is Long_Integer'Base range -(2**63) .. +(2**63 - 1);
   subtype i64 is Long;
   subtype v64 is Long;

   --  Float
   subtype f32 is Float;

   --  Long_Float
   subtype Double is Long_Float'Base;
   subtype f64 is Double;

   -------------
   --  SKILL  --
   -------------
   type Skill_State is limited private;
   type Skill_Type is abstract tagged private;
   type Skill_Type_Access is access all Skill_Type'Class;

${
  var output = "";
  for (d ← IR) {
    output += s"""   type ${d.getName}_Type is new Skill_Type with private;\r\n"""
    output += s"""   type ${d.getName}_Type_Access is access all ${d.getName}_Type;\r\n"""
  }
  output
}
${
  var output = "";
  for (d ← IR) {
    d.getAllFields.filter({ f ⇒ !f.isIgnored }).foreach({ f =>
      output += s"""   function Get_${f.getName.capitalize} (Object : ${d.getName}_Type) return ${mapType(f.getType)};\r\n"""
      if (!f.isConstant)
        output += s"""   procedure Set_${f.getName.capitalize} (Object : in out ${d.getName}_Type; Value : ${mapType(f.getType)});\r\n"""
    })
  }
  output
}
private

   type Skill_Type is abstract tagged null record;

${
  var output = "";
  for (d ← IR) {
    val superType = if (d.getSuperType == null) "Skill" else d.getSuperType.getName
    output += s"""   type ${d.getName}_Type is new ${superType}_Type with\r\n      record\r\n"""
    val fields = d.getFields.filter({ f ⇒ !f.isConstant && !f.isIgnored })
    output += fields.map({ f ⇒
      var comment = "";
      if (f.isAuto()) comment = "  --  auto aka not serialized"
      s"""         ${f.getName} : ${mapType(f.getType)};${comment}"""
    }).mkString("\r\n")
    if (fields.length <= 0) output += s"""         null;"""
    output += s"""\r\n      end record;\r\n\r\n"""
  }
  output.stripSuffix("\r\n")
}
   ------------------
   --  STRING POOL --
   ------------------
   package String_Pool_Vector is new Ada.Containers.Indefinite_Vectors (Positive, String);

   --------------------
   --  STORAGE POOL  --
   --------------------
   package Storage_Pool_Vector is new Ada.Containers.Indefinite_Vectors (Positive, Skill_Type_Access);

   --------------------------
   --  FIELD DECLARATIONS  --
   --------------------------
   type Field_Declaration (Size : Positive) is
      record
         Name : String (1 .. Size);
         F_Type : Long;
         Constant_Value : Long;
      end record;
   type Field_Information is access Field_Declaration;

   package Fields_Vector is new Ada.Containers.Indefinite_Vectors
      (Index_Type => Positive, Element_Type => Field_Information);

   -------------------------
   --  TYPE DECLARATIONS  --
   -------------------------
   type Type_Declaration (Size : Positive) is
      record
         Name : String (1 .. Size);
         Super_Name : Long;
         bpsi : Positive;
         lbpsi : Positive;
         Fields : Fields_Vector.Vector;
         Storage_Pool : Storage_Pool_Vector.Vector;
      end record;
   type Type_Information is access Type_Declaration;

   package Types_Hash_Map is new Ada.Containers.Indefinite_Hashed_Maps
      (String, Type_Information, Ada.Strings.Hash, "=");

   -------------------
   --  SKILL STATE  --
   -------------------
   protected type Skill_State is

      --  string pool
      function Get_String (Index : Positive) return String;
      function Get_String (Index : Long) return String;
      function Get_String_Index (Value : String) return Natural;
      function String_Pool_Size return Natural;
      procedure Put_String (Value : String);

      --  storage pool
      function Storage_Pool_Size (Type_Name : String) return Natural;
      function Get_Object (Type_Name : String; Position : Positive) return Skill_Type_Access;
      procedure Put_Object (Type_Name : String; New_Object : Skill_Type_Access);
      procedure Replace_Object (Type_Name : String; Position : Positive; New_Object : Skill_Type_Access);

      --  field declarations
      function Field_Size (Type_Name : String) return Natural;
      function Get_Field (Type_Name : String; Position : Positive) return Field_Information;
      function Get_Field (Type_Name : String; Position : Long) return Field_Information;
      procedure Put_Field (Type_Name : String; New_Field : Field_Information);

      --  type declarations
      function Type_Size return Natural;
      function Has_Type (Name : String) return Boolean;
      function Get_Type (Name : String) return Type_Information;
      procedure Put_Type (New_Type : Type_Information);
      function Get_Types return Types_Hash_Map.Map;

   private

      String_Pool : String_Pool_Vector.Vector;
      Types : Types_Hash_Map.Map;

   end Skill_State;

end ${packagePrefix.capitalize};
""")

    out.close()
  }
}
