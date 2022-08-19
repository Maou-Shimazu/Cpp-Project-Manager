#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct CCJson {
    directory: String,
    file: String,
    command: String,
}

pub fn compile_commands(
    dir: String,
    src: String,
    compiler: String,
    name: String,
    include: Vec<&str>,
    flags: String,
) {
    let incl = include.join(" ");
    let json: CCJson = CCJson {
        directory: dir,
        file: src.clone(),
        command: format!("{compiler} {src} -o build/{name} {incl} {flags}"),
    };
    if !std::path::Path::new("build/compile_commands.json").exists() {
        std::fs::File::create("build/compile_commands.json")
            .expect("could not create compile_commands.json");
    }

    std::fs::write(
        "build/compile_commands.json",
        serde_json::to_string_pretty(&json).unwrap().as_bytes(),
    )
    .expect("could not write to compile_commands.json");
}
/// clangd file template
pub fn clangd() -> String {
    let cppm: crate::build::LocalConfig =
        toml_edit::de::from_str(&std::fs::read_to_string("Cppm.toml").unwrap()).unwrap();
    let includes: Vec<&str> = cppm.project["include"].split(", ").collect();
    let cd: String = format!(
"
CompileFlags:                   
  Add: [-xc++, -Wall, -D, NAME=\"{}\", -D, VERSION=\"{}\", -D, EDITION=\"{}\",  -std=c++17, -fdiagnostics-color=always, -Wpedantic, -Werror, -Wshadow, -Wformat=2, -Wconversion, -Wunused-parameter, -I../{}]

Diagnostics:
  UnusedIncludes: None #Possible values: None, Strict
  ClangTidy: # Checklist can be found here: https://clang.llvm.org/extra/clang-tidy/checks/list.html
    Add: modernize*
    Remove: [hicpp-braces-around-statements, modernize-use-trailing-return-type]

Hover:
   ShowAKA: Yes

InlayHints:
  Enabled: No
  ParameterNames: No
  DeducedTypes: No
", cppm.project["name"], cppm.project["version"], cppm.project["edition"], includes.join("-I../ ")
    );
    cd
}

/// clang format file template
pub static CLANG_FORMAT: &str = r#"
BasedOnStyle: LLVM
IndentWidth: 4
TabWidth: 4
UseTab: Always

AllowShortIfStatementsOnASingleLine: true
NamespaceIndentation: All
Language: Cpp
DerivePointerAlignment: false
PointerAlignment: Left

AccessModifierOffset: -4
AlignAfterOpenBracket: true
AlignConsecutiveAssignments: true
AlignConsecutiveDeclarations: false
AlignEscapedNewlinesLeft: false
AlignOperands:   true
AlignTrailingComments: false
AllowAllParametersOfDeclarationOnNextLine: true
AllowShortBlocksOnASingleLine: false
AllowShortCaseLabelsOnASingleLine: false
AllowShortFunctionsOnASingleLine: Empty
AllowShortIfStatementsOnASingleLine: false
AllowShortLoopsOnASingleLine: false
AlwaysBreakAfterDefinitionReturnType: false
AlwaysBreakAfterReturnType: None
AlwaysBreakBeforeMultilineStrings: false
AlwaysBreakTemplateDeclarations: true
BinPackArguments: false
BinPackParameters: false

BraceWrapping: {
  AfterClass: 'true'
  AfterControlStatement: 'true'
  AfterEnum : 'true'
  AfterFunction : 'true'
  AfterNamespace : 'true'
  AfterStruct : 'true'
  AfterUnion : 'true'
  BeforeCatch : 'true'
  BeforeElse : 'true'
  IndentBraces : 'false'
  AfterExternBlock : 'true'
  SplitEmptyFunction : 'false'
  SplitEmptyRecord : 'false'
  SplitEmptyNamespace : 'true'
}
"#;

/// clang tidy file template
pub static CLANG_TIDY: &str = r#"
Checks:              '-checks=-*,clang-analyzer-*,-clang-analyzer-cplusplus*'
WarningsAsErrors:    true
HeaderFilterRegex:   '.*'
FormatStyle:         LLVM
InheritParentConfig: true
User:                user
CheckOptions:
  - { key: readability-identifier-naming.VariableCase,        value: lower_case }
  - { key: readability-identifier-naming.GlobalConstantCase,  value: UPPER_CASE }
  - { key: readability-identifier-naming.FunctionCase,        value: lower_case }
"#;

pub const CBOILER: &str = r#"
#include <stdio.h>

int main(void) {
    printf("Hello World!\n");
    return 0;
}
"#;

/// c headefile boilerplate
pub fn header_boiler_c(header_name: &str) -> String {
    format!(
        r#"#ifndef {0}_H
#define {0}_H

#include <stdio.h>


#endif"#,
        header_name.to_uppercase().replace('-', "_")
    )
}

/// c++ main file boilerplate
pub const CPPBOILER: &str = r#"#include <iostream>

int main(){

    std::cout << "Hello World" << std::endl;
    return 0;
}
"#;

/// C++ header file boilerplate.
pub fn header_boiler(header_name: &str) -> String {
    format!(
        r#"#pragma once

#ifndef {0}_HPP
#define {0}_HPP

#include <iostream>


#endif"#,
        header_name.to_uppercase().replace('-', "_")
    )
}

pub fn cppm_toml_template(name: &String, src: &str) -> String {
    format!(r#"[project]
name = "{name}"
version = "1.0.0"
edition = "2022"
include = "include"
src = "{src}"
standard = "17"

[dependencies]"#)
}
