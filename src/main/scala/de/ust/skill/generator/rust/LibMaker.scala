/*  ___ _  ___ _ _                                                            *\
** / __| |/ (_) | |       The SKilL Generator                                 **
** \__ \ ' <| | | |__     (c) 2013-18 University of Stuttgart                 **
** |___/_|\_\_|_|____|    see LICENSE                                         **
\*                                                                            */
package de.ust.skill.generator.rust

import de.ust.skill.generator.common.IndenterLaw._

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
                  §#![feature(nll)]
                  §#![feature(coerce_unsized)]
                  §#![feature(unsize)]
                  §#![feature(box_into_raw_non_null)]
                  §#![feature(specialization)]
                  §#![feature(core_intrinsics)]
                  §#![feature(extern_prelude)]
                  §#![feature(slice_index_methods)]
                  §
                  §#[macro_use]
                  §extern crate log;
                  §extern crate memmap;
                  §
                  §
                  §extern crate failure;
                  §#[macro_use]
                  §extern crate failure_derive;
                  §
                  §#[macro_use]
                  §pub mod common;
                  §
                  §pub mod skill_file;
                  §pub mod ptr;
                  §
                  §${genModuleUsage()}
                  §""".stripMargin('§')
             )

    out.close()
  }

  private final def genModuleUsage(): String = {
    val ret = new StringBuilder()


    for (base ← IR) {
      val low_base = snakeCase(storagePool(base))

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
    // TODO replace hardcoded path to dependency
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
                  §[dependencies]
                  §memmap = "0.6.2"
                  §log = { version = "0.4.3", features = ["max_level_trace", "release_max_level_off"] }
                  §failure = "0.1.2"
                  §failure_derive = "0.1.2"
                  §
                  §[dev-dependencies]
                  §env_logger = "0.5.11"
                  §""".stripMargin('§')
             )
    out.close()
  }
}
