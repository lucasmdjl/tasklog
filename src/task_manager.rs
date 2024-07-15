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
use std::cmp::Ordering;
use chrono::{DateTime, Duration, Local, NaiveDate, NaiveTime};
use serde::{Deserialize, Deserializer, Serialize};
use serde::de;
use crate::TaskError;

/// Task structure representing a task with its name, time segments, and current state.
/// 
/// ### Contract:
/// - Segments are stored in chronological order.
/// - No two segments overlap.
/// - If current is present, its value is after the end of the last segment.
/// - Either current is present or segments is non-empty.
#[derive(Debug, Serialize, PartialEq)]
pub struct Task {
    /// The name of the task.
    name: String,
    /// The time segments of work done on the task.
    segments: Vec<Segment>,
    /// The current state of the task: [Some] with the last start time when the task is running,
    /// [None] otherwise.
    #[serde(skip_serializing_if = "Option::is_none")]
    current: Option<DateTime<Local>>,
}
impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self == other {
            return Some(Ordering::Equal);
        }
        if self.name != other.name { 
            return None;
        }
        if !other.is_running() && self.start_time() >= other.stop_time().unwrap() {
            return Some(Ordering::Greater); 
        }
        if !self.is_running() && other.start_time() >= self.stop_time().unwrap() {
            return Some(Ordering::Less);
        }
        None
    }
}
impl Task {
    /// Returns the name of the task.
    pub fn name(&self) -> &str {
        &self.name
    }
    
    /// Renames the task.
    pub fn rename(&mut self, new_name: impl ToString) {
        self.name = new_name.to_string();
    }
    
    /// Creates a new (running) task with the given name.
    pub fn new(name: impl ToString, now: DateTime<Local>) -> Self {
        Task {
            name: name.to_string(),
            segments: vec![],
            current: Some(now),
        }
    }

    /// Checks if the task is currently running.
    pub fn is_running(&self) -> bool {
        self.current.is_some()
    }

    /// Stops the task if it is running. Returns [Err] if the task is not running.
    pub fn stop(&mut self, end: TaskEnd) -> crate::Result<()> {
        let start = match self.current {
            None => return Err(TaskError::TaskNotRunning),
            Some(start) => start,
        };
        let end = match end {
            TaskEnd::Time(end) => end,
            TaskEnd::Duration(duration) => start + duration,
        };
        self.segments.push(Segment::new(start, end));
        self.current = None;
        Ok(())
    }

    /// Starts the task if it is not already running. Returns [Err] if the task is already running.
    pub fn start(&mut self, now: DateTime<Local>) -> crate::Result<()> {
        if self.is_running() {
            return Err(TaskError::TaskAlreadyRunning(self.name.clone()));
        }
        let end = self.segments.last().unwrap().end;
        assert!(now >= end);
        self.current = Some(now);
        Ok(())
    }

    /// Calculates the total time spent on the task.
    pub fn time_spent(&self, now: NaiveTime) -> Duration {
        let current = match self.current {
            Some(current) => now - current.time(),
            None => Duration::zero(),
        };
        self.segments.iter().fold(current, |total, segment| total + segment.duration())
    }
    
    /// Returns the start time of the task.
    pub fn start_time(&self) -> DateTime<Local> {
        self.segments.first().map(|segment| segment.start)
            .or(self.current).unwrap()
    }
    
    /// Returns the last stop time of the task, or None if the task is running.
    pub fn stop_time(&self) -> Option<DateTime<Local>> {
        if self.is_running() { 
            None
        } else {
            self.segments.last().map(|segment| segment.end)
        }
    }
}

#[derive(Debug, Deserialize)]
struct TaskDeser {
    name: String,
    segments: Vec<SegmentDeser>,
    #[serde(skip_serializing_if = "Option::is_none")]
    current: Option<DateTime<Local>>,
}
impl <'de> Deserialize<'de> for Task {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        let deser = TaskDeser::deserialize(deserializer)?;
        deser.try_into().map_err(|e| de::Error::custom(e))
    }
}
impl TryFrom<TaskDeser> for Task {
    type Error = String;
    fn try_from(value: TaskDeser) -> Result<Self, Self::Error> {
        if value.segments.is_empty() && value.current.is_none() { 
            Err("task must be running or have at least one segment")?;
        }
        let segments: Vec<Segment> = value.segments.into_iter().map(|segment_deser| segment_deser.try_into()).collect::<Result<_, _>>()?;
        for i in 1..segments.len() {
            if segments[i].start < segments[i - 1].end { 
                Err("segments must be in chronological order")?;
            }
        }
        if let Some(current) = value.current {
            if let Some(last) = segments.last() {
                if current < last.end { 
                    Err("current must be after the end of the last segment")?;
                }
            }
        }
        Ok(Task {
            name: value.name,
            segments,
            current: value.current,
        })
    }
}

/// Time segment structure representing the start and end times of work done on a task.
/// 
/// ### Contract:
/// - The start and end times are stored in chronological order.
#[derive(Debug, Serialize, PartialEq)]
pub struct Segment {
    start: DateTime<Local>,
    end: DateTime<Local>,
}
impl PartialOrd for Segment {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self == other {
            Some(Ordering::Equal)
        } else if self.start >= other.end {
            Some(Ordering::Greater)
        } else if self.end <= other.start {
            Some(Ordering::Less)
        } else {
            None
        }
    }
}
impl Segment {
    
    /// Creates a new segment with the given start and end times.
    pub fn new(start: DateTime<Local>, end: DateTime<Local>) -> Self {
        assert!(start <= end);
        Segment { start, end }
    }
    
    /// Calculates the duration of the time segment.
    pub fn duration(&self) -> Duration {
        self.end - self.start
    }
}

/// Describes the end of a task.
pub enum TaskEnd {
    /// Ends the task at the given time.
    Time(DateTime<Local>),
    /// Ends the task after the given duration.
    Duration(Duration),
}

#[derive(Debug, Deserialize)]
struct SegmentDeser {
    start: DateTime<Local>,
    end: DateTime<Local>,
}
impl <'de> Deserialize<'de> for Segment {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        let deser = SegmentDeser::deserialize(deserializer)?;
        deser.try_into().map_err(|e| de::Error::custom(e))
    }
}
impl TryFrom<SegmentDeser> for Segment {
    type Error = String;
    fn try_from(value: SegmentDeser) -> Result<Self, Self::Error> {
        if value.start > value.end {
            Err("Start time must be before end time".to_string())
        } else {
            Ok(Segment { start: value.start, end: value.end })
        }
    }
}

/// List of tasks.
/// 
/// ### Contract:
/// - The last task in the list is the one currently running if there is one,
/// or the last to have run otherwise.
#[derive(Debug, Serialize, Default)]
pub struct TaskManager { 
    tasks: Vec<Task> 
}
impl TaskManager {

    /// Returns the currently running task if any.
    pub fn current_task(&self) -> Option<&str> {
        self.tasks.last().filter(|&task| task.is_running()).map(Task::name)
    }
    
    /// Checks if there is a current task. Returns [Err] if there is one.
    fn check_no_current_task(&self) -> crate::Result<()> {
        match self.current_task() {
            None => Ok(()),
            Some(task_name) => Err(TaskError::TaskAlreadyRunning(task_name.to_string()))
        }
    }
    
    /// Returns the index of the task with the given name.
    fn index_of(&self, task_name: &str) -> Option<usize> {
        self.tasks.iter().enumerate().find(|(_, task)| task.name == task_name).map(|(i, _)| i)
    }
    
    /// Resumes an existing one with the given name.
    pub fn resume_task(&mut self, task_name: String, start: DateTime<Local>) -> crate::Result<String> {
        self.check_no_current_task()?;
        match self.index_of(&task_name) {
            None => Err(TaskError::TaskNotFound(task_name)),
            Some(index) => {
                let mut task = self.tasks.swap_remove(index);
                task.start(start)?;
                self.tasks.push(task);
                Ok(task_name)
            }
        }
    }
    
    /// Starts a new task with the given name.
    pub fn start_new_task(&mut self, task_name: String, start: DateTime<Local>) -> crate::Result<String> {
        self.check_no_current_task()?;
        match self.index_of(&task_name) {
            None => {
                let new_task = Task::new(task_name.clone(), start);
                self.tasks.push(new_task);
                Ok(task_name)
            }
            Some(_) => Err(TaskError::TaskAlreadyExists(task_name))
        }
    }

    /// Stops the current task.
    pub fn stop_current_task(&mut self, end: TaskEnd) -> crate::Result<String> {
        let task = self.tasks.last_mut();
        match task {
            None => Err(TaskError::NoTasksFound),
            Some(task) => {
                task.stop(end)?;
                Ok(task.name.clone())
            }
        }
    }

    /// Resumes the last task.
    pub fn resume_last_task(&mut self, start: DateTime<Local>) -> crate::Result<String> {
        let task = self.tasks.last_mut();
        match task {
            None => Err(TaskError::NoTasksFound),
            Some(task) => {
                task.start(start)?;
                Ok(task.name.clone())
            }
        }
    }

    /// Stops the current task and resumes the given one.
    pub fn switch_task(&mut self, task_name: String, now: DateTime<Local>) -> crate::Result<String> {
        self.stop_current_task(TaskEnd::Time(now))?;
        let task = self.resume_task(task_name, now)?;
        Ok(task)
    }

    /// Stops the current task and starts a new one.
    pub fn switch_new_task(&mut self, task_name: String, now: DateTime<Local>) -> crate::Result<String> {
        self.stop_current_task(TaskEnd::Time(now))?;
        let task = self.start_new_task(task_name, now)?;
        Ok(task)
    }

    /// Generates a report of the tasks.
    pub fn generate_report(&self, date: NaiveDate, time: NaiveTime) -> String {
        let mut report = format!("  {} \n", date.format("%F"));
        let total = self.tasks.iter().fold(Duration::zero(), |total, task| total + task.time_spent(time));
        let max_length = self.tasks.iter().map(|task| if task.is_running() { task.name.len() + 4 } else { task.name.len() }).max().unwrap_or(0).max(5);
        for task in &self.tasks {
            let time = task.time_spent(time);
            let minutes = time.num_minutes() % 60;
            let hours = time.num_hours();
            let percent = (time.num_milliseconds() as f64 * 100.0 / total.num_milliseconds() as f64).round() as u32;
            if task.is_running() {
                report += &format!("    {:<max_length$} | {hours:0>2}:{minutes:0>2} | {percent:>3}%\n", task.name.clone() + " (R)");
            } else {
                report += &format!("    {:<max_length$} | {hours:0>2}:{minutes:0>2} | {percent:>3}%\n", task.name.clone());
            }
        }
        let minutes = total.num_minutes() % 60;
        let hours = total.num_hours();
        report += &format!("    {:=>1$}\n", "", max_length + 15);
        report += &format!("    {:<max_length$} | {hours:0>2}:{minutes:0>2} | 100%\n", "Total");
        report
    }
    
    pub fn rename_task(&mut self, task_name: String, new_name: String) -> crate::Result<(String, String)> {
        let task = self.tasks.iter_mut().find(|task| task.name == task_name);
        match task {
            None => Err(TaskError::TaskNotFound(task_name)),
            Some(task) => {
                task.rename(new_name.clone());
                Ok((task_name, new_name))
            }
        }
    }
}

#[derive(Debug, Deserialize)]
struct TaskManagerDeser {
    tasks: Vec<TaskDeser>
}
impl <'de> Deserialize<'de> for TaskManager {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        let deser = TaskManagerDeser::deserialize(deserializer)?;
        deser.try_into().map_err(|e: String| de::Error::custom(e))
    }
}
impl TryFrom<TaskManagerDeser> for TaskManager {
    type Error = String;
    fn try_from(value: TaskManagerDeser) -> Result<Self, Self::Error> {
        let size = value.tasks.len();
        let tasks = value.tasks.into_iter().map(Task::try_from).collect::<Result<Vec<_>, _>>()?;
        if tasks.iter().take(size - 1).any(|task| task.is_running()) {
            Err("Only the last task can be running.".to_string())
        } else {
            Ok(TaskManager { tasks })
        }
    }
}
