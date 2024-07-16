/*
 * tasklog - A command-line task tracking tool.
 *
 * Copyright (C) 2024 Lucas M. de Jong Larrarte
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */
use std::{env, fs};
use std::path::PathBuf;
use std::str::FromStr;

use chrono::{Days, Duration, Local, NaiveDate, NaiveTime};
use clap::{Parser, Subcommand, ArgAction};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::task_manager::{TaskEnd, TaskManager};

pub mod task_manager;

/// Errors that can occur while managing tasks.
#[derive(Error, Debug)]
pub enum TaskError {
    #[error("Task '{0}' is already running")]
    TaskAlreadyRunning(String),
    #[error("No task is currently running")]
    TaskNotRunning,
    #[error("No tasks found")]
    NoTasksFound,
    #[error("Task '{0}' not found")]
    TaskNotFound(String),
    #[error("Task '{0}' already exists")]
    TaskAlreadyExists(String),
    #[error("Task name is ambiguous")]
    MultipleTasksFound,
    #[error("File IO error: {0}")]
    FileIO(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Configuration error: {0}")]
    ConfigError(#[from] config::ConfigError),
}


/// Result type for task operations.
pub type Result<T> = std::result::Result<T, TaskError>;

/// Command-line interface structure.
#[derive(Debug, Parser)]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
    /// The path to the configuration file.
    #[arg(long, value_name = "FILE", short = 'C', global = true)]
    config: Option<PathBuf>,
}

/// Enumeration of available commands.
#[derive(Debug, Subcommand)]
enum Command {
    /// Starts work on a given task.
    #[command()]
    Start {
        /// The name of the task to start.
        #[arg(value_name = "TASK")]
        task: String
    },
    /// Resumes work on the given task. If no task is given, resumes the last task.
    Resume {
        /// The name of the task to resume.
        #[arg(value_name = "TASK")]
        task: Option<String>
    },
    /// Stops work on the current task.
    Stop {
        /// The number of days between today and the day for which to stop the running task.
        #[arg(short, require_equals = true, value_name = "DAYS", requires = "duration")]
        n: Option<u16>,
        /// The task duration in minutes.
        #[arg(short, long, require_equals = true, value_name = "MINUTES")]
        duration: Option<u16>
    },
    /// Switches to a different task.
    Switch {
        /// The name of the task to switch to.
        #[arg(value_name = "TASK")]
        task: Option<String>,
        #[arg(short, long, action = ArgAction::SetTrue, requires = "task")]
        create: bool,
    },
    /// Prints a report of the tasks worked on in a day.
    Report {
        /// The number of days between today and the day to report.
        #[arg(short, action = ArgAction::Append, require_equals = true, value_name = "DAYS", default_values = vec!["0"])]
        n: Vec<u16>
    },
    /// Prints the current task.
    Current,
    /// Renames a task.
    Rename {
        /// The name of the task to rename.
        #[arg(value_name = "TASK")]
        task: String,
        /// The new name of the task.
        #[arg(value_name = "NEW_NAME")]
        new_name: String
    },
    /// Lists all tasks.
    List {
        /// The number of days before today to list tasks.
        #[arg(short, default_value_t = 0, require_equals = true, value_name = "DAYS")]
        n: u16
    },
    /// Deletes a task.
    Delete {
        /// The name of the task to delete.
        #[arg(value_name = "TASK")]
        task: String
    },
}

/// Configuration structure representing configuration options.
#[derive(Debug, Serialize, Deserialize)]
struct Config {
    data_dir: String,
    day_start: String,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            data_dir: dirs::data_local_dir().expect("data_local_dir should exist")
                .join("tasklog").to_str().expect("data_dir should be a valid string")
                .to_string(),
            day_start: "04:30".to_string(),
        }
    }
}
impl Config {
    /// Loads the configuration from the given file.
    fn load(config_file: PathBuf) -> Result<Self> {
        Self::create_config_file_if_needed(&config_file)?;
        let settings = config::Config::builder().add_source(
            config::File::from(config_file)
        ).add_source(
            config::Environment::with_prefix("TASKLOG")
        ).build()?;
        Ok(settings.try_deserialize()?)
    }


    /// Creates the config file if it doesn't exist.
    fn create_config_file_if_needed(config_file: &PathBuf) -> Result<()> {
        if !config_file.exists() {
            if let Some(parent) = config_file.parent() {
                fs::create_dir_all(parent)?;
            }
            let default_config = Config::default();
            fs::write(config_file, toml::to_string(&default_config).expect("config should be serializable"))?;
        }
        Ok(())
    }
}

/// Handles the command-line arguments and executes the corresponding command.
pub fn handle(cli: Cli) -> Result<()> {
    let config = cli.config.unwrap_or_else(|| 
    env::var("TASKLOG_CONFIG").map(PathBuf::from).unwrap_or_else(|_| 
    dirs::config_local_dir().expect("config_local_dir should exist")
        .join("tasklog").join("settings.toml")
    ));
    let config = Config::load(config)?;
    fs::create_dir_all(PathBuf::from(&config.data_dir))?;
    match cli.command {
        Command::Start { task } => start_new(task, &config),
        Command::Resume { task } => match task {
            None => resume_last(&config),
            Some(task) => resume(task, &config),
        }
        Command::Stop { n, duration } => stop(n.unwrap_or(0), duration, &config),
        Command::Switch { task, create } => if create { 
            switch_new(task.expect("task should exist when create flag is set"), &config) 
        } else { 
            match task {
                Some(task) => switch(task, &config),
                None => switch_previous(&config),
            } 
        },
        Command::Report { n } => report(n, &config),
        Command::Current => current(&config),
        Command::Rename { task, new_name } => rename(task, new_name, &config),
        Command::List { n } => list(n, &config),
        Command::Delete { task } => delete(task, &config),
    }
}

/// Processes a mutating action on the tasks.
fn process_mutating_action<T>(days_ago: u16, config: &Config, action: impl FnOnce(&mut TaskManager) -> Result<T>) -> Result<T> {
    let today = date(days_ago, config)?;
    let mut tasks = read_tasks(today, config)?;
    let task_name = action(&mut tasks)?;
    write_tasks(&tasks, today, config)?;
    Ok(task_name)
}

/// Resumes the task with the given name.
fn resume(task_name: String, config: &Config) -> Result<()> {
    let task_name = process_mutating_action(0, config, |task_manager| 
    task_manager.resume_task(task_name, Local::now()))?;
    println!("Resumed task: {task_name}");
    Ok(())
}

/// Starts a new task with the given name.
fn start_new(task_name: String, config: &Config) -> Result<()> {
    let task_name = process_mutating_action(0, config, |task_manager| 
    task_manager.start_new_task(task_name, Local::now()))?;
    println!("Started new task: {task_name}");
    Ok(())
}

/// Stops the currently running task.
fn stop(days_ago: u16, duration: Option<u16>, config: &Config) -> Result<()> {
    let task_name = process_mutating_action(days_ago, config, |task_manager| 
        match duration {
            None => task_manager.stop_current_task(TaskEnd::Time(Local::now())),
            Some(minutes) => task_manager.stop_current_task(TaskEnd::Duration(Duration::minutes(minutes as i64)))
        }
    )?;
    println!("Stopped task: {task_name}");
    Ok(())
}

/// Resumes the last running task.
fn resume_last(config: &Config) -> Result<()> {
    let task_name = process_mutating_action(0, config, |task_manager| 
    task_manager.resume_last_task(Local::now()))?;
    println!("Resumed task: {task_name}");
    Ok(())
}

/// Switches to the given task.
fn switch(task_name: String, config: &Config) -> Result<()> {
    let task_name = process_mutating_action(0, config, |task_manager| task_manager.switch_task(task_name, Local::now()))?;
    println!("Switched to task: {task_name}");
    Ok(())
}

/// Switches to a new task.
fn switch_new(task_name: String, config: &Config) -> Result<()> {
    let task_name = process_mutating_action(0, config, |task_manager| task_manager.switch_new_task(task_name, Local::now()))?;
    println!("Switched to new task: {task_name}");
    Ok(())
}

/// Switches to the previous task.
fn switch_previous(config: &Config) -> Result<()> {
    let task_name = process_mutating_action(0, config, |task_manager| task_manager.switch_last_task(Local::now()))?;
    println!("Switched to task: {task_name}");
    Ok(())
}

/// Prints the name of the currently running task.
fn current(config: &Config) -> Result<()> {
    let today = date(0, config)?;
    let task_manager = read_tasks(today, config)?;
    match task_manager.current_task() {
        None => println!("No task currently running"),
        Some(task) => println!("Current task: {}", task),
    }
    Ok(())
}

/// Lists all tasks.
fn list(days_ago: u16,config: &Config) -> Result<()> {
    let today = date(days_ago, config)?;
    let task_manager = read_tasks(today, config)?;
    let tasks = task_manager.list_tasks();
    println!("{}", tasks.join("\n"));
    Ok(())
}

/// Deletes the given task.
fn delete(task_name: String, config: &Config) -> Result<()> {
    let task_name = process_mutating_action(0, config, |task_manager| task_manager.delete_task(task_name))?;
    println!("Deleted task: {task_name}");
    Ok(())
}

/// Renames the given task.
fn rename(task_name: String, new_name: String, config: &Config) -> Result<()> {
    let (task_name, new_name) = process_mutating_action(0, config, |task_manager| task_manager.rename_task(task_name, new_name))?;
    println!("Renamed task: {task_name} to {new_name}");
    Ok(())
}

/// Prints a report of the tasks worked on. The report is generated for the given number of days ago.
fn report(days_ago: Vec<u16>, config: &Config) -> Result<()> {
    for days_ago in days_ago {
        let date = date(days_ago, config)?;
        let task_manager = read_tasks(date, config)?;
        let report = task_manager.generate_report(date, Local::now());
        println!("\n{report}");
    }
    Ok(())
}

/// Reads the tasks from the file for the given date.
fn read_tasks(today: NaiveDate, config: &Config) -> Result<TaskManager> {
    let file = get_file(today, config)?;
    let task_manager = match fs::read_to_string(file) {
        Ok(data) => serde_json::from_str(&data)?,
        Err(_) => TaskManager::default(),
    };
    Ok(task_manager)
}

/// Writes the tasks to the file for the given date.
fn write_tasks(tasks: &TaskManager, today: NaiveDate, config: &Config) -> Result<()> {
    let file = get_file(today, config)?;
    let data = serde_json::to_string(&tasks).expect("should be able to serialize tasks");
    fs::write(file, data)?;
    Ok(())
}

/// Gets the file path for the given date.
fn get_file(today: NaiveDate, config: &Config) -> Result<PathBuf> {
    let today = today.format("%F.json").to_string();
    let file = PathBuf::from(&config.data_dir).join(today);
    Ok(file)
}

/// Returns today's date, or yesterday if it's before configured day start.
fn date(days_ago: u16, config: &Config) -> Result<NaiveDate> {
    let now = Local::now();
    let time = now.time();
    let day_start = NaiveTime::from_str(&config.day_start)
        .map_err(|e| config::ConfigError::Foreign(Box::new(e)))?;
    let today = if time >= day_start {
        now.date_naive()
    } else { 
        now.date_naive().checked_sub_days(Days::new(1)).expect("should be able to subtract 1 from today")
    };
    Ok(today.checked_sub_days(Days::new(days_ago as u64)).expect("should be able to subtract any u16 days from today"))
}
