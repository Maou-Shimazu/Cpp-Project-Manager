#![allow(unused_assignments)]
/// Build, handles the functions for the build process.
use crate::cppm;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{self, read_to_string},
    io::{self, Write},
    path::Path,
    process::*,
    str,
    time::Instant,
};

/// ### Struct used to serialze Cppm.toml
/// Usage:
/// ```rs
/// let value: LocalConfig = toml::from_str(&std::fs::read_to_string("Cppm.toml").unwrap
/// ```
#[derive(Serialize, Deserialize, Debug)]
pub struct LocalConfig {
    pub project: HashMap<String, String>,
    pub dependencies: HashMap<String, String>,
}

/// Generates an include command based on all the folders passed into the argument.\
/// Used in [build](https://github.com/Maou-Shimazu/Cpp-Project-Manager/blob/main/src/build.rs#L56)
// fn include(folders: Vec<&str>) -> Vec<&str> {
//     let mut return_string: Vec<&str> = Vec::new();
//     for i in folders {
//         return_string.push(i);
//     }
//     let join = format!("-I{}", return_string.join(" -I"));
//     let join = join.split(" ").collect();
//     return join;
// }

/// Links libraries specified.\
/// Used in [build](https://github.com/Maou-Shimazu/Cpp-Project-Manager/blob/main/src/build.rs#L56)
// fn library(libs: Vec<&str>) -> Vec<&str> {
//     let mut return_string: Vec<&str> = Vec::new();
//     for i in libs {
//         return_string.push(i);
//     }
//     let join = format!("-L{}", return_string.join(" -L"));
//     join.split(" ").collect()
// }

/// ### Struct used to serialze defaults file
/// Usage:
/// ```rs
/// let value: Def = toml::from_str(&std::fs::read_to_string(&cppm::defaults_file()).unwrap()).unwrap();
/// ```
#[derive(Debug, Deserialize, Serialize)]
pub struct Def {
    compilers: HashMap<String, String>,
}

/// ### Main Build Function.
/// Handles the following:
/// - Checks if Cppm.toml is present in current directory.
/// - Checks if  `&defaults_file()` exists.
/// - Times the build process.
/// - Handles build targets.
/// - Builds executable.
/// - Reads available compilers.
// note: add `flags_all = bool`, `flags = ""`
// note: optimize for smart object building headerfiles in the future
pub fn build(release: bool, run_type: bool, i: bool, c: bool) {
    let start = Instant::now();
    if !Path::new("Cppm.toml").exists() {
        println!("{}", "Cppm project isnt in current directory!".red());
        exit(0);
    }
    if !Path::new(&defaults_file()).exists() {
        println!(
            "{}",
            "You haven't set up your defaults yet! Run `cppm --config` to resolve this error."
                .red()
        );
        exit(0);
    }

    let mut target = String::new();
    let mut build_t = String::new();

    let l: LocalConfig = toml::from_str(&read_to_string("Cppm.toml").unwrap()).unwrap();

    let canc: String = std::fs::canonicalize(".")
        .unwrap()
        .as_os_str()
        .to_str()
        .unwrap()
        .to_owned();

    #[cfg(windows)]
    let canc = canc.replace('\\', "\\").trim()[4..].to_owned();

    println!(
        "   {} {} v{} ({})",
        "Compiling".bright_blue().bold(),
        l.project["name"],
        l.project["version"],
        canc
    );

    let cppm: LocalConfig = toml::from_str(&read_to_string("Cppm.toml").unwrap()).unwrap();

    // Setting Env Vars. note: Only available when running with cppm.
    std::env::set_var("VERSION", cppm.project["version"].clone());
    std::env::set_var("PKGNAME", cppm.project["name"].clone());
    let compilers: Def = toml::from_str(&read_to_string(&cppm::defaults_file()).unwrap()).unwrap();

    let includes: Vec<&str> = cppm.project["include"].split(", ").collect();
    let includes: String = format!("-I{}", includes.join(" -I"));
    let includes: Vec<&str> = includes.split(" ").collect();
    let includes: Vec<String> = crate::dependencies::read_deps(includes);
    let includes: Vec<&str> = includes.iter().map(|f| f.as_str()).collect();

    let mut libraries: Vec<&str> = Vec::new(); // note: someone please test that the libraries link properly.
    if cppm.project.contains_key("libs") {
        libraries = cppm.project["libs"].split(", ").collect();
    } else {
        libraries = vec![""];
    }

    let l: String = if libraries.is_empty() {
        format!("-l{}", libraries.join(" -l"))
    } else {
        "".to_string()
    };
    libraries = l.split(" ").collect();

    let mut extra: Vec<&str> = Vec::new();
    if cppm.project.contains_key("extra") {
        extra = cppm.project["extra"].split(", ").collect();
    } else {
        extra = vec![""];
    }
    let mut flags: Vec<&str> = vec![
        "-fdiagnostics-color=always",
        "-Wall",
        "-Wpedantic",
        "-Werror",
        "-Wshadow",
        "-Wformat=2",
        "-Wconversion",
        "-Wunused-parameter",
    ];
    if let Some(flag_string) = cppm.project.get("flags").filter(|f| !f.is_empty()) {
        flags = flag_string.split(", ").collect();
    }
    let src = cppm.project["src"].clone();

    let mut standard = cppm.project["standard"].clone();
    standard = if c {
        format!("-std=c{standard}")
    } else {
        format!("-std=c++{standard}")
    };
    fs::create_dir_all("build").ok();

    let out: Output;

    let compiler: String = if c {
        compilers.compilers["c"].clone()
    } else {
        compilers.compilers["cpp"].clone()
    };

    if release {
        // use variables to make a better implimentation of this
        out = Command::new(&compiler)
            .arg(standard.clone())
            .arg("-o")
            .arg(format!("build/{}", cppm.project["name"]))
            .arg(src.clone())
            .args(extra.clone())
            .args(includes.clone())
            .args(libraries.clone())
            .arg("-O3")
            .args(flags.clone())
            .arg("-D") // note: look into a better way to impliment the quotes, test on linux. note: plug these in a constant array.
            .arg(format!(
                "VERSION=\"\"\"\"\"\"\"{}\"\"\"\"\"\"\"",
                cppm.project["version"]
            ))
            .arg("-D")
            .arg(format!(
                "NAME=\"\"\"\"\"\"\"{}\"\"\"\"\"\"\"",
                cppm.project["name"]
            ))
            .arg("-D")
            .arg(format!(
                "EDITION=\"\"\"\"\"\"\"{}\"\"\"\"\"\"\"",
                cppm.project["edition"]
            ))
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .output()
            .unwrap();
        target = "optimized".to_string();
        build_t = "release".to_string();
    } else {
        out = Command::new(&compiler)
            .arg(standard.clone())
            .arg("-o")
            .arg(format!("build/{}", cppm.project["name"]))
            .arg(src.clone())
            .args(extra.clone())
            .args(includes.clone())
            .args(libraries.clone())
            .args(flags.clone())
            .arg("-D") // note: look into a better way to impliment the quotes, test on linux.
            .arg(format!(
                "VERSION=\"\"\"\"\"\"\"{}\"\"\"\"\"\"\"",
                cppm.project["version"]
            ))
            .arg("-D")
            .arg(format!(
                "NAME=\"\"\"\"\"\"\"{}\"\"\"\"\"\"\"",
                cppm.project["name"]
            ))
            .arg("-D")
            .arg(format!(
                "EDITION=\"\"\"\"\"\"\"{}\"\"\"\"\"\"\"",
                cppm.project["edition"]
            ))
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .output()
            .unwrap();
        target = "unoptimized".to_owned();
        build_t = "dev".to_string();
    }

    if !out.status.success() {
        io::stdout().write_all(&out.stdout).unwrap();
        io::stderr().write_all(&out.stderr).unwrap();
        exit(0);
    }

    if !run_type {
        println!(
            "    {} {build_t} [{target}] target(s) in {:?}",
            "Finished".bright_blue().bold(),
            start.elapsed()
        );
    }

    if i {
        #[cfg(windows)]
        install(format!("{}.exe", cppm.project["name"]));
        #[cfg(unix)]
        install(format!("{}", cppm.project["name"]));
    }

    let cc = fs::canonicalize(cppm.project["src"].clone())
        .unwrap()
        .as_os_str()
        .to_str()
        .unwrap()
        .to_string();

    #[cfg(windows)]
    let cc = cc.trim()[4..].to_string();

    crate::templates::compile_commands(
        canc.clone(),
        cc,
        compiler.clone().to_string(),
        cppm.project["name"].clone(),
        includes.clone(),
        flags.clone().join(" "),
    );
}
/// #### Run function.
/// Handles building and piping extra arguments to the executable.
pub fn run(release: bool, run_type: bool, c: bool, extra_args: Vec<String>) {
    if !Path::new("Cppm.toml").exists() {
        println!("Cppm project isnt in current directory!");
        exit(0);
    }
    if !Path::new(&defaults_file()).exists() {
        println!(
            "{}",
            "You haven't set up your defaults yet! Run `cppm --config` to resolve this error."
                .red()
        );
        exit(0);
    }
    let cppm: LocalConfig = toml::from_str(&read_to_string("Cppm.toml").unwrap()).unwrap();

    build(release, run_type, false, c);

    #[cfg(windows)]
    let name = format!("build/{}.exe", cppm.project["name"]);
    #[cfg(unix)]
    let name = format!("build/{}", cppm.project["name"]);

    let canc: String = std::fs::canonicalize(name)
        .unwrap()
        .as_os_str()
        .to_str()
        .unwrap()
        .to_owned();

    #[cfg(windows)]
    let canc = canc.trim()[4..].to_string();
    
    println!(
        "     {} `{} {}`",
        "Running".bright_blue().bold(),
        canc,
        extra_args.join(" ")
    );

    let run = format!("build/{}", cppm.project["name"]);
    let out = Command::new(run).args(extra_args).output().unwrap();
    io::stdout().write_all(&out.stdout).unwrap();
    io::stderr().write_all(&out.stderr).unwrap();
}

/// Default configuration file location.
fn defaults_file() -> String {
    let defaultsdir = dirs::home_dir()
        .unwrap()
        .into_os_string()
        .into_string()
        .unwrap()
        .replace('"', "")
        .replace('\\', "/");
    format!("{}/.cppm/defaults.toml", defaultsdir)
}

pub fn install(exe: String) {
    let exe_dir = dirs::home_dir()
        .unwrap()
        .into_os_string()
        .into_string()
        .unwrap()
        .replace('"', "")
        .replace('\\', "/");
    if !Path::new(&format!("{}/.cppm/bin/{}", exe_dir, exe)).exists() {
        fs::File::create(&format!("{}/.cppm/bin/{}", exe_dir, exe)).unwrap();
    }
    fs::copy(
        format!("build/{}", exe.clone()),
        format!("{}/.cppm/bin/{}", exe_dir, exe),
    )
    .expect("could not move file");
}
