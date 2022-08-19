mod build;
mod clangd;
mod cppm;
mod dependencies;
mod templates;
use clap::{Parser, Subcommand};
use colored::Colorize;
use cppm::*;
use human_panic::setup_panic;
use std::{env, vec};

/// Argument struct for clap. All flags go here.
#[derive(Parser)]
#[clap(about, version)]
struct Args {
    /// Lists all projects configured with cppm
    #[clap(short, long)]
    list: bool,

    /// Record current directory if its recognized as a cppm project.
    #[clap(long)]
    record: bool,

    //note: add makefile flag, for cppm use only generating files
    /// Configure cppm defaults
    #[clap(long)]
    config: bool,

    /// Compare your current cppm version to the most recent version
    #[clap(short = 's', long)]
    status: bool,

    /// Clean up build
    #[clap(long)]
    clean: bool,

    /// View and Edit cppm config file
    #[clap(long)]
    toml: bool,

    /// Create a project without initializing an empty git repo
    #[clap(long = "no-git")]
    git: bool,

    /// Remove a cppm project
    #[clap(short, long)]
    remove: Option<String>,

    /// Run or Build without compilation messages
    #[clap(short, long)]
    quiet: bool,

    #[clap(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Clone)]
enum Command {
    /// Open a cppm project
    Open {
        name: String,
        editor: Option<String>,
        #[clap(allow_hyphen_values = true)]
        flags: Vec<String>,
    },
    /// Creates a new cppm project
    New {
        name: String,
        editor: Option<String>,
        /// Generate C files instead of C++ files
        #[clap(short)]
        c: bool,

        /// Enable cppm generated clangd linting
        #[clap(long)]
        clangd: bool,
    },
    /// Initialize a new cppm project to an existing git repo
    GitInit {
        /// Github Username
        name: String,
        /// Repository to initialize to
        repo: String,
    },
    /// Initialize a cppm project in current directory
    Init {
        /// Generate C files instead of C++ files
        #[clap(short)]
        c: bool,

        /// Enable cppm generated clangd linting
        #[clap(long)]
        clangd: bool,
    },
    /// Build your project to build directory.
    Build {
        // Build your project in release mode
        #[clap(long)]
        release: bool,

        /// Build the project in c mode
        #[clap(long)]
        c: bool,

        // Install your project to the system
        #[clap(long)]
        install: bool,
    },
    /// Clean current build
    Clean,
    /// Run your project
    Run {
        #[clap(long)]
        release: bool,

        /// Build the project in c mode
        #[clap(long)]
        c: bool,

        #[clap(allow_hyphen_values = true)]
        extra_args: Vec<String>,
    },
    /// Constantly watch a file for changes and build/run when changes are detected
    Watch { filename: Option<String> },

    /// Format files in project
    Format,
    /// Lints and shows error checks for the project
    Clint {
        /// Default is src/main.cpp
        source: Option<String>,

        /// Lint C files
        #[clap(long)]
        c: bool,
    },
}

fn main() {
    setup_panic!(); // set up human panic
    let args = Args::parse();

    // Bool Checks
    if args.list {
        cppm::list_projects()
    }
    if args.config {
        cppm::defaults()
    }
    if args.status {
        Cppm::status();
    }
    if args.clean {
        Cppm::clean()
    }
    if args.toml {
        cppm::toml();
    }
    if args.record {
        cppm::record();
    }

    if let Some(remove) = args.remove {
        cppm::remove(remove);
    }

    // Command impls
    match args.command {
        Some(Command::Open {
            name,
            editor,
            flags,
        }) => {
            if editor.is_some() {
                Cppm::open(name, editor, flags);
            } else {
                Cppm::open(name, None, vec![]);
            }
        }
        Some(Command::New {
            name,
            editor,
            c,
            clangd,
        }) => {
            if c {
                Cppm::spawn(
                    name.clone(),
                    editor.unwrap_or_else(|| "null".to_string()),
                    "c",
                );
                println!("    {} C project `{}`", "Created".bright_green(), name);
            } else {
                Cppm::spawn(
                    name.clone(),
                    editor.unwrap_or_else(|| "null".to_string()),
                    "cpp",
                );
                println!("    {} C++ project `{}`", "Created".bright_green(), name);
            }
            if !args.git && cppm::git_exists() {
                env::set_current_dir(name.clone()).unwrap();
                cppm::git_init();
            }
            if clangd {
                clangd::create();
            }
        }
        Some(Command::GitInit { name, repo }) => {
            Cppm::init_existing(name, repo);
            println!("{}", "Project initialized successfully".bright_green());
        }

        Some(Command::Init { c, clangd }) => {
            if c {
                Cppm::initialize("c").ok(); // warning: change this to a bool cuz this is dumb
            } else {
                Cppm::initialize("cpp").ok();
            }
            if !args.git && cppm::git_exists() {
                cppm::git_init();
            }
            if clangd {
                clangd::create();
            }
        }
        Some(Command::Build {
            release,
            install,
            c,
        }) => build::build(release, args.quiet, install, c),
        Some(Command::Clean) => {
            Cppm::clean();
        }
        Some(Command::Watch { filename }) => {
            Cppm::watch((filename).unwrap_or("src/main.cpp".to_string()));
        }
        // note: pass extra args through run
        Some(Command::Run {
            release,
            extra_args,
            c,
        }) => {
            build::run(release, args.quiet, c, extra_args);
        }
        Some(Command::Format) => {
            clangd::format();
        }
        Some(Command::Clint { source, c }) => {
            clangd::clint(source, c);
        }
        None => (),
    }
}
