use crate::build::LocalConfig;
use git2::Repository;
use serde::{Deserialize, Serialize};
use std::{io::Write, process};

#[derive(Serialize, Deserialize, Debug)]
struct LC {
    package_name: String,
    package_includes: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct PackageIncludes {
    package: Vec<LC>,
}
impl PackageIncludes {
    pub fn new() -> PackageIncludes {
        PackageIncludes { package: vec![] }
    }
}

pub fn regloc() -> String {
    let configdir = dirs::home_dir()
        .unwrap()
        .into_os_string()
        .into_string()
        .unwrap()
        .replace('"', "")
        .replace('\\', "/");
    format!("{}/.cppm/registry", configdir)
}

pub fn read_deps(includes: Vec<&str>) -> Vec<String> {
    let mut includes: Vec<String> = includes.iter().map(|x| x.to_string()).collect();
    let cppm: LocalConfig =
        toml_edit::de::from_str(&std::fs::read_to_string("Cppm.toml").unwrap()).unwrap();
    if !cppm.dependencies.is_empty() {
        for (key, value) in cppm.dependencies {
            if !std::path::Path::new("Cppm.lock").exists() {
                std::fs::File::create("Cppm.lock").expect("Could not create lock file.");
                process::exit(0);
            }
            let p = format!("{}/{}/{key}/Cppm.toml", &regloc(), cppm.project["name"]);
            let loc = &format!("{}/{}/{key}", &regloc(), cppm.project["name"]);
            if !std::path::Path::new(loc).exists() {
                match Repository::clone(&value, loc) {
                    Ok(_) => (),
                    Err(e) => {
                        println!("Error Occured Cloning Dependency {key}: `{e}`");
                    }
                }
                let canc: String = std::fs::canonicalize(loc)
                    .unwrap()
                    .as_os_str()
                    .to_str()
                    .unwrap()
                    .to_string();

                #[cfg(windows)]
                let canc = canc.replace('\\', "\\").trim()[4..].to_string();

                let llc: LocalConfig =
                    toml_edit::de::from_str(&std::fs::read_to_string(p.clone()).unwrap())
                        .expect("Dependency isnt a cppm project");
                let local_includes: Vec<String> = llc.project["include"]
                    .split(", ")
                    .into_iter()
                    .map(|f| f.to_string())
                    .collect();
                let local_includes: String =
                    format!("-I{canc}/{}", local_includes.join(&format!(" -I{canc}/")));
                let mut local_includes: Vec<String> = local_includes
                    .split(" ")
                    .into_iter()
                    .map(|x| x.to_string())
                    .collect();
                includes.append(&mut local_includes);

                let mut config: PackageIncludes;
                if !std::fs::read_to_string(format!("Cppm.lock"))
                    .unwrap()
                    .is_empty()
                {
                    config = toml_edit::de::from_str(
                        &std::fs::read_to_string(format!("Cppm.lock")).unwrap(),
                    )
                    .unwrap();
                } else {
                    config = PackageIncludes::new();
                }

                let imp: LC = LC {
                    package_name: llc.project["name"].clone(),
                    package_includes: includes.clone(),
                };
                config.package.push(imp);
                let result = toml_edit::ser::to_string(&config).unwrap();
                let mut file = std::fs::OpenOptions::new()
                    .write(true)
                    .append(true)
                    .open("Cppm.lock")
                    .unwrap();
                file.write_all(result.as_bytes())
                    .expect("Could not write to lock");
            } else {
                let pi: PackageIncludes;
                if !std::fs::read_to_string(format!("Cppm.lock"))
                    .unwrap()
                    .is_empty()
                {
                    pi = toml_edit::de::from_str(
                        &std::fs::read_to_string(format!("Cppm.lock")).unwrap(),
                    )
                    .unwrap();
                } else {
                    pi = PackageIncludes::new();
                }
                for mut i in pi.package {
                    includes.append(&mut i.package_includes);
                }
            }
        }
    }
    return includes;
}
