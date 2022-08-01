use std::process::*;
/// Compilers enum
/// note: look into support for cppm and msvc
pub enum Compilers {
    Gcc,
    Clang,
    Msvc,
    Gpp,
    Clangpp,
}

/// Returns a result of the processed params. Used to determine if a program exists or not. \
/// usage:
/// ```rs
/// if crate::cppm::builder::subprocess("clang-format", "--version").is_err() {
///     println!(
///         "   {}",
///         "Could not run clang-format. Please ensure that it is installed and on your PATH."
///         .bright_red()
///     );
///     return;
/// }
/// ```
pub fn subprocess(process: &str, arg: &str) -> std::io::Result<ExitStatus> {
    Command::new(process)
        .arg(arg)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
}

/// Returns either clang or gcc enum depending on existing one. Clang takes presedence.
pub fn c() -> std::io::Result<Compilers> {
    if subprocess("clang", "-v").is_ok() {
        println!("Clang is installed.");
        Ok(Compilers::Clang)
    } else if subprocess("gcc", "-v").is_ok() {
        println!("Gcc is installed.");
        Ok(Compilers::Gcc)
    } else {
        println!("Neither Gcc or Clang is installed.");
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Neither Gcc or Clang is installed.",
        ))
    }
}

/// Returns either clang++ or g++ enum depending on existing one. Clang++ takes presedence.
pub fn cpp() -> std::io::Result<Compilers> {
    if subprocess("clang++", "-v").is_ok() {
        println!("Clang++ is installed.");
        Ok(Compilers::Clangpp)
    } else if subprocess("g++", "-v").is_ok() {
        println!("G++ is installed.");
        Ok(Compilers::Gpp)
    } else {
        println!("Neither G++ or Clang++ is installed.");
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Neither G++ or Clang++ is installed.",
        ))
    }
}
