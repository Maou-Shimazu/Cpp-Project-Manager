#![allow(unused_imports)]
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::process::{Command, Stdio};
use std::{collections::HashMap, fs, fs::read_to_string, fs::File, path::Path};

/// Creates .clangd, .clang-format and .clang-tidy files and appends content from templates.rs
pub fn create() {
    // Constant file declarations
    const CLANGD: &str = ".clangd";
    const CLANG_FORMAT: &str = ".clang-format";
    const CLANG_TIDY: &str = ".clang-tidy";

    // Checks if the files exist and if they dont, creates them.
    if !Path::new(CLANGD).exists() {
        File::create(CLANGD).expect("could not create .clangd");
    }
    if !Path::new(CLANG_FORMAT).exists() {
        File::create(CLANG_FORMAT).expect("could not create .clang-format");
    }
    if !Path::new(CLANG_TIDY).exists() {
        File::create(CLANG_TIDY).expect("could not create .clang-tidy");
    }

    // Appends content to the files
    fs::write(CLANGD, crate::templates::clangd().as_bytes()).expect("Could not write to .clangd.");
    fs::write(CLANG_FORMAT, crate::templates::CLANG_FORMAT.as_bytes())
        .expect("Could not write to .clang-format.");
    fs::write(CLANG_TIDY, crate::templates::CLANG_TIDY.as_bytes())
        .expect("Could not write to .clang-tidy.");
}

/// Iterates through the project directory and returns a vector of all the c/cpp files in the project.
fn files() -> Vec<String> {
    let files: Vec<String> = glob::glob("**/*.cpp")
        .expect("couldnt glob cpp")
        .chain(glob::glob("**/*.hpp").expect("couldnt glob hpp"))
        .chain(glob::glob("**/*.h").expect("couldnt glob h"))
        .chain(glob::glob("**/*.c").expect("couldnt glob c"))
        .chain(glob::glob("**/*.cxx").expect("couldnt glob cxx"))
        .chain(glob::glob("**/*.hxx").expect("couldnt glob hxx"))
        .map(|x| {
            x.expect("couldnt unwrap pathbuf")
                .as_os_str()
                .to_str()
                .unwrap()
                .to_string()
        })
        .collect::<Vec<_>>();
    files
}

/// Calls clang format and format files in the project.
pub fn format() {
    if crate::cppm::builder::subprocess("clang-format", "--version").is_err() {
        println!(
            "   {}",
            "Could not run clang-format. Please ensure that it is installed and on your PATH."
                .bright_red()
        );
        return;
    }
    let mut cmd = Command::new("clang-format")
        .arg("--Werror")
        .arg("-style=file")
        .arg("--files")
        .args(files())
        .arg("-i")
        .args(files())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Could not run clang-format.");
    cmd.wait().expect("Could not wait on clang-format.");

    println!(
        "    {}`{}`{}",
        "Formatted ".bright_blue(),
        files().len(),
        " files."
    );
}

/// Calls clang tidy and lints the file specified or main.cpp and its included headers.
pub fn clint(src: Option<String>, c: bool) {
    if crate::cppm::builder::subprocess("clang-tidy", "--version").is_err() {
        println!(
            "   {}",
            "Could not run clang-tidy. Please ensure that it is installed and on your PATH."
                .bright_red()
        );
        return;
    }
    let cppm: crate::build::LocalConfig =
        toml_edit::de::from_str(&std::fs::read_to_string("Cppm.toml").unwrap()).unwrap();

    let includes: Vec<&str> = cppm.project["include"].split(", ").collect();
    let includes: String = format!("-I{}", includes.join(" -I"));
    let includes: Vec<&str> = includes.split(" ").collect();

    let source: String;

    if c {
        source = src.unwrap_or("src/main.c".to_string());
    } else {
        source = src.unwrap_or("src/main.cpp".to_string());
    }

    let mut cmd = Command::new("clang-tidy")
        .arg("--quiet")
        .arg("--use-color")
        .arg(source)
        .arg("--")
        .args(includes)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("Could not run clang-tidy.");
    cmd.wait().expect("Could not wait for clang-tidy.");
}
