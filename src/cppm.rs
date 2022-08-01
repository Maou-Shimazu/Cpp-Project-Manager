#![allow(dead_code)]
pub mod builder;

use std::{
    collections::HashMap,
    fs::{self, File},
    io::Write,
    path::Path,
    process::{self, Command},
};

use colored::Colorize;
use serde::{Deserialize, Serialize};

use crate::build::run;

// Imports for `cppm --status`
use serde_json::Value;
use std::str;

/// Struct used to serialize configfile.
#[derive(Serialize, Deserialize, Debug)]
struct Config {
    name: String,
    location: String,
}

/// writes project location to config file
pub fn write(project_name: &str, location: &str) {
    if !Path::new(&configfile()).exists() {
        File::create(&configfile()).expect("couldnt create config file");
    }

    let config: Config = Config {
        name: project_name.to_string(),
        location: location.replace('/', "\\"),
    };
    fs::OpenOptions::new()
        .write(true)
        .append(true)
        .open(&configfile())
        .unwrap()
        .write_all(format!("\n[[config]]\n{}", toml::to_string_pretty(&config).unwrap()).as_bytes())
        .expect("couldnt write to config file");
}

/// gets the name of the current directory
pub fn dir_name() -> String {
    std::path::Path::new(&std::env::current_dir().unwrap())
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string()
}

/// ~/.cppm/config.toml
pub fn configfile() -> String {
    let configdir = dirs::home_dir()
        .unwrap()
        .into_os_string()
        .into_string()
        .unwrap()
        .replace('"', "")
        .replace('\\', "/");
    format!("{}/.cppm/config.toml", configdir)
}

/// lists all the projects in configfile
pub fn list_projects() {
    if !Path::new(&configfile()).exists() {
        println!("{}", "You have not created any projects yet!".red());
        std::process::exit(0);
    }
    let config: HashMap<String, Vec<Config>> =
        toml::from_str(&std::fs::read_to_string(configfile()).unwrap()).unwrap();
    let items: &[Config] = &config["config"];
    print!("\nProjects configured with cppm: \n");
    for i in items {
        println!("{}: {}", i.name, i.location);
    }
}

/// Main struct
pub struct Cppm {
    project_name: String,
    editor: String,
}

impl Cppm {
    /// cppm init
    fn init() -> Cppm {
        Cppm {
            project_name: String::new(),
            editor: String::new(),
        }
    }
    /// spawns a new project and writes its location to config file
    // note: possibly integrate git_init
    pub fn spawn(_project_name: String, editor: String, init_type: &str) {
        if Path::new(&_project_name).exists() {
            println!("{}", "Folder with the same name already exists!".red());
            process::exit(0);
        }

        let local_open = |s: &Cppm, pn: String| {
            if !s.editor.contains("null") {
                let mut child = if cfg!(target_os = "windows")
                    || cfg!(target_os = "linux")
                    || cfg!(target_os = "unix")
                {
                    Command::new(s.editor.clone())
                        .arg(pn)
                        .spawn()
                        .expect("Failed to open editor.")
                } else {
                    println!(
                        "{}",
                        "Your OS is not supported, please open an issue to get it implemented."
                            .red()
                    );
                    return;
                };
                child.wait().expect("Failed to wait on process.");
            }
        };

        if init_type == "c" {
            let mut s = Cppm::init();
            s.project_name = _project_name;
            let pn = s.project_name.clone();
            s.editor = editor;
            fs::create_dir_all(s.project_name.clone()).expect("Folder creation failed.");
            fs::create_dir_all(format!("{}/src", s.project_name)).expect("Folder creation failed.");
            fs::create_dir_all(format!("{}/include", s.project_name))
                .expect("Folder creation failed.");

            // switch to closure
            let files = |s: &Cppm| -> (String, String) {
                let main: String = format!("{}/src/main.c", s.project_name);
                let header: String = format!("{0}/include/{0}.h", s.project_name);
                (main, header)
            };

            let (main, header) = files(&s);
            let main_path = Path::new(main.as_str());
            let header_path = Path::new(header.as_str());

            File::create(&main_path)
                .expect("File creation failed.")
                .write_all(crate::templates::CBOILER.as_bytes())
                .expect("Failed to write to main file.");
            File::create(&header_path)
                .expect("Unable to create file or file already exists.")
                .write_all(crate::templates::header_boiler_c("main").as_bytes())
                .expect("Unable to write to file.");

            Cppm::cppm_toml(
                &format!("{}/{}", std::env::current_dir().unwrap().display(), pn)
                    .replace('\\', "/"),
                init_type == "c",
            );
            write(
                pn.as_str(),
                &format!("{}/{}", std::env::current_dir().unwrap().display(), pn),
            );
            local_open(&s, pn.clone());

        } else {
            let mut s = Cppm::init();
            s.project_name = _project_name;
            let pn = s.project_name.clone();
            s.editor = editor;
            fs::create_dir_all(s.project_name.clone()).expect("Folder creation failed.");
            fs::create_dir_all(format!("{}/src", s.project_name)).expect("Folder creation failed.");
            fs::create_dir_all(format!("{}/include", s.project_name))
                .expect("Folder creation failed.");

            let (main, header) = path(&s);
            let main_path = Path::new(main.as_str());
            let header_path = Path::new(header.as_str());

            File::create(&main_path)
                .expect("File creation failed.")
                .write_all(crate::templates::CPPBOILER.as_bytes())
                .expect("Failed to write to main file.");
            File::create(&header_path)
                .expect("File creation failed.")
                .write_all(crate::templates::header_boiler(pn.as_str()).as_bytes())
                .expect("failed to write to header file.");

            Cppm::cppm_toml(
                &format!("{}/{}", std::env::current_dir().unwrap().display(), pn)
                    .replace('\\', "/"),
                init_type == "c",
            );
            write(
                pn.as_str(),
                &format!("{}/{}", std::env::current_dir().unwrap().display(), pn),
            );

            local_open(&s, pn.clone());
        }
    }

    /// reads config file and opens a project based on the name provided
    // note: add aliases for known editors
    pub fn open(_project_name: String, editor: Option<String>, flags: Vec<String>) {
        if !Path::new(&configfile()).exists() {
            println!("{}", "You have not created any projects yet!".red());
            process::exit(0);
        }

        let editor = match editor {
            Some(val) => val,
            None => {
                if !Path::new(&defaults_file()).exists() {
                    println!("{}", "You haven't configured a default editor yet! Run `cppm --config` to resolve this error.".red());
                    process::exit(0);
                }
                let contents: Def =
                    toml::from_str(&fs::read_to_string(defaults_file()).unwrap()).unwrap();
                if contents.editor == "null" {
                    println!("{}", "You haven't configured a default editor yet!".red());
                    process::exit(0);
                }
                contents.editor
            }
        };

        if builder::subprocess(&editor, "--version").is_err() {
            println!(
                "    {}",
                "Editor does not exist or cannot be opened with the argument passed.".red()
            );
            process::exit(0);
        }
        let toml_config: HashMap<String, Vec<Config>> =
            toml::from_str(&fs::read_to_string(configfile()).unwrap()).unwrap();
        let config: &[Config] = &toml_config["config"];
        let mut b = false;
        for i in config {
            if i.name == _project_name {
                let project_location = i.location.clone();
                println!(
                    "   Opening Project `{}`: {}",
                    _project_name.green(),
                    project_location
                );

                let mut editor = if cfg!(target_os = "windows") {
                    Command::new(editor.clone())
                        .args(flags.clone())
                        .arg(project_location)
                        .spawn()
                        .expect("Failed to open editor.")
                } else {
                    println!(
                        "{}",
                        "Your OS is not supported, please open an issue to get it implemented."
                            .red()
                    );
                    return;
                };
                b = true;
                editor.wait().expect("Failed to wait on process.");
            }
        }
        if !b {
            println!(
                "    {}",
                "Project does not exist or was not created with cppm!".red()
            );
        }
    }

    /// removes the build directory
    pub fn clean() {
        if !Path::new("build").exists() {
            println!("{}", "Project has not been built!".red());
            process::exit(0);
        } else {
            fs::remove_dir_all("build").ok();
        }
    }

    /// watchhes a file and compiles if it is changed
    pub fn watch(file: String) {
        let mut original_contents = fs::read_to_string(&file)
            .expect("That file either doesn't exist or cannot be accessed!");

        loop {
            let contents = fs::read_to_string(&file)
                .expect("That file either doesn't exist or cannot be accessed!");

            if contents != original_contents && !contents.is_empty() {
                // `contents != ""` is a fix for a bug with VSC - it does not impair normal usage
                original_contents = contents;
                run(false, true, false, vec![]);
            }
        }
    }

    /// check version status of cppm gh release and package version
    pub fn status() {
        let result = minreq::get(
            "https://api.github.com/repos/cpp-project-manager/cpp-project-manager/releases/latest",
        )
        .with_header("User-Agent", "cppm")
        .send()
        .unwrap();

        let result_str = result.as_str().unwrap();

        let json_value: Value = serde_json::from_str(result_str).unwrap();
        let latest = &json_value["tag_name"];

        println!(
            "Installed version: cppm \"{}\" - Latest version: cppm {}",
            env!("CARGO_PKG_VERSION"), latest
        )
    }

    /// initialize the cwd to the github repo provided
    pub fn init_existing(name: String, mut repo: String) {
        // TODO: Add nice error messages
        repo = format!("https://github.com/{name}/{repo}.git");
        Command::new("git")
            .arg("init")
            .output()
            .expect("An error initializing Git - Make sure you have Git installed and try again!");
        Command::new("git")
            .arg("add")
            .arg("-A")
            .output()
            .expect("An error occurred while trying to commit changes to Git - Make sure you have Git installed and try again");
        Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg("Initial commit")
            .output()
            .expect("An error occured while trying to set the branch to main - Make sure you have Git installed and try again");
        Command::new("git")
            .arg("branch")
            .arg("-M")
            .arg("main")
            .output()
            .expect("An error occured while trying to set the branch to main - Make sure you have Git installed and try again");
        Command::new("git")
            .arg("remote")
            .arg("add")
            .arg("origin")
            .arg(repo)
            .output()
            .expect("An error occurred while trying to connect to the remote repository - Make sure the repository exists and try again");
        Command::new("git")
            .arg("push")
            .arg("-u")
            .arg("origin")
            .arg("main")
            .output()
            .expect("An error occurred while trying to push changes to the repository - Make sure the repository exists and try again");
    }

    /// Initializes a project in the current directory.
    pub fn initialize(init_type: &str) -> std::io::Result<()> {
        if Path::new("src").exists() || Path::new("include").exists() {
            println!(
                "    {}",
                "You already have a project in this directory!".red()
            );
            process::exit(0);
        }
        if init_type == "c" {
            fs::create_dir_all("src").expect("Folder creation failed or folder already exists.");
            fs::create_dir_all("include")
                .expect("Folder creation failed or folder already exists.");
            File::create("src/main.c")
                .expect("Folder creation failed or folder already exists.")
                .write_all(crate::templates::CBOILER.as_bytes())
                .expect("Unable to write to file.");
            File::create("include/main.h")
                .expect("Unable to create file or file already exists.")
                .write_all(crate::templates::header_boiler_c("main").as_bytes())
                .expect("Unable to write to file.");

            write(
                dir_name().as_str(),
                std::env::current_dir()?.as_os_str().to_str().unwrap(),
            );
            Cppm::cppm_toml(
                std::env::current_dir()?.as_os_str().to_str().unwrap(),
                init_type == "c",
            );
            Ok(())
        } else {
            fs::create_dir_all("src").expect("Folder creation failed or folder already exists.");
            fs::create_dir_all("include")
                .expect("Folder creation failed or folder already exists.");
            File::create("include/main.hpp")
                .expect("Unable to create file or file already exists.")
                .write_all(crate::templates::header_boiler("main").as_bytes())
                .expect("Unable to write to file.");
            File::create("src/main.cpp")
                .expect("Folder creation failed or folder already exists.")
                .write_all(crate::templates::CPPBOILER.as_bytes())
                .expect("Unable to write to file.");

            write(
                dir_name().as_str(),
                std::env::current_dir()?.as_os_str().to_str().unwrap(),
            );
            Cppm::cppm_toml(
                std::env::current_dir()?.as_os_str().to_str().unwrap(),
                init_type == "c",
            );
            Ok(())
        }
    }
    /// Cppm.toml boilerplate
    // note: implement libs
    pub fn cppm_toml(loc: &str, c: bool) {
        let __loc__ = std::path::Path::new(loc)
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        let mut cc: crate::build::LocalConfig = crate::build::LocalConfig {
            project: HashMap::from([
                ("name".to_string(), __loc__),
                ("version".to_string(), "1.0.0".to_string()),
                ("edition".to_string(), "2022".to_string()),
                ("include".to_string(), "include".to_string()),
                ("src".to_string(), "src/main.cpp".to_string()),
                ("standard".to_string(), "17".to_string()),
            ]),
            dependencies: HashMap::new(),
        };
        if c {
            cc.project
                .insert("src".to_string(), "src/main.c".to_string());
        }

        fs::write(
            &format!("{}/Cppm.toml", loc),
            toml::to_string(&cc).unwrap().as_bytes(),
        )
        .expect("Unable to write to file.");
    }
}

// returns a tuple of src and include files
fn path(s: &Cppm) -> (String, String) {
    let main: String = format!("{}/src/main.cpp", s.project_name);
    let header: String = format!("{0}/include/{0}.hpp", s.project_name);
    (main, header)
}

/// removes a project who's location is in configfile
// note: find a way to implement removing from the config file
pub fn remove(project_name: String) {
    let toml_config: HashMap<String, Vec<Config>> =
        toml::from_str(&fs::read_to_string(configfile()).unwrap()).unwrap();
    let config: &[Config] = &toml_config["config"];
    let project = config.iter().find(|p| p.name == project_name);
    if project.is_none() {
        println!("Project does not exist or was not created with cppm!");
        process::exit(1);
    }
    let project_location = project.unwrap().location.clone();
    println!(
        "   Removing Project `{}`: {}",
        project_name.green(),
        project_location
    );
    fs::remove_dir_all(project_location).expect("Failed to remove project.");
    process::exit(0);
}

/// Defaults file serializer
#[derive(Debug, Deserialize, Serialize)]
pub struct Def {
    editor: String,
    compilers: HashMap<String, String>,
}
impl Def {
    pub fn new() -> Def {
        Def {
            editor: String::new(),
            compilers: HashMap::new(),
        }
    }
}

/// returns defaults file location
pub fn defaults_file() -> String {
    let defaultsdir = dirs::home_dir()
        .unwrap()
        .into_os_string()
        .into_string()
        .unwrap()
        .replace('"', "")
        .replace('\\', "/");
    format!("{}/.cppm/defaults.toml", defaultsdir)
}

/// constructs defaults file and its contents
use std::io::{stdin, stdout};
pub fn defaults() {
    let mut config: Def = Def::new();
    fs::File::create(&defaults_file()).expect("Could not create defaults file.");

    print!("Default editor [ENTER `null` FOR NO DEFAULT EDITOR]: ");
    stdout().flush().ok();
    stdin()
        .read_line(&mut config.editor)
        .expect("Failed to read line");
    config.editor = config.editor.trim().to_string();

    let c = builder::c();
    let cpp = builder::cpp();

    match c {
        Ok(x) => match x {
            builder::Compilers::Clang => {
                config
                    .compilers
                    .insert("c".to_string(), "clang".to_string());
            }
            builder::Compilers::Gcc => {
                config.compilers.insert("c".to_string(), "gcc".to_string());
            }
            _ => (),
        },
        Err(e) => println!("{}", e),
    }

    match cpp {
        Ok(x) => match x {
            builder::Compilers::Clangpp => {
                config
                    .compilers
                    .insert("cpp".to_string(), "clang++".to_string());
            }
            builder::Compilers::Gpp => {
                config
                    .compilers
                    .insert("cpp".to_string(), "g++".to_string());
            }
            _ => (),
        },
        Err(e) => println!("{}", e),
    }

    fs::write(
        &defaults_file(),
        toml::to_string(&config).unwrap().as_bytes(),
    )
    .expect("Unable to write to file.");

    println!("Location: {}", defaults_file());
}

/// show the location of configfile
// warning: file dosent spawn properly
pub fn toml() {
    println!("location: {}", configfile());
    #[cfg(windows)]
    Command::new("powershell")
        .arg(configfile())
        .spawn()
        .expect("Couldn't open config file");
    #[cfg(unix)]
    Command::new(configfile())
        .spawn()
        .expect("Couldn't open config file");
    // check on linux, dosent work on wsl
}

/// git initialization
pub fn git_init() {
    Command::new("git")
        .arg("init")
        .stdout(process::Stdio::null())
        .spawn()
        .expect("Couldn't start git.");

    let git = r#"# Generated by cppm
*.exe
*.obj
*.o
/build
        "#;
    fs::write(
        &format!(
            "{}/.gitignore",
            std::env::current_dir()
                .unwrap()
                .as_os_str()
                .to_str()
                .unwrap()
        ),
        git.as_bytes(),
    )
    .expect("Unable to write to file.");
}
/// check if git exists
pub fn git_exists() -> bool {
    builder::subprocess("git", "--version").is_ok()
}
