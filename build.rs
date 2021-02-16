
#![allow(unused)]
use std::process::Command;
use std::io;
#[cfg(windows)] use winres::WindowsResource;

fn main() {
  if cfg!(target_os = "windows") {
    Command::new("packfolder.exe")
      .args(&["build", "target/assets.rc", "-binary"])
      .output()
      .expect("failed to execute packfolder.exe, is it in your PATH?");
      WindowsResource::new()
        .set_icon("icon.ico")
        .compile();
    }
}