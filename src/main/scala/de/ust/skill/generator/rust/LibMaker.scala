/*  ___ _  ___ _ _                                                            *\
** / __| |/ (_) | |       The SKilL Generator                                 **
** \__ \ ' <| | | |__     (c) 2013-18 University of Stuttgart                 **
** |___/_|\_\_|_|____|    see LICENSE                                         **
\*                                                                            */
package de.ust.skill.generator.rust

import de.ust.skill.generator.common.Indenter._

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
                  |#![allow(unused)]
                  |#![allow(dead_code)]
                  |
                  |#![feature(coerce_unsized)]
                  |#![feature(unsize)]
                  |#![feature(box_into_raw_non_null)]
                  |#![feature(specialization)]
                  |
                  |#[macro_use]
                  |extern crate log;
                  |extern crate memmap;
                  |
                  |#[macro_use]
                  |pub mod common;
                  |
                  |pub mod skill_file;
                  |pub mod ptr;
                  |
                  |${genModuleUsage()}
                  |""".stripMargin
             )

    out.close()
  }

  private final def genModuleUsage(): String = {
    val ret = new StringBuilder()


    for (base ← IR) {
      val low_base = snakeCase(storagePool(base))

      ret.append(
                  e"""pub mod $low_base;
                     |""".stripMargin
                )
    }
    ret.mkString.trim
  }

  //----------------------------------------
  // Cargo.toml
  //----------------------------------------
  private final def genCargo(): Unit = {
    // TODO replace hardcoded path to dependency
    val out = files.openRaw("Cargo.toml")
    out.write(
               e"""[package]
                  |name = "skill-${snakeCase(packageName)}-tests"
                  |version = "0.1.0"
                  |publish = false
                  |
                  |[lib]
                  |name = "${snakeCase(packageName)}"
                  |test = true
                  |doctest = false
                  |
                  |[dependencies]
                  |memmap = "0.6.2"
                  |log = { version = "0.4", features = ["max_level_trace", "release_max_level_off"] }
                  |
                  |[dev-dependencies]
                  |env_logger = "0.5.10"
                  |""".stripMargin
             )
    out.close()
  }
}
