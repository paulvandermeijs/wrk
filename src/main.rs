use std::{env, error::Error, fs, process::Command};

use clap::{arg, command, Parser};
use inquire::Select;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Select a project
    project: Option<String>,

    #[command(flatten)]
    action: Action,
}

#[derive(clap::Args, Debug)]
#[group(multiple = false)]
struct Action {
    /// Open the project
    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    open: bool,

    /// Execute a command
    #[arg(short = 'x', long, value_name = "COMMAND")]
    execute: Option<String>,
}

fn main() {
    let args = Args::parse();

    let editor_command = if args.action.open {
        get_editor_command().expect("No editor configured")
    } else {
        "".to_string()
    };
    let workspace_root = get_workspace_root().expect("Unable to determine workspace root");
    let mut projects = get_projects(&workspace_root)
        .expect(&format!("Could not read projects from {workspace_root}"));
    let project = if let Some(project) = &args.project {
        projects
            .into_iter()
            .find(|v| v.to_lowercase() == project.to_lowercase())
            .expect(&format!("Couldn't find project {project}"))
    } else {
        projects.sort();

        Select::new("Select a project", projects)
            .prompt()
            .expect("Failed to select a project")
    };
    let project_full_path = format!("{}/{}", workspace_root, project);

    if args.action.open {
        Command::new(&editor_command)
            .arg(&project_full_path)
            .spawn()
            .expect(&format!("Failed to run editor command {editor_command}"))
            .wait()
            .expect("Editor command returned a non-zero status");
    } else if let Some(execute_command) = args.action.execute {
        let shell = env::var("SHELL").expect("Couldn't determine shell");

        Command::new(&shell)
            .current_dir(&project_full_path)
            .args(["-c", &execute_command])
            .spawn()
            .expect(&format!("Failed to run execute command {execute_command}"))
            .wait()
            .expect("Execute command returned a non-zero status");
    } else {
        let shell = env::var("SHELL").expect("Couldn't determine shell");

        Command::new(&shell)
            .current_dir(&project_full_path)
            .arg("-i")
            .spawn()
            .expect(&format!("Failed to run shell command {shell}"))
            .wait()
            .expect("Shell command returned a non-zero status");
    };
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
