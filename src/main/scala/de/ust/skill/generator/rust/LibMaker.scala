/*  ___ _  ___ _ _                                                            *\
** / __| |/ (_) | |       The SKilL Generator                                 **
** \__ \ ' <| | | |__     (c) 2013-18 University of Stuttgart                 **
** |___/_|\_\_|_|____|    see LICENSE                                         **
\*                                                                            */
package de.ust.skill.generator.rust

import de.ust.skill.generator.common.IndenterLaw._

/**
  * Generates the lib.rs and Cargo.toml file
  */
trait LibMaker extends GeneralOutputMaker {

  abstract override def make {
    super.make

    genLibRS()
    genCargo()
  }

  //----------------------------------------
  // lib.rs
  //----------------------------------------
  private final def genLibRS(): Unit = {
    val out = files.open("src/lib.rs")

    out.write(
               e"""// FIXME remove
                  §#![allow(unused_imports)]
                  §#![allow(unused_variables)]
                  §#![allow(unused_mut)]
                  §#![allow(dead_code)]
                  §
                  §#![recursion_limit="128"]
                  §
                  §#![feature(nll)]
                  §#![feature(coerce_unsized)]
                  §#![feature(unsize)]
                  §#![feature(box_into_raw_non_null)]
                  §#![feature(specialization)]
                  §#![feature(core_intrinsics)]
                  §#![feature(optin_builtin_traits)]
                  §
                  §#[macro_use]
                  §extern crate log;
                  §extern crate memmap;
                  §
                  §extern crate failure;
                  §#[macro_use]
                  §extern crate failure_derive;
                  §
                  §#[macro_use]
                  §extern crate lazy_static;
                  §
                  §#[macro_use]
                  §pub mod common;
                  §
                  §mod skill_file;
                  §pub(crate) mod ptr;
                  §
                  §pub(crate) use self::skill_file::SkillFileBuilder;
                  §pub use self::skill_file::FileMode;
                  §pub use self::skill_file::SkillFile;
                  §
                  §${genModuleUsage()}
                  §""".stripMargin('§')
             )

    out.close()
  }

  private final def genModuleUsage(): String = {
    val ret = new StringBuilder()

    for (base ← IR) {
      val low_base = field(base)

      ret.append(
                  e"""pub mod $low_base;
                     §pub use $low_base::*;
                     §""".stripMargin('§')
                )
    }
    for (base ← IRInterfaces) {
      val low_base = field(base)

      ret.append(
                  e"""pub mod $low_base;
                     §pub use $low_base::*;
                     §""".stripMargin('§')
                )
    }
    ret.mkString.trim
  }

  //----------------------------------------
  // Cargo.toml
  //----------------------------------------
  private final def genCargo(): Unit = {
    val out = files.openRaw("Cargo.toml")
    out.write(
               e"""[package]
                  §name = "skill_${snakeCase(packageName)}"
                  §version = "0.1.0"
                  §publish = false
                  §
                  §[lib]
                  §name = "${snakeCase(packageName)}"
                  §test = true
                  §doctest = false
                  §
                  §[dev-dependencies]
                  §env_logger = "0.5.11"
                  §
                  §[dependencies]
                  §memmap = "0.6.2"
                  §log = { version = "0.4.4", features = ["max_level_trace", "release_max_level_off"] }
                  §failure = "0.1.2"
                  §failure_derive = "0.1.2"
                  §lazy_static = "1.1.0"
                  §
                  §""".stripMargin('§')
             )
    out.close()
  }
}
