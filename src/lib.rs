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
use std::ops::RangeInclusive;
use std::path::PathBuf;
use std::str::FromStr;

use chrono::{Days, Duration, Local, NaiveDate, NaiveTime};
use clap::{Parser, Subcommand, ArgAction, builder::ArgPredicate};
use serde::{Deserialize, Serialize};

pub use crate::task_manager::{TaskManager, TaskError, TaskResult};

pub mod task_manager;


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
        /// The name of the task to start. If no name is given, the previous task is started.
        #[arg(value_name = "TASK")]
        task: Option<String>,
        /// Creates the task before starting it. Requires a task name.
        #[arg(short, long, action = ArgAction::SetTrue, requires = "task")]
        create: bool,
    },
    /// Stops work on the current task.
    Stop {
        /// The number of days between today and the day for which to stop the running task.
        #[arg(long, require_equals = true, value_name = "DATE", requires = "duration")]
        date: Option<NaiveDate>,
        /// The task duration in minutes.
        #[arg(short, long, require_equals = true, value_name = "MINUTES")]
        duration: Option<u16>
    },
    /// Switches to a different task.
    Switch {
        /// The name of the task to switch to. If no name is given, switch to the previous task.
        #[arg(value_name = "TASK")]
        task: Option<String>,
        /// Creates the task before switching to it. Requires a task name.
        #[arg(short, long, action = ArgAction::SetTrue, requires = "task")]
        create: bool,
    },
    /// Prints a report of the tasks worked on in a day.
    Report {
        /// Whether to report on today.
        #[arg(short, short_alias = '0', long, action = ArgAction::SetTrue, default_value = "true", default_value_ifs = [
            ("yesterday", ArgPredicate::IsPresent, Some("false")), 
            ("dates", ArgPredicate::IsPresent, Some("false")),
            ("from", ArgPredicate::IsPresent, Some("false")),
            ("to", ArgPredicate::IsPresent, Some("false")),
        ], conflicts_with_all = ["from", "to"])]
        today: bool,
        /// Whether to report on yesterday.
        #[arg(short, short_alias = '1', long, action = ArgAction::SetTrue, conflicts_with_all = ["from", "to"])]
        yesterday: bool,
        /// The dates to report on. In format YYYY-MM-DD.
        #[arg(long, action = ArgAction::Append, value_name = "DATE", num_args = 0.., conflicts_with_all = ["from", "to"])]
        dates: Vec<NaiveDate>,
        /// The date to start the report from (inclusive). In format YYYY-MM-DD.
        #[arg(long, value_name = "DATE", require_equals = true, conflicts_with_all = ["today", "yesterday", "dates"])]
        from: Option<NaiveDate>,
        /// The date to end the report on (inclusive). In format YYYY-MM-DD.
        #[arg(long, value_name = "DATE", requires = "from", require_equals = true, conflicts_with_all = ["today", "yesterday", "dates"])]
        to: Option<NaiveDate>,
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
    fn load(config_file: PathBuf) -> TaskResult<Self> {
        Self::create_config_file_if_needed(&config_file)?;
        let settings = config::Config::builder().add_source(
            config::File::from(config_file)
        ).add_source(
            config::Environment::with_prefix("TASKLOG")
        ).build()?;
        Ok(settings.try_deserialize()?)
    }


    /// Creates the config file if it doesn't exist.
    fn create_config_file_if_needed(config_file: &PathBuf) -> TaskResult<()> {
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
pub fn handle(cli: Cli) -> TaskResult<()> {
    let config = cli.config.unwrap_or_else(|| 
    env::var("TASKLOG_CONFIG").map(PathBuf::from).unwrap_or_else(|_| 
    dirs::config_local_dir().expect("config_local_dir should exist")
        .join("tasklog").join("settings.toml")
    ));
    let config = Config::load(config)?;
    fs::create_dir_all(PathBuf::from(&config.data_dir))?;
    match cli.command {
        Command::Start { task, create } => if create {
            start_new(task.expect("task should exist when create flag is set"), &config)
        } else {
            match task {
                Some(task) => resume(task, &config),
                None => resume_last(&config),
            }
        },
        Command::Stop { date, duration } => stop(date, duration, &config),
        Command::Switch { task, create } => if create { 
            switch_new(task.expect("task should exist when create flag is set"), &config) 
        } else { 
            match task {
                Some(task) => switch(task, &config),
                None => switch_previous(&config),
            } 
        },
        Command::Report { today, yesterday, dates, from, to } => report(today, yesterday, dates, from, to, &config),
        Command::Current => current(&config),
        Command::Rename { task, new_name } => rename(task, new_name, &config),
        Command::List { n } => list(n, &config),
        Command::Delete { task } => delete(task, &config),
    }
}

/// Processes a mutating action on the tasks.
fn process_mutating_action<T>(date: NaiveDate, config: &Config, action: impl FnOnce(&mut TaskManager) -> TaskResult<T>) -> TaskResult<T> {
    let mut tasks = read_tasks(date, config)?;
    let task_name = action(&mut tasks)?;
    write_tasks(&tasks, date, config)?;
    Ok(task_name)
}

/// Resumes the task with the given name.
fn resume(task_name: String, config: &Config) -> TaskResult<()> {
    let today = today(config)?;
    let task_name = process_mutating_action(today, config, |task_manager|
    task_manager.resume_task(task_name, Local::now()))?;
    println!("Resumed task: {task_name}");
    Ok(())
}

/// Starts a new task with the given name.
fn start_new(task_name: String, config: &Config) -> TaskResult<()> {
    let today = today(config)?;
    let task_name = process_mutating_action(today, config, |task_manager|
    task_manager.start_new_task(task_name, Local::now()))?;
    println!("Started new task: {task_name}");
    Ok(())
}

/// Stops the currently running task.
fn stop(date: Option<NaiveDate>, duration: Option<u16>, config: &Config) -> TaskResult<()> {
    let date = date.unwrap_or(today(config)?);
    let task_name = process_mutating_action(date, config, |task_manager|
        match duration {
            None => task_manager.stop_running_task_with_time(Local::now()),
            Some(minutes) => task_manager.stop_running_task_with_duration(Duration::minutes(minutes as i64), Local::now()),
        }
    )?;
    println!("Stopped task: {task_name}");
    Ok(())
}

/// Resumes the last running task.
fn resume_last(config: &Config) -> TaskResult<()> {
    let today = today(config)?;
    let task_name = process_mutating_action(today, config, |task_manager|
    task_manager.resume_last_task(Local::now()))?;
    println!("Resumed task: {task_name}");
    Ok(())
}

/// Switches to the given task.
fn switch(task_name: String, config: &Config) -> TaskResult<()> {
    let today = today(config)?;
    let task_name = process_mutating_action(today, config, |task_manager| task_manager.switch_task(task_name, Local::now()))?;
    println!("Switched to task: {task_name}");
    Ok(())
}

/// Switches to a new task.
fn switch_new(task_name: String, config: &Config) -> TaskResult<()> {
    let today = today(config)?;
    let task_name = process_mutating_action(today, config, |task_manager| task_manager.switch_new_task(task_name, Local::now()))?;
    println!("Switched to new task: {task_name}");
    Ok(())
}

/// Switches to the previous task.
fn switch_previous(config: &Config) -> TaskResult<()> {
    let today = today(config)?;
    let task_name = process_mutating_action(today, config, |task_manager| task_manager.switch_last_task(Local::now()))?;
    println!("Switched to task: {task_name}");
    Ok(())
}

/// Prints the name of the currently running task.
fn current(config: &Config) -> TaskResult<()> {
    let today = date(0, config)?;
    let task_manager = read_tasks(today, config)?;
    match task_manager.running_task() {
        None => println!("No task currently running"),
        Some(task) => println!("Current task: {}", task),
    }
    Ok(())
}

/// Lists all tasks.
fn list(days_ago: u16,config: &Config) -> TaskResult<()> {
    let today = date(days_ago, config)?;
    let task_manager = read_tasks(today, config)?;
    let tasks = task_manager.list_tasks();
    println!("{}", tasks.join("\n"));
    Ok(())
}

/// Deletes the given task.
fn delete(task_name: String, config: &Config) -> TaskResult<()> {
    let today = today(config)?;
    let task_name = process_mutating_action(today, config, |task_manager| task_manager.delete_task(task_name))?;
    println!("Deleted task: {task_name}");
    Ok(())
}

/// Renames the given task.
fn rename(task_name: String, new_name: String, config: &Config) -> TaskResult<()> {
    let today = today(config)?;
    let (task_name, new_name) = process_mutating_action(today, config, |task_manager| task_manager.rename_task(task_name, new_name))?;
    println!("Renamed task: {task_name} to {new_name}");
    Ok(())
}

/// Prints a report of the tasks worked on. The report is generated for the given number of days ago.
fn report(today: bool, yesterday: bool, mut dates: Vec<NaiveDate>, from: Option<NaiveDate>, to: Option<NaiveDate>, config: &Config) -> TaskResult<()> {
    if let Some(from) = from {
        let to = to.unwrap_or(date(0, config)?);
        dates = NaiveDateIter::new(from, to).collect();
    } else {
        if yesterday {
            dates.push(date(1, config)?);
        }
        if today {
            dates.push(date(0, config)?);
        }
        dates.sort();
        dates.dedup();
    }
    let now = Local::now();
    println!();
    for date in dates {
        let task_manager = read_tasks(date, config)?;
        let report = task_manager.generate_report(date, now);
        println!("{report}");
    }
    Ok(())
}

/// An iterator that yields dates in an inclusive range.
struct NaiveDateIter {
    range: RangeInclusive<NaiveDate>,
    current: Option<NaiveDate>,
}
impl NaiveDateIter {
    /// Creates a new iterator.
    fn new(from: NaiveDate, to: NaiveDate) -> Self {
        Self {
            range: from..=to,
            current: Some(from),
        }
    }
}

impl Iterator for NaiveDateIter {
    type Item = NaiveDate;
    fn next(&mut self) -> Option<Self::Item> {
        match self.current {
            None => None,
            Some(current) => {
                if current <= *self.range.end() {
                    self.current = current.succ_opt();
                    Some(current)
                } else {
                    None
                }
            }
        }
    }
}

/// Reads the tasks from the file for the given date.
fn read_tasks(today: NaiveDate, config: &Config) -> TaskResult<TaskManager> {
    let file = get_file(today, config)?;
    let task_manager = match fs::read_to_string(file) {
        Ok(data) => serde_json::from_str(&data)?,
        Err(_) => TaskManager::default(),
    };
    Ok(task_manager)
}

/// Writes the tasks to the file for the given date.
fn write_tasks(tasks: &TaskManager, today: NaiveDate, config: &Config) -> TaskResult<()> {
    let file = get_file(today, config)?;
    let data = serde_json::to_string(&tasks).expect("should be able to serialize tasks");
    fs::write(file, data)?;
    Ok(())
}

/// Gets the file path for the given date.
fn get_file(today: NaiveDate, config: &Config) -> TaskResult<PathBuf> {
    let today = today.format("%F.json").to_string();
    let file = PathBuf::from(&config.data_dir).join(today);
    Ok(file)
}

/// Returns today's date.
fn today(config: &Config) -> TaskResult<NaiveDate> {
    date(0, config)
}

/// Returns the date `days_ago` days ago.
fn date(days_ago: u16, config: &Config) -> TaskResult<NaiveDate> {
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
