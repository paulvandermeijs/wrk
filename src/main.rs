use std::{env, error::Error, fs, process::Command};

use clap::{arg, command, Parser};
use inquire::Select;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Select a project
    project: Option<String>,

    /// Open the project
    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    open: bool,
}

fn main() {
    let args = Args::parse();

    let editor_command = if args.open {
        get_editor_command().expect("No editor configured")
    } else {
        "".to_string()
    };
    let workspace_root = get_workspace_root().expect("Unable to determine workspace root");
    let projects = get_projects(&workspace_root)
        .expect(&format!("Could not read projects from {workspace_root}"));
    let project = if let Some(project) = &args.project {
        projects
            .into_iter()
            .find(|v| v.to_lowercase() == project.to_lowercase())
            .expect(&format!("Couldn't find project {project}"))
    } else {
        Select::new("Select a project", projects)
            .prompt()
            .expect("Failed to select a project")
    };
    let command = if args.open {
        editor_command
    } else {
        "cd".to_string()
    };

    Command::new(&command)
        .arg(format!("{}/{}", workspace_root, project))
        .output()
        .expect(&format!("Failed to run command {command}"));
}

fn get_editor_command() -> Result<String, impl Error> {
    env::var("VISUAL").or_else(|_| env::var("EDITOR"))
}

fn get_workspace_root() -> Result<String, impl Error> {
    env::var("WORKSPACE_ROOT").or_else(|_| env::var("HOME").map(|home| format!("{home}/Workspace")))
}

fn get_projects(workspace_root: &String) -> Result<Vec<String>, impl Error> {
    fs::read_dir(workspace_root).map(|rd| {
        rd.filter_map(|d| {
            d.ok().and_then(|e| {
                if e.path().is_dir() {
                    e.path()
                        .as_path()
                        .file_name()
                        .and_then(|o| o.to_str())
                        .map(|s| s.to_string())
                } else {
                    None
                }
            })
        })
        .collect::<Vec<String>>()
    })
}
