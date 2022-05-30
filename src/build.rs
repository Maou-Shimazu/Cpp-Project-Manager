use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{self, read_to_string},
    path::Path,
    process::*,
    str,
};

#[derive(Serialize, Deserialize, Debug)]
struct Build {
    project: HashMap<String, String>,
}

pub fn include(folders: Vec<&str>) -> String {
    let mut return_string: Vec<&str> = Vec::new();
    for i in folders {
        return_string.push(i);
    }
    format!("-I../{}", return_string.join(" -I../"))
}

#[allow(dead_code)]
pub fn sources() {}

#[allow(dead_code)]
pub fn compiler() {}
#[allow(dead_code)]
pub fn lib() {}

// note: dynamically "include"
// note: add `flags_all = bool`, `flags = ""`
// note: look for a logger lib
// note: optimize for smart object building headerfiles in the future
// warning: dont forget linux support
pub fn build() {
    if !Path::new("Cppm.toml").exists() {
        println!("Cppm project isnt in current directory!");
        exit(0);
    }
    let cppm: Build = toml::from_str(&read_to_string("Cppm.toml").unwrap()).unwrap();
    let includes: Vec<&str> = cppm.project["include"].split(",").collect();
    let src = cppm.project["src"].clone();

    let build = format!(
        "g++ {} {src} -o build/{}",
        include(includes.clone()),
        cppm.project["name"].clone()
    );
    //println!("{}", build.clone());

    fs::create_dir_all("build").ok();
    let out = Command::new("powershell").arg(build).output().unwrap();
    println!("{}", &out.status);
    let out = str::from_utf8(&out.stdout).unwrap();
    println!("{}", out);

    //println!("{}", include(includes));
}

// warning: get output and print to console
pub fn run() {
    let cppm: Build = toml::from_str(&read_to_string("Cppm.toml").unwrap()).unwrap();
    build();
    let run = format!("build/{}.exe", cppm.project["name"]);
    let out = Command::new("powershell")
        .arg(run.clone())
        .output()
        .unwrap();
    let out = str::from_utf8(&out.stdout).unwrap();
    println!("{}", out.trim());
}
