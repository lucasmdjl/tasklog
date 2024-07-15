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
use colored::Colorize;
use serde::{Deserialize, Deserializer, Serialize};
use serde::de;
use crate::TaskError;

/// Task enum representing a running or stopped task.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum Task {
    Running(RunningTask),
    Stopped(StoppedTask),
}

/// Task structure representing a running task.
#[derive(Debug, Serialize, PartialEq)]
pub struct RunningTask {
    name: String,
    segments: Vec<Segment>,
    current: DateTime<Local>,
}
impl RunningTask {
    /// Creates a new running task with the given name.
    pub fn new(name: impl ToString, now: DateTime<Local>) -> Self {
        RunningTask {
            name: name.to_string(),
            segments: vec![],
            current: now,
        }
    }
    
    /// Stops the task.
    pub fn stop(self, end: TaskEnd) -> StoppedTask {
        let start = self.current;
        let end = match end {
            TaskEnd::Time(end) => end,
            TaskEnd::Duration(duration) => start + duration,
        };
        StoppedTask {
            name: self.name,
            segments: self.segments,
            last_segment: Segment::new(start, end),
        }
    }
}

/// Task structure representing a stopped task.
#[derive(Debug, Deserialize)]
pub struct RunningTaskDeser {
    name: String,
    segments: Vec<Segment>,
    current: DateTime<Local>,
}
impl <'de> Deserialize<'de> for RunningTask {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        let deser = RunningTaskDeser::deserialize(deserializer)?;
        deser.try_into().map_err(|e: String| de::Error::custom(e))
    }
}
impl TryFrom<RunningTaskDeser> for RunningTask {
    type Error = String;
    fn try_from(value: RunningTaskDeser) -> Result<Self, Self::Error> {
        let segments = value.segments;
        for i in 1..segments.len() {
            if segments[i].start < segments[i - 1].end { 
                Err("segments must be in chronological order")?;
            }
        }
        if let Some(segment) = segments.last() {
            if value.current < segment.end { 
                Err("current must be after the end of the last segment")?;
            }
        }
        Ok(RunningTask {
            name: value.name,
            segments,
            current: value.current,
        })
    }
}

#[derive(Debug, Serialize, PartialEq)]
pub struct StoppedTask {
    name: String,
    segments: Vec<Segment>,
    last_segment: Segment,
}
impl StoppedTask {
    /// Starts the task. Panics if now is before the end of the last segment.
    pub fn start(self, now: DateTime<Local>) -> RunningTask {
        let end = self.last_segment.end;
        assert!(now >= end);
        let mut segments = self.segments;
        segments.push(self.last_segment);
        RunningTask {
            name: self.name,
            segments,
            current: now,
        }
    }

    /// Returns the last stop time of the task, or None if the task is running.
    pub fn stop_time(&self) -> DateTime<Local> {
        self.last_segment.end
    }
}

#[derive(Debug, Deserialize)]
struct StoppedTaskDeser {
    name: String,
    segments: Vec<Segment>,
    last_segment: Segment,
}
impl <'de> Deserialize<'de> for StoppedTask {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        let deser = StoppedTaskDeser::deserialize(deserializer)?;
        deser.try_into().map_err(de::Error::custom)
    }
}
impl TryFrom<StoppedTaskDeser> for StoppedTask {
    type Error = String;
    fn try_from(value: StoppedTaskDeser) -> Result<Self, Self::Error> {
        let segments = value.segments;
        for i in 1..segments.len() {
            if segments[i].start < segments[i - 1].end {
                Err("segments must be in chronological order")?;
            }
        }
        if let Some(segment) = segments.last() {
            if value.last_segment.start < segment.end {
                Err("segments must be in chronological order")?;
            }
        }
        Ok(StoppedTask {
            name: value.name,
            segments,
            last_segment: value.last_segment,
        })
    }
}

impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self == other {
            return Some(Ordering::Equal);
        }
        if self.name() != other.name() { 
            return None;
        }
        if let Task::Stopped(other) = other {
            if self.start_time() >= other.stop_time() { 
                return Some(Ordering::Greater);
            }
        }
        if let Task::Stopped(slef) = self {
            if other.start_time() >= slef.stop_time() { 
                return Some(Ordering::Less);
            }
        }
        None
    }
}
impl Task {
    /// Returns the name of the task.
    pub fn name(&self) -> &str {
        match self {
            Task::Running(task) => &task.name,
            Task::Stopped(task) => &task.name,
        }
    }

    /// Renames the task.
    pub fn rename(&mut self, new_name: impl ToString) {
        match self {
            Task::Running(task) => { task.name = new_name.to_string(); },
            Task::Stopped(task) => {  task.name = new_name.to_string(); },
        }
    }
    
    /// Creates a new (running) task with the given name.
    pub fn new(name: impl ToString, now: DateTime<Local>) -> Self {
        Task::Running(RunningTask::new(name, now))
    }

    /// Checks if the task is currently running.
    pub fn is_running(&self) -> bool {
        match self {
            Task::Running(_) => true,
            Task::Stopped(_) => false,
        }
    }
    
    /// Starts the task. Returns the started task, or an error if the task is already running.
    pub fn start(self, now: DateTime<Local>) -> crate::Result<Task> {
        match self {
            Task::Running(task) => Err(TaskError::TaskAlreadyRunning(task.name)),
            Task::Stopped(task) => Ok(Task::Running(task.start(now))),
        }
    }
    
    /// Stops the task. Returns the stopped task, or an error if the task is not running.
    pub fn stop(self, end: TaskEnd) -> crate::Result<Task> {
        match self {
            Task::Running(task) => Ok(Task::Stopped(task.stop(end))),
            Task::Stopped(_) => Err(TaskError::TaskNotRunning),
        }
    }

    /// Calculates the total time spent on the task.
    pub fn time_spent(&self, now: NaiveTime) -> Duration {
        match self {
            Task::Running(task) => 
                task.segments.iter().fold(Duration::zero(), |total, segment| total + segment.duration()) +
                    (now - task.current.time()),
            Task::Stopped(task) => task.segments.iter().fold(task.last_segment.duration(), |total, segment| total + segment.duration()),
        }
    }
    
    /// Returns the start time of the task.
    pub fn start_time(&self) -> DateTime<Local> {
        match self {
            Task::Running(task) => task.segments.first().map(|s| s.start).unwrap_or(task.current),
            Task::Stopped(task) => task.segments.first().map(|s| s.start).unwrap_or(task.last_segment.start),
        }
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
        deser.try_into().map_err(de::Error::custom)
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
    tasks: Vec<Task>,
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
    
    /// Returns the index of the task matching the given predicate if any. If there are multiple, returns [Err].
    fn index_of(&self, f: impl Fn(&Task) -> bool) -> crate::Result<Option<usize>> {
        let mut rs: Vec<_> = self.tasks.iter().enumerate().filter(|(_, task)| f(task)).map(|(i, _)| i).collect();
        if rs.len() >= 2 { 
            Err(TaskError::MultipleTasksFound)
        } else {
            Ok(rs.pop())
        }
    }
    
    /// Resumes an existing task with the given name.
    pub fn resume_task(&mut self, task_name: String, start: DateTime<Local>) -> crate::Result<String> {
        self.check_no_current_task()?;
        match self.index_of(|task| task.name().contains(&task_name))? {
            None => Err(TaskError::TaskNotFound(task_name)),
            Some(index) => {
                let task = self.tasks.swap_remove(index);
                self.tasks.push(task.start(start)?);
                Ok(task_name)
            }
        }
    }
    
    /// Starts a new task with the given name.
    pub fn start_new_task(&mut self, task_name: String, start: DateTime<Local>) -> crate::Result<String> {
        self.check_no_current_task()?;
        match self.index_of(|task| task.name() == task_name)? {
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
        let task = self.tasks.pop();
        match task {
            None => Err(TaskError::NoTasksFound),
            Some(task) => {
                let name = task.name().to_string();
                self.tasks.push(task.stop(end)?);
                Ok(name)
            }
        }
    }

    /// Resumes the last task.
    pub fn resume_last_task(&mut self, start: DateTime<Local>) -> crate::Result<String> {
        let task = self.tasks.pop();
        match task {
            None => Err(TaskError::NoTasksFound),
            Some(task) => {
                let name = task.name().to_string();
                self.tasks.push(task.start(start)?);
                Ok(name)
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
        let max_length = self.tasks.iter().map(|task| task.name().len()).max().unwrap_or(0).max(5);
        for task in &self.tasks {
            let time = task.time_spent(time);
            let minutes = time.num_minutes() % 60;
            let hours = time.num_hours();
            let percent = (time.num_milliseconds() as f64 * 100.0 / total.num_milliseconds() as f64).round() as u32;
            if task.is_running() {
                report += &format!("    {:<max_length$} | {hours:0>2}:{minutes:0>2} | {percent:>3}%\n", task.name()).green().bold().to_string();
            } else {
                report += &format!("    {:<max_length$} | {hours:0>2}:{minutes:0>2} | {percent:>3}%\n", task.name());
            }
        }
        let minutes = total.num_minutes() % 60;
        let hours = total.num_hours();
        report += &format!("    {:=>1$}\n", "", max_length + 15);
        report += &format!("    {:<max_length$} | {hours:0>2}:{minutes:0>2} | 100%\n", "Total");
        report
    }

    pub fn rename_task(&mut self, task_name: String, new_name: String) -> crate::Result<(String, String)> {
        let mut tasks: Vec<_> = self.tasks.iter_mut().filter(|task| task.name().contains(&task_name)).collect();
        if tasks.len() >= 2 { 
            return Err(TaskError::MultipleTasksFound)
        }
        let task = tasks.pop();
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
    tasks: Vec<Task>
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
        let tasks = value.tasks;
        let size = tasks.len();
        if tasks.iter().take(size - 1).any(|task| task.is_running()) {
            Err("Only the last task can be running.".to_string())
        } else {
            Ok(TaskManager { tasks })
        }
    }
}
