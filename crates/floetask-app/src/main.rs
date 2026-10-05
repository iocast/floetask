//! Composition root: parses the command line, builds the infrastructure
//! adapters and hands the resulting services to the GUI.
//!
//! This is the only crate that knows every layer.

// Release builds are GUI apps on Windows: no console window opens next to
// the app. Debug builds keep the console for logs and panics.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;

use floetask_application::Services;
use floetask_infrastructure::AppPaths;

/// A todo.txt manager.
#[derive(Debug, Parser)]
#[command(version, about)]
struct Cli {
    /// Config file to use instead of ~/.config/floetask/config.toml.
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Print where floetask keeps its files and exit.
    #[arg(long)]
    paths: bool,

    /// A todo.txt file to open (and register) on start.
    todo_file: Option<PathBuf>,
}

fn main() -> ExitCode {
    attach_parent_console();
    let cli = Cli::parse();
    let paths = AppPaths::resolve(cli.config);

    if cli.paths {
        println!("config:  {}", paths.config_file.display());
        println!("colors:  {}", paths.colors_file().display());
        println!("filters: {}", paths.filters_file().display());
        println!("state:   {}", paths.state_file().display());
        println!("notified: {}", paths.notified_file().display());
        return ExitCode::SUCCESS;
    }

    if let Err(error) = floetask_infrastructure::paths::migrate_legacy_files(&paths) {
        eprintln!(
            "floetask: could not move saved filters to {}: {error}",
            paths.filters_file().display()
        );
    }

    let services = Services::new(floetask_infrastructure::ports(&paths));
    let startup = floetask_gui::Startup {
        services,
        open: cli.todo_file,
    };
    match floetask_gui::run(startup) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("floetask: {error}");
            ExitCode::FAILURE
        }
    }
}

/// A GUI-subsystem exe has no console, so `--help`, `--paths` and errors
/// would print nowhere. When started from a terminal, write to that
/// terminal instead; when started by a double click there is none to attach
/// to and nothing happens.
#[cfg(windows)]
fn attach_parent_console() {
    use windows_sys::Win32::System::Console::{ATTACH_PARENT_PROCESS, AttachConsole};
    // SAFETY: AttachConsole has no preconditions; failure only means there is
    // no parent console, which is fine.
    unsafe {
        AttachConsole(ATTACH_PARENT_PROCESS);
    }
}

#[cfg(not(windows))]
fn attach_parent_console() {}
