// Copyright (C) 2022-2022 Fuwn <contact@fuwn.me>
// SPDX-License-Identifier: MIT

use std::env::var;

/// <https://github.com/denoland/deno/blob/main/cli/build.rs#L265:L285>
fn git_commit_hash() -> String {
  if let Ok(output) = std::process::Command::new("git")
    .arg("rev-list")
    .arg("-1")
    .arg("HEAD")
    .output()
  {
    if output.status.success() {
      std::str::from_utf8(&output.stdout[..40]).unwrap().to_string()
    } else {
      "UNKNOWN".to_string()
    }
  } else {
    "UNKNOWN".to_string()
  }
}

fn main() {
  println!("cargo:rustc-env=GIT_COMMIT_HASH={}", git_commit_hash());
  println!("cargo:rerun-if-env-changed=GIT_COMMIT_HASH");
  println!("cargo:rustc-env=TARGET={}", var("TARGET").unwrap());
  println!("cargo:rustc-env=PROFILE={}", var("PROFILE").unwrap());
}
