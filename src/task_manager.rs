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
#[cfg(test)]
mod test;

use std::mem;
use chrono::{DateTime, Duration, Local, NaiveDate};
use colored::Colorize;
use serde::{Deserialize, Deserializer, Serialize};
use serde::de;
use crate::TaskError;

/// Task structure representing a running task.
///
/// ### Contract
/// - segments must be in chronological order.
/// - current must be after the end of the last of segments.
#[derive(Debug, Serialize, PartialEq, Clone)]
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
    pub fn stop(self, end: DateTime<Local>) -> StoppedTask {
        let start = self.current;
        StoppedTask {
            name: self.name,
            segments: self.segments,
            last_segment: Segment::new(start, end),
        }
    }
    
    /// Calculates the total time spent on the task.
    pub fn time_spent(&self, now: DateTime<Local>) -> Duration {
        self.segments.iter().fold(Duration::zero(), |total, segment| total + segment.duration()) +
                    (now - self.current)
    }
}

/// Helper for deserializing a running task.
#[derive(Debug, Deserialize)]
struct RunningTaskDeser {
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

/// Task structure representing a stopped task.
/// 
/// ### Contract
/// - segments must be in chronological order.
/// - last_segment must start after the end of the last of segments.
#[derive(Debug, Serialize, PartialEq, Clone)]
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

    /// Returns the last stop time of the task.
    pub fn stop_time(&self) -> DateTime<Local> {
        self.last_segment.end
    }

    /// Calculates the total time spent on the task.
    pub fn time_spent(&self) -> Duration {
        self.segments.iter().fold(self.last_segment.duration(), |total, segment| total + segment.duration())
    }
}

/// Helper for deserializing a stopped task.
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

/// Time segment structure representing the start and end times of work done on a task.
/// 
/// ### Contract:
/// - The start and end times are stored in chronological order.
#[derive(Debug, Serialize, PartialEq, Clone)]
pub struct Segment {
    start: DateTime<Local>,
    end: DateTime<Local>,
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
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct TaskManager { 
    tasks: Vec<StoppedTask>,
    current: Option<RunningTask>,
}
impl TaskManager {
    
    /// Creates a new task manager.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the currently running task if any.
    pub fn current_task(&self) -> Option<&str> {
        self.current.as_ref().map(|task| task.name.as_str())
    }
    
    /// Checks if there is a current task. Returns [Err] if there is one.
    fn check_no_current_task(&self) -> crate::Result<()> {
        match self.current_task() {
            None => Ok(()),
            Some(task_name) => Err(TaskError::TaskAlreadyRunning(task_name.to_string()))
        }
    }
    
    /// Returns the index of the task matching the given predicate if any. If there are multiple, returns [Err].
    fn index_of(&self, f: impl Fn(&StoppedTask) -> bool) -> crate::Result<Option<usize>> {
        let mut indices: Vec<_> = self.tasks.iter().enumerate().filter(|(_, task)| f(task)).map(|(i, _)| i).collect();
        let index = indices.pop();
        if !indices.is_empty() {
            Err(TaskError::MultipleTasksFound)
        } else {
            Ok(index)
        }
    }
    
    /// Starts a new task with the given name.
    pub fn start_new_task(&mut self, task_name: String, start: DateTime<Local>) -> crate::Result<String> {
        self.check_no_current_task()?;
        match self.index_of(|task| task.name == task_name)? {
            None => Ok(self.do_start_new_task(task_name, start)),
            Some(_) => Err(TaskError::TaskAlreadyExists(task_name))
        }
    }
    
    /// Starts a new task with the given name without performing any checks.
    fn do_start_new_task(&mut self, task_name: String, start: DateTime<Local>) -> String {
        let new_task = RunningTask::new(task_name.clone(), start);
        self.current = Some(new_task);
        task_name
    }

    /// Stops the current task.
    pub fn stop_current_task_with_time(&mut self, end: DateTime<Local>) -> crate::Result<String> {
        match self.current.take() {
            None => Err(TaskError::TaskNotRunning),
            Some(task) => {
                let name = task.name.to_string();
                self.tasks.push(task.stop(end));
                Ok(name)
            }
        }
    }

    /// Stops the current task.
    pub fn stop_current_task_with_duration(&mut self, duration: Duration, now: DateTime<Local>) -> crate::Result<String> {
        match &self.current {
            None => Err(TaskError::TaskNotRunning),
            Some(task) => {
                let end = task.current + duration;
                if end > now {
                    Err(TaskError::InvalidStopTime)
                } else {
                    self.stop_current_task_with_time(end)
                }
            }
        }
    }

    /// Resumes the last task.
    pub fn resume_last_task(&mut self, start: DateTime<Local>) -> crate::Result<String> {
        self.check_no_current_task()?;
        let task = self.tasks.pop();
        match task {
            None => Err(TaskError::NoTasksFound),
            Some(task) => {
                let name = task.name.to_string();
                self.current = Some(task.start(start));
                Ok(name)
            }
        }
    }

    /// Resumes an existing task with the given name.
    pub fn resume_task(&mut self, task_name: String, start: DateTime<Local>) -> crate::Result<String> {
        self.check_no_current_task()?;
        match self.index_of(|task| task.name.contains(&task_name))? {
            None => Err(TaskError::TaskNotFound(task_name)),
            Some(index) => Ok(self.do_resume_task(index, start)),
        }
    }

    /// Resumes an existing task at the given index without performing any checks.
    fn do_resume_task(&mut self, index: usize, start: DateTime<Local>) -> String {
        let task = self.tasks.remove(index);
        let task_name = task.name.clone();
        self.current = Some(task.start(start));
        task_name
    }

    /// Stops the current task and starts a new one.
    pub fn switch_new_task(&mut self, task_name: String, now: DateTime<Local>) -> crate::Result<String> {
        match self.index_of(|task| task.name == task_name)? {
            Some(_) => Err(TaskError::TaskAlreadyExists(task_name)),
            None => {
                self.stop_current_task_with_time(now)?;
                let task = self.do_start_new_task(task_name, now);
                Ok(task)
            }
        }
    }

    /// Stops the current task and starts a new one.
    pub fn switch_last_task(&mut self, now: DateTime<Local>) -> crate::Result<String> {
        match self.tasks.len() {
            0 => Err(TaskError::NoTasksFound),
            len => {
                self.stop_current_task_with_time(now)?;
                let task = self.do_resume_task(len - 1, now);
                Ok(task)
            }
        }
    }

    /// Stops the current task and resumes the given one.
    pub fn switch_task(&mut self, task_name: String, now: DateTime<Local>) -> crate::Result<String> {
        match self.index_of(|task| task.name.contains(&task_name))? {
            None => Err(TaskError::TaskNotFound(task_name)),
            Some(index) => {
                self.stop_current_task_with_time(now)?;
                let task = self.do_resume_task(index, now);
                Ok(task)
            }
        }
    }

    /// Deletes the given task.
    pub fn delete_task(&mut self, task_name: String) -> crate::Result<String> {
        let index = self.index_of(|task| task.name.contains(&task_name))?;
        let current_task = self.current.as_ref().filter(|task| task.name.contains(&task_name));
        match (index, current_task) {
            (None, None) => Err(TaskError::TaskNotFound(task_name)),
            (Some(index), None) => {
                let task = self.tasks.remove(index);
                Ok(task.name)
            },
            (None, Some(_)) => {
                let task = self.current.take().expect("Should exist since current_task is Some");
                Ok(task.name)
            },
            _ => Err(TaskError::MultipleTasksFound)
        }
    }

    /// Renames the given task.
    pub fn rename_task(&mut self, task_name: String, new_name: String) -> crate::Result<(String, String)> {
        let mut tasks: Vec<_> = self.tasks.iter_mut().filter(|task| task.name.contains(&task_name)).collect();
        let task = tasks.pop();
        if !tasks.is_empty() {
            return Err(TaskError::MultipleTasksFound);
        }
        let current_task = self.current.as_mut().filter(|task| task.name.contains(&task_name));
        match (task, current_task) {
            (None, None) => Err(TaskError::TaskNotFound(task_name)),
            (Some(task), None) => {
                let task_name = mem::replace(&mut task.name, new_name.clone());
                Ok((task_name, new_name))
            },
            (None, Some(task)) => {
                let task_name = mem::replace(&mut task.name, new_name.clone());
                Ok((task_name, new_name))
            },
            _ => Err(TaskError::MultipleTasksFound)
        }
    }

    /// Returns a list of all tasks.
    pub fn list_tasks(&self) -> Vec<&str> {
        let mut tasks: Vec<_> = self.tasks.iter().map(|task| task.name.as_str()).collect();
        if let Some(task) = &self.current {
            tasks.push(task.name.as_str());
        }
        tasks
    }

    /// Generates a report of the tasks.
    pub fn generate_report(&self, date: NaiveDate, time: DateTime<Local>) -> String {
        let mut report = format!("  {} \n", date.format("%F"));
        let total = self.tasks.iter().fold(self.current.as_ref().map(|task| task.time_spent(time)).unwrap_or_default(), 
                                           |total, task| total + task.time_spent());
        let max_length = self.tasks.iter().map(|task| task.name.len()).max().unwrap_or(0)
            .max(self.current.as_ref().map(|task| task.name.len()).unwrap_or(0))
            .max(5);
        for task in &self.tasks {
            let time = task.time_spent();
            let percent = percent(time.num_milliseconds() as u32, total.num_milliseconds() as u32);
            report += &format!("    {:<max_length$} | {} | {percent:>5.1}%\n", task.name, format_duration(time));
        }
        if let Some(task) = &self.current {
            let time = task.time_spent(time);
            let percent = percent(time.num_milliseconds() as u32, total.num_milliseconds() as u32);
            report += &format!("    {:<max_length$} | {} | {percent:>5.1}%\n", task.name, format_duration(time)).green().bold().to_string();
        }
        report += &format!("    {:=>1$}\n", "", max_length + 17);
        report += &format!("    {:<max_length$} | {} | 100.0%\n", "Total", format_duration(total));
        report
    }
}

/// Formats a duration in hours and minutes.
fn format_duration(duration: Duration) -> String {
    let minutes = duration.num_minutes() % 60;
    let hours = duration.num_hours();
    format!("{hours:0>2}:{minutes:0>2}")
}

/// Calculates the percentage of a number.
fn percent(numerator: u32, denominator: u32) -> f64 {
    numerator as f64 / denominator as f64 * 100.0
}
