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
use super::*;

mod running_task {
    use super::*;

    #[test]
    fn test_running_task_new() {
        let now = Local::now();
        let task = RunningTask::new("Test", now);
        assert_eq!(task.name, "Test");
        assert!(task.entries.is_empty());
        assert_eq!(task.last_entry.start, now);
    }

    #[test]
    fn test_running_task_stop() {
        let before = Local::now();
        let after = Local::now() + Duration::minutes(10);
        let task = RunningTask {
            name: "Test".to_string(),
            entries: vec![],
            last_entry: OngoingTimeEntry::new(before),
        }
        .stop(after);
        assert_eq!(task.name, "Test");
        assert!(task.entries.is_empty());
        assert_eq!(task.last_entry.start, before);
        assert_eq!(task.last_entry.end, after);
    }

    #[test]
    #[should_panic]
    fn test_running_task_stop_when_time_before_start() {
        let now = Local::now();
        let before = Local::now() - Duration::minutes(10);
        RunningTask {
            name: "Test".to_string(),
            entries: vec![],
            last_entry: OngoingTimeEntry::new(now),
        }
        .stop(before);
    }

    #[test]
    fn test_running_task_time_spent_without_entries() {
        let before = Local::now();
        let after = Local::now() + Duration::minutes(10);
        let task = RunningTask {
            name: "Test".to_string(),
            entries: vec![],
            last_entry: OngoingTimeEntry::new(before),
        };
        assert_eq!(task.time_spent(after).num_minutes(), 10);
    }

    #[test]
    fn test_running_task_time_spent_with_entries() {
        let start = Local::now();
        let task = RunningTask {
            name: "Test".to_string(),
            entries: vec![
                CompletedTimeEntry::new(start, start + Duration::minutes(1)),
                CompletedTimeEntry::new(start + Duration::minutes(2), start + Duration::minutes(4)),
                CompletedTimeEntry::new(start + Duration::minutes(5), start + Duration::minutes(8)),
            ],
            last_entry: OngoingTimeEntry::new(start + Duration::minutes(9)),
        };
        let end = start + Duration::minutes(13);
        assert_eq!(task.time_spent(end).num_minutes(), 10);
    }
}

mod stopped_task {
    use super::*;

    #[test]
    fn test_stopped_task_start() {
        let before = Local::now();
        let time_entry = CompletedTimeEntry::new(before, before + Duration::minutes(10));
        let now = Local::now() + Duration::minutes(20);
        let task = StoppedTask {
            name: "Test".to_string(),
            entries: vec![],
            last_entry: time_entry.clone(),
        }
        .start(now);
        assert_eq!(task.name, "Test");
        assert_eq!(task.entries, vec![time_entry]);
        assert_eq!(task.last_entry.start, now);
    }

    #[test]
    #[should_panic]
    fn test_stopped_task_start_earlier_time() {
        let before = Local::now();
        let time_entry = CompletedTimeEntry::new(before, before + Duration::minutes(10));
        let now = Local::now() + Duration::minutes(5);
        StoppedTask {
            name: "Test".to_string(),
            entries: vec![],
            last_entry: time_entry.clone(),
        }
        .start(now);
    }

    #[test]
    fn test_stopped_task_stop_time() {
        let start = Local::now();
        let end = start + Duration::minutes(10);
        let task = StoppedTask {
            name: "Test".to_string(),
            entries: vec![
                CompletedTimeEntry::new(start, start + Duration::minutes(1)),
                CompletedTimeEntry::new(start + Duration::minutes(2), start + Duration::minutes(4)),
                CompletedTimeEntry::new(start + Duration::minutes(5), start + Duration::minutes(8)),
            ],
            last_entry: CompletedTimeEntry::new(start + Duration::minutes(9), end),
        };
        assert_eq!(task.stop_time(), end);
    }

    #[test]
    fn test_stopped_task_time_spent_without_entries() {
        let before = Local::now();
        let after = Local::now() + Duration::minutes(10);
        let task = StoppedTask {
            name: "Test".to_string(),
            entries: vec![],
            last_entry: CompletedTimeEntry::new(before, after),
        };
        assert_eq!(task.time_spent().num_minutes(), 10);
    }

    #[test]
    fn test_stopped_task_time_spent_with_entries() {
        let start = Local::now();
        let task = StoppedTask {
            name: "Test".to_string(),
            entries: vec![
                CompletedTimeEntry::new(start, start + Duration::minutes(1)),
                CompletedTimeEntry::new(start + Duration::minutes(2), start + Duration::minutes(4)),
                CompletedTimeEntry::new(start + Duration::minutes(5), start + Duration::minutes(8)),
            ],
            last_entry: CompletedTimeEntry::new(
                start + Duration::minutes(9),
                start + Duration::minutes(13),
            ),
        };
        assert_eq!(task.time_spent().num_minutes(), 10);
    }
}

mod ongoing_time_entry {
    use super::*;

    #[test]
    fn test_ongoing_time_entry_new() {
        let now = Local::now();
        let time_entry = OngoingTimeEntry::new(now);
        assert_eq!(time_entry.start, now);
    }

    #[test]
    fn test_ongoing_time_entry_duration() {
        let now = Local::now();
        let time_entry = OngoingTimeEntry::new(now);
        assert_eq!(
            time_entry
                .duration(now + Duration::minutes(10))
                .num_minutes(),
            10
        );
    }

    #[test]
    #[should_panic]
    fn test_ongoing_time_entry_duration_earlier_time() {
        let now = Local::now();
        let time_entry = OngoingTimeEntry::new(now);
        time_entry
            .duration(now - Duration::minutes(10))
            .num_minutes();
    }

    #[test]
    fn test_ongoing_time_entry_complete() {
        let now = Local::now();
        let time_entry = OngoingTimeEntry::new(now);
        let time_entry = time_entry.complete(now + Duration::minutes(10));
        assert_eq!(time_entry.start, now);
        assert_eq!(time_entry.end, now + Duration::minutes(10));
    }

    #[test]
    #[should_panic]
    fn test_ongoing_time_entry_complete_earlier() {
        let now = Local::now();
        let time_entry = OngoingTimeEntry::new(now);
        time_entry.complete(now - Duration::minutes(10));
    }
}

mod completed_time_entry {
    use super::*;

    #[test]
    fn test_completed_time_entry_new() {
        let start = Local::now();
        let end = start + Duration::minutes(10);
        let time_entry = CompletedTimeEntry::new(start, end);
        assert_eq!(time_entry.start, start);
        assert_eq!(time_entry.end, end);
    }

    #[test]
    #[should_panic]
    fn test_completed_time_entry_new_earlier_time() {
        let start = Local::now();
        let end = start - Duration::minutes(10);
        CompletedTimeEntry::new(start, end);
    }

    #[test]
    fn test_completed_time_entry_duration() {
        let start = Local::now();
        let end = start + Duration::minutes(10);
        let time_entry = CompletedTimeEntry { start, end };
        assert_eq!(time_entry.duration().num_minutes(), 10);
    }
}

mod task_manager {
    use super::*;

    #[test]
    fn test_task_manager_new() {
        let task_manager = TaskManager::new();
        assert!(task_manager.stopped.is_empty());
        assert!(task_manager.running.is_none());
    }

    #[test]
    fn test_task_manager_start_new_task_when_none_exist() {
        let mut task_manager = TaskManager {
            stopped: vec![],
            running: None,
        };
        let now = Local::now();
        let result = task_manager.start_new_task("Test".to_string(), now);
        assert_eq!(task_manager.running, Some(RunningTask::new("Test", now)));
        assert!(task_manager.stopped.is_empty());
        assert!(result.is_ok());
        let task_name = result.unwrap();
        assert_eq!(task_name, "Test");
    }

    #[test]
    fn test_task_manager_start_new_task_when_other_exists() {
        let now = Local::now();
        let stopped_task = StoppedTask {
            name: "OtherTest".to_string(),
            entries: vec![],
            last_entry: CompletedTimeEntry::new(now, now + Duration::minutes(10)),
        };
        let mut task_manager = TaskManager {
            stopped: vec![stopped_task.clone()],
            running: None,
        };
        let result = task_manager.start_new_task("Test".to_string(), now + Duration::minutes(20));
        assert_eq!(
            task_manager.running,
            Some(RunningTask::new("Test", now + Duration::minutes(20)))
        );
        assert_eq!(task_manager.stopped, vec![stopped_task]);
        assert!(result.is_ok());
        let task_name = result.unwrap();
        assert_eq!(task_name, "Test");
    }

    #[test]
    fn test_task_manager_start_new_task_when_already_running() {
        let now = Local::now();
        let task = RunningTask::new("Test", now);
        let mut task_manager = TaskManager {
            stopped: vec![],
            running: Some(task.clone()),
        };
        let result = task_manager.start_new_task("Test2".to_string(), now + Duration::minutes(10));
        assert_eq!(task_manager.running, Some(task));
        assert!(task_manager.stopped.is_empty());
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, TaskError::TaskAlreadyRunning(name) if name == "Test"));
    }

    #[test]
    fn test_task_manager_start_new_task_when_already_exists() {
        let now = Local::now();
        let stopped_task = StoppedTask {
            name: "Test".to_string(),
            entries: vec![],
            last_entry: CompletedTimeEntry::new(now, now + Duration::minutes(10)),
        };
        let mut task_manager = TaskManager {
            stopped: vec![stopped_task.clone()],
            running: None,
        };
        let result = task_manager.start_new_task("Test".to_string(), now + Duration::minutes(20));
        assert!(task_manager.running.is_none());
        assert_eq!(task_manager.stopped, vec![stopped_task]);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, TaskError::TaskAlreadyExists(name) if name == "Test"));
    }

    #[test]
    fn test_task_manager_stop_running_task_with_time() {
        let now = Local::now();
        let task = RunningTask::new("Test", now);
        let mut task_manager = TaskManager {
            stopped: vec![],
            running: Some(task.clone()),
        };
        let result = task_manager.stop_running_task_with_time(now + Duration::minutes(10));
        assert!(task_manager.running.is_none());
        assert_eq!(
            task_manager.stopped,
            vec![task.stop(now + Duration::minutes(10))]
        );
        assert!(result.is_ok());
        let task_name = result.unwrap();
        assert_eq!(task_name, "Test");
    }

    #[test]
    fn test_task_manager_stop_running_task_with_time_when_end_before_start() {
        let now = Local::now();
        let task = RunningTask::new("Test", now);
        let mut task_manager = TaskManager {
            stopped: vec![],
            running: Some(task.clone()),
        };
        let result = task_manager.stop_running_task_with_time(now - Duration::minutes(10));
        assert_eq!(task_manager.running, Some(task));
        assert!(task_manager.stopped.is_empty());
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, TaskError::InvalidStopTime));
    }

    #[test]
    fn test_task_manager_stop_running_task_with_time_when_no_running() {
        let now = Local::now();
        let mut task_manager = TaskManager {
            stopped: vec![],
            running: None,
        };
        let result = task_manager.stop_running_task_with_time(now + Duration::minutes(10));
        assert!(task_manager.running.is_none());
        assert!(task_manager.stopped.is_empty());
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, TaskError::TaskNotRunning));
    }

    #[test]
    fn test_task_manager_stop_running_task_with_duration() {
        let now = Local::now();
        let task = RunningTask::new("Test", now);
        let mut task_manager = TaskManager {
            stopped: vec![],
            running: Some(task.clone()),
        };
        let result = task_manager
            .stop_running_task_with_duration(Duration::minutes(10), now + Duration::minutes(20));
        assert!(task_manager.running.is_none());
        assert_eq!(
            task_manager.stopped,
            vec![task.stop(now + Duration::minutes(10))]
        );
        assert!(result.is_ok());
        let task_name = result.unwrap();
        assert_eq!(task_name, "Test");
    }

    #[test]
    fn test_task_manager_stop_running_task_with_duration_when_no_running() {
        let now = Local::now();
        let mut task_manager = TaskManager {
            stopped: vec![],
            running: None,
        };
        let result = task_manager
            .stop_running_task_with_duration(Duration::minutes(10), now + Duration::minutes(20));
        assert!(task_manager.running.is_none());
        assert!(task_manager.stopped.is_empty());
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, TaskError::TaskNotRunning));
    }

    #[test]
    fn test_task_manager_stop_running_task_with_duration_when_end_in_future() {
        let now = Local::now();
        let task = RunningTask::new("Test", now);
        let mut task_manager = TaskManager {
            stopped: vec![],
            running: Some(task.clone()),
        };
        let result = task_manager
            .stop_running_task_with_duration(Duration::minutes(20), now + Duration::minutes(10));
        assert_eq!(task_manager.running, Some(task));
        assert!(task_manager.stopped.is_empty());
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, TaskError::InvalidStopTime));
    }

    #[test]
    fn test_task_manager_resume_last_task_when_no_tasks() {
        let mut task_manager = TaskManager {
            stopped: vec![],
            running: None,
        };
        let result = task_manager.resume_last_task(Local::now());
        assert!(task_manager.running.is_none());
        assert!(task_manager.stopped.is_empty());
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, TaskError::NoTasksFound));
    }

    #[test]
    fn test_task_manager_resume_last_task_when_already_running() {
        let now = Local::now();
        let task = RunningTask::new("Test", now);
        let mut task_manager = TaskManager {
            stopped: vec![],
            running: Some(task.clone()),
        };
        let result = task_manager.resume_last_task(now + Duration::minutes(10));
        assert!(task_manager.stopped.is_empty());
        assert_eq!(task_manager.running, Some(task));
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, TaskError::TaskAlreadyRunning(name) if name == "Test"));
    }

    #[test]
    fn test_task_manager_resume_last_task_when_none_running_and_now_before_start() {
        let now = Local::now();
        let time_entry1 = CompletedTimeEntry::new(now, now + Duration::minutes(5));
        let time_entry2 =
            CompletedTimeEntry::new(now + Duration::minutes(6), now + Duration::minutes(7));
        let task1 = StoppedTask {
            name: "Test1".to_string(),
            entries: vec![],
            last_entry: time_entry1.clone(),
        };
        let task2 = StoppedTask {
            name: "Test2".to_string(),
            entries: vec![],
            last_entry: time_entry2.clone(),
        };
        let mut task_manager = TaskManager {
            stopped: vec![task1.clone(), task2.clone()],
            running: None,
        };
        let result = task_manager.resume_last_task(now - Duration::minutes(10));
        assert_eq!(task_manager.stopped, vec![task1, task2]);
        assert_eq!(task_manager.running, None);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, TaskError::InvalidStartTime));
    }

    #[test]
    fn test_task_manager_resume_last_task_when_none_running() {
        let now = Local::now();
        let time_entry1 = CompletedTimeEntry::new(now, now + Duration::minutes(5));
        let time_entry2 =
            CompletedTimeEntry::new(now + Duration::minutes(6), now + Duration::minutes(7));
        let task1 = StoppedTask {
            name: "Test1".to_string(),
            entries: vec![],
            last_entry: time_entry1.clone(),
        };
        let task2 = StoppedTask {
            name: "Test2".to_string(),
            entries: vec![],
            last_entry: time_entry2.clone(),
        };
        let mut task_manager = TaskManager {
            stopped: vec![task1.clone(), task2.clone()],
            running: None,
        };
        let result = task_manager.resume_last_task(now + Duration::minutes(10));
        assert_eq!(task_manager.stopped, vec![task1]);
        assert_eq!(
            task_manager.running,
            Some(task2.start(now + Duration::minutes(10)))
        );
        assert!(result.is_ok());
        let task_name = result.unwrap();
        assert_eq!(task_name, "Test2");
    }

    #[test]
    fn test_task_manager_resume_task_when_no_tasks() {
        let mut task_manager = TaskManager {
            stopped: vec![],
            running: None,
        };
        let result = task_manager.resume_task("Test".to_string(), Local::now());
        assert!(task_manager.running.is_none());
        assert!(task_manager.stopped.is_empty());
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, TaskError::TaskNotFound(name) if name == "Test"));
    }

    #[test]
    fn test_task_manager_resume_task_when_already_running() {
        let now = Local::now();
        let task = RunningTask::new("Test", now);
        let mut task_manager = TaskManager {
            stopped: vec![],
            running: Some(task.clone()),
        };
        let result = task_manager.resume_task("Test".to_string(), now + Duration::minutes(10));
        assert!(task_manager.stopped.is_empty());
        assert_eq!(task_manager.running, Some(task));
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, TaskError::TaskAlreadyRunning(name) if name == "Test"));
    }

    #[test]
    fn test_task_manager_resume_task_when_none_running() {
        let now = Local::now();
        let time_entry1 = CompletedTimeEntry::new(now, now + Duration::minutes(5));
        let time_entry2 =
            CompletedTimeEntry::new(now + Duration::minutes(6), now + Duration::minutes(7));
        let task1 = StoppedTask {
            name: "Test10".to_string(),
            entries: vec![],
            last_entry: time_entry1.clone(),
        };
        let task2 = StoppedTask {
            name: "Test2".to_string(),
            entries: vec![],
            last_entry: time_entry2.clone(),
        };
        let mut task_manager = TaskManager {
            stopped: vec![task1.clone(), task2.clone()],
            running: None,
        };
        let result = task_manager.resume_task("Test1".to_string(), now + Duration::minutes(10));
        assert_eq!(task_manager.stopped, vec![task2]);
        assert_eq!(
            task_manager.running,
            Some(task1.start(now + Duration::minutes(10)))
        );
        assert!(result.is_ok());
        let task_name = result.unwrap();
        assert_eq!(task_name, "Test10");
    }

    #[test]
    fn test_task_manager_resume_task_when_none_running_and_now_before_start() {
        let now = Local::now();
        let time_entry1 = CompletedTimeEntry::new(now, now + Duration::minutes(5));
        let time_entry2 =
            CompletedTimeEntry::new(now + Duration::minutes(6), now + Duration::minutes(7));
        let task1 = StoppedTask {
            name: "Test10".to_string(),
            entries: vec![],
            last_entry: time_entry1.clone(),
        };
        let task2 = StoppedTask {
            name: "Test2".to_string(),
            entries: vec![],
            last_entry: time_entry2.clone(),
        };
        let mut task_manager = TaskManager {
            stopped: vec![task1.clone(), task2.clone()],
            running: None,
        };
        let result = task_manager.resume_task("Test1".to_string(), now - Duration::minutes(10));
        assert_eq!(task_manager.stopped, vec![task1, task2]);
        assert_eq!(task_manager.running, None);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, TaskError::InvalidStartTime));
    }

    #[test]
    fn test_task_manager_resume_task_when_none_running_and_ambiguous_name() {
        let now = Local::now();
        let time_entry1 = CompletedTimeEntry::new(now, now + Duration::minutes(5));
        let time_entry2 =
            CompletedTimeEntry::new(now + Duration::minutes(6), now + Duration::minutes(7));
        let task1 = StoppedTask {
            name: "Test1".to_string(),
            entries: vec![],
            last_entry: time_entry1.clone(),
        };
        let task2 = StoppedTask {
            name: "Test2".to_string(),
            entries: vec![],
            last_entry: time_entry2.clone(),
        };
        let mut task_manager = TaskManager {
            stopped: vec![task1.clone(), task2.clone()],
            running: None,
        };
        let result = task_manager.resume_task("Test".to_string(), now + Duration::minutes(10));
        assert_eq!(task_manager.stopped, vec![task1, task2]);
        assert_eq!(task_manager.running, None);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, TaskError::MultipleTasksFound));
    }

    #[test]
    fn test_task_manager_switch_new_task_when_none_running() {
        let mut task_manager = TaskManager {
            stopped: vec![],
            running: None,
        };
        let result = task_manager.switch_new_task("Test".to_string(), Local::now());
        assert!(task_manager.running.is_none());
        assert!(task_manager.stopped.is_empty());
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, TaskError::TaskNotRunning));
    }

    #[test]
    fn test_task_manager_switch_new_task_when_already_exists() {
        let now = Local::now();
        let task1 = StoppedTask {
            name: "Test".to_string(),
            entries: vec![],
            last_entry: CompletedTimeEntry::new(now, now + Duration::minutes(10)),
        };
        let task2 = RunningTask::new("Test2", now + Duration::minutes(15));
        let mut task_manager = TaskManager {
            stopped: vec![task1.clone()],
            running: Some(task2.clone()),
        };
        let result = task_manager.switch_new_task("Test".to_string(), now + Duration::minutes(20));
        assert_eq!(task_manager.running, Some(task2));
        assert_eq!(task_manager.stopped, vec![task1]);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, TaskError::TaskAlreadyExists(name) if name == "Test"));
    }

    #[test]
    fn test_task_manager_switch_new_task_when_not_exists() {
        let now = Local::now();
        let task1 = StoppedTask {
            name: "Test1".to_string(),
            entries: vec![],
            last_entry: CompletedTimeEntry::new(now, now + Duration::minutes(10)),
        };
        let task2 = RunningTask::new("Test2", now + Duration::minutes(15));
        let mut task_manager = TaskManager {
            stopped: vec![task1.clone()],
            running: Some(task2.clone()),
        };
        let result = task_manager.switch_new_task("Test".to_string(), now + Duration::minutes(20));
        assert_eq!(
            task_manager.running,
            Some(RunningTask::new("Test", now + Duration::minutes(20)))
        );
        assert_eq!(
            task_manager.stopped,
            vec![task1, task2.stop(now + Duration::minutes(20))]
        );
        assert!(result.is_ok());
        let task_name = result.unwrap();
        assert_eq!(task_name, "Test");
    }

    #[test]
    fn test_task_manager_switch_last_task_when_no_tasks() {
        let mut task_manager = TaskManager {
            stopped: vec![],
            running: None,
        };
        let result = task_manager.switch_last_task(Local::now());
        assert!(task_manager.running.is_none());
        assert!(task_manager.stopped.is_empty());
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, TaskError::NoTasksFound));
    }

    #[test]
    fn test_task_manager_switch_last_task_when_none_running() {
        let now = Local::now();
        let task = StoppedTask {
            name: "Test".to_string(),
            entries: vec![],
            last_entry: CompletedTimeEntry::new(now, now + Duration::minutes(10)),
        };
        let mut task_manager = TaskManager {
            stopped: vec![task.clone()],
            running: None,
        };
        let result = task_manager.switch_last_task(now + Duration::minutes(20));
        assert!(task_manager.running.is_none());
        assert_eq!(task_manager.stopped, vec![task]);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, TaskError::TaskNotRunning));
    }

    #[test]
    fn test_task_manager_switch_last_task_when_none_stopped() {
        let now = Local::now();
        let task = RunningTask::new("Test", now);
        let mut task_manager = TaskManager {
            stopped: vec![],
            running: Some(task.clone()),
        };
        let result = task_manager.switch_last_task(now + Duration::minutes(20));
        assert_eq!(task_manager.running, Some(task));
        assert!(task_manager.stopped.is_empty());
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, TaskError::NoTasksFound));
    }

    #[test]
    fn test_task_manager_switch_last_task() {
        let now = Local::now();
        let task1 = StoppedTask {
            name: "Test1".to_string(),
            entries: vec![],
            last_entry: CompletedTimeEntry::new(now, now + Duration::minutes(10)),
        };
        let task2 = RunningTask::new("Test2", now + Duration::minutes(15));
        let mut task_manager = TaskManager {
            stopped: vec![task1.clone()],
            running: Some(task2.clone()),
        };
        let result = task_manager.switch_last_task(now + Duration::minutes(20));
        assert_eq!(
            task_manager.running,
            Some(task1.start(now + Duration::minutes(20)))
        );
        assert_eq!(
            task_manager.stopped,
            vec![task2.stop(now + Duration::minutes(20))]
        );
        assert!(result.is_ok());
        let task_name = result.unwrap();
        assert_eq!(task_name, "Test1");
    }

    #[test]
    fn test_task_manager_switch_last_task_when_before_start() {
        let now = Local::now();
        let task1 = StoppedTask {
            name: "Test1".to_string(),
            entries: vec![],
            last_entry: CompletedTimeEntry::new(now, now + Duration::minutes(10)),
        };
        let task2 = RunningTask::new("Test2", now + Duration::minutes(15));
        let mut task_manager = TaskManager {
            stopped: vec![task1.clone()],
            running: Some(task2.clone()),
        };
        let result = task_manager.switch_last_task(now + Duration::minutes(12));
        assert_eq!(task_manager.running, Some(task2));
        assert_eq!(task_manager.stopped, vec![task1]);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, TaskError::InvalidStopTime));
    }

    #[test]
    fn test_task_manager_switch_last_task_when_before_end() {
        let now = Local::now();
        let task1 = StoppedTask {
            name: "Test1".to_string(),
            entries: vec![],
            last_entry: CompletedTimeEntry::new(now, now + Duration::minutes(15)),
        };
        let task2 = RunningTask::new("Test2", now + Duration::minutes(10));
        let mut task_manager = TaskManager {
            stopped: vec![task1.clone()],
            running: Some(task2.clone()),
        };
        let result = task_manager.switch_last_task(now + Duration::minutes(12));
        assert_eq!(task_manager.running, Some(task2));
        assert_eq!(task_manager.stopped, vec![task1]);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, TaskError::InvalidStartTime));
    }

    #[test]
    fn test_task_manager_switch_task_when_not_exists() {
        let mut task_manager = TaskManager {
            stopped: vec![],
            running: None,
        };
        let result = task_manager.switch_task("Test".to_string(), Local::now());
        assert!(task_manager.running.is_none());
        assert!(task_manager.stopped.is_empty());
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, TaskError::TaskNotFound(name) if name == "Test"));
    }

    #[test]
    fn test_task_manager_switch_task_when_none_running() {
        let now = Local::now();
        let stopped_task = StoppedTask {
            name: "Test".to_string(),
            entries: vec![],
            last_entry: CompletedTimeEntry::new(now, now + Duration::minutes(10)),
        };
        let mut task_manager = TaskManager {
            stopped: vec![stopped_task.clone()],
            running: None,
        };
        let result = task_manager.switch_task("Test".to_string(), now + Duration::minutes(20));
        assert!(task_manager.running.is_none());
        assert_eq!(task_manager.stopped, vec![stopped_task]);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, TaskError::TaskNotRunning));
    }

    #[test]
    fn test_task_manager_switch_task_when_already_running() {
        let now = Local::now();
        let task1 = StoppedTask {
            name: "Test1".to_string(),
            entries: vec![],
            last_entry: CompletedTimeEntry::new(now, now + Duration::minutes(5)),
        };
        let task2 = StoppedTask {
            name: "Test2".to_string(),
            entries: vec![],
            last_entry: CompletedTimeEntry::new(
                now + Duration::minutes(5),
                now + Duration::minutes(10),
            ),
        };
        let task3 = RunningTask::new("Test3", now + Duration::minutes(15));
        let mut task_manager = TaskManager {
            stopped: vec![task1.clone(), task2.clone()],
            running: Some(task3.clone()),
        };
        let result = task_manager.switch_task("Test3".to_string(), now + Duration::minutes(20));
        assert_eq!(task_manager.running, Some(task3));
        assert_eq!(task_manager.stopped, vec![task1, task2]);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, TaskError::TaskNotFound(name) if name == "Test3"));
    }

    #[test]
    fn test_task_manager_switch_task_when_other_running() {
        let now = Local::now();
        let task1 = StoppedTask {
            name: "Test1".to_string(),
            entries: vec![],
            last_entry: CompletedTimeEntry::new(now, now + Duration::minutes(5)),
        };
        let task2 = StoppedTask {
            name: "Test2".to_string(),
            entries: vec![],
            last_entry: CompletedTimeEntry::new(
                now + Duration::minutes(5),
                now + Duration::minutes(10),
            ),
        };
        let task3 = RunningTask::new("Test3", now + Duration::minutes(15));
        let mut task_manager = TaskManager {
            stopped: vec![task1.clone(), task2.clone()],
            running: Some(task3.clone()),
        };
        let result = task_manager.switch_task("Test1".to_string(), now + Duration::minutes(20));
        assert_eq!(
            task_manager.running,
            Some(task1.start(now + Duration::minutes(20)))
        );
        assert_eq!(
            task_manager.stopped,
            vec![task2, task3.stop(now + Duration::minutes(20))]
        );
        assert!(result.is_ok());
        let task_name = result.unwrap();
        assert_eq!(task_name, "Test1");
    }

    #[test]
    fn test_task_manager_switch_task_when_other_running_when_before_start() {
        let now = Local::now();
        let task1 = StoppedTask {
            name: "Test1".to_string(),
            entries: vec![],
            last_entry: CompletedTimeEntry::new(now, now + Duration::minutes(5)),
        };
        let task2 = RunningTask::new("Test2", now + Duration::minutes(15));
        let mut task_manager = TaskManager {
            stopped: vec![task1.clone()],
            running: Some(task2.clone()),
        };
        let result = task_manager.switch_task("Test1".to_string(), now + Duration::minutes(10));
        assert_eq!(task_manager.running, Some(task2));
        assert_eq!(task_manager.stopped, vec![task1]);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, TaskError::InvalidStopTime));
    }

    #[test]
    fn test_task_manager_switch_task_when_other_running_when_before_end() {
        let now = Local::now();
        let task1 = StoppedTask {
            name: "Test1".to_string(),
            entries: vec![],
            last_entry: CompletedTimeEntry::new(now, now + Duration::minutes(15)),
        };
        let task2 = RunningTask::new("Test2", now + Duration::minutes(5));
        let mut task_manager = TaskManager {
            stopped: vec![task1.clone()],
            running: Some(task2.clone()),
        };
        let result = task_manager.switch_task("Test1".to_string(), now + Duration::minutes(10));
        assert_eq!(task_manager.running, Some(task2));
        assert_eq!(task_manager.stopped, vec![task1]);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, TaskError::InvalidStartTime));
    }

    #[test]
    fn test_task_manager_switch_task_when_ambiguous_name1() {
        let now = Local::now();
        let task1 = StoppedTask {
            name: "Test1".to_string(),
            entries: vec![],
            last_entry: CompletedTimeEntry::new(now, now + Duration::minutes(5)),
        };
        let task2 = StoppedTask {
            name: "Test2".to_string(),
            entries: vec![],
            last_entry: CompletedTimeEntry::new(
                now + Duration::minutes(5),
                now + Duration::minutes(10),
            ),
        };
        let task3 = RunningTask::new("Abc", now + Duration::minutes(15));
        let mut task_manager = TaskManager {
            stopped: vec![task1.clone(), task2.clone()],
            running: Some(task3.clone()),
        };
        let result = task_manager.switch_task("Test".to_string(), now + Duration::minutes(20));
        assert_eq!(task_manager.running, Some(task3));
        assert_eq!(task_manager.stopped, vec![task1, task2]);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, TaskError::MultipleTasksFound));
    }

    #[test]
    fn test_task_manager_switch_task_when_ambiguous_name2() {
        let now = Local::now();
        let task1 = StoppedTask {
            name: "Test1".to_string(),
            entries: vec![],
            last_entry: CompletedTimeEntry::new(now, now + Duration::minutes(5)),
        };
        let task2 = StoppedTask {
            name: "Abc".to_string(),
            entries: vec![],
            last_entry: CompletedTimeEntry::new(
                now + Duration::minutes(5),
                now + Duration::minutes(10),
            ),
        };
        let task3 = RunningTask::new("Test2", now + Duration::minutes(15));
        let mut task_manager = TaskManager {
            stopped: vec![task1.clone(), task2.clone()],
            running: Some(task3.clone()),
        };
        let result = task_manager.switch_task("Test".to_string(), now + Duration::minutes(20));
        assert_eq!(
            task_manager.running,
            Some(task1.start(now + Duration::minutes(20)))
        );
        assert_eq!(
            task_manager.stopped,
            vec![task2, task3.stop(now + Duration::minutes(20))]
        );
        assert!(result.is_ok());
        let task_name = result.unwrap();
        assert_eq!(task_name, "Test1");
    }

    #[test]
    fn test_task_manager_list_when_no_tasks() {
        let task_manager = TaskManager {
            stopped: vec![],
            running: None,
        };
        let result = task_manager.list_tasks();
        assert!(result.is_empty());
    }

    #[test]
    fn test_task_manager_list_when_tasks() {
        let now = Local::now();
        let task1 = StoppedTask {
            name: "Test1".to_string(),
            entries: vec![],
            last_entry: CompletedTimeEntry::new(now, now + Duration::minutes(5)),
        };
        let task2 = StoppedTask {
            name: "Test2".to_string(),
            entries: vec![],
            last_entry: CompletedTimeEntry::new(
                now + Duration::minutes(5),
                now + Duration::minutes(10),
            ),
        };
        let task3 = RunningTask::new("Test3", now + Duration::minutes(15));
        let task_manager = TaskManager {
            stopped: vec![task1.clone(), task2.clone()],
            running: Some(task3.clone()),
        };
        let result = task_manager.list_tasks();
        assert_eq!(result, vec!["Test1", "Test2", "Test3"]);
    }

    #[test]
    fn test_task_manager_rename_task_when_no_match() {
        let now = Local::now();
        let task1 = StoppedTask {
            name: "Test1".to_string(),
            entries: vec![],
            last_entry: CompletedTimeEntry::new(now, now + Duration::minutes(5)),
        };
        let task2 = RunningTask::new("Test2", now + Duration::minutes(15));
        let mut task_manager = TaskManager {
            stopped: vec![task1.clone()],
            running: Some(task2.clone()),
        };
        let result = task_manager.rename_task("Abc".to_string(), "Test".to_string());
        assert_eq!(task_manager.running, Some(task2));
        assert_eq!(task_manager.stopped, vec![task1]);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, TaskError::TaskNotFound(_)));
    }

    #[test]
    fn test_task_manager_rename_task_when_stopped() {
        let now = Local::now();
        let mut task1 = StoppedTask {
            name: "Test10".to_string(),
            entries: vec![],
            last_entry: CompletedTimeEntry::new(now, now + Duration::minutes(5)),
        };
        let task2 = RunningTask::new("Test2", now + Duration::minutes(15));
        let mut task_manager = TaskManager {
            stopped: vec![task1.clone()],
            running: Some(task2.clone()),
        };
        let result = task_manager.rename_task("Test1".to_string(), "Test".to_string());
        task1.name = "Test".to_string();
        assert_eq!(task_manager.running, Some(task2));
        assert_eq!(task_manager.stopped, vec![task1]);
        assert!(result.is_ok());
        let (old, new) = result.unwrap();
        assert_eq!("Test10", old);
        assert_eq!("Test", new);
    }

    #[test]
    fn test_task_manager_rename_task_when_running() {
        let now = Local::now();
        let task1 = StoppedTask {
            name: "Test1".to_string(),
            entries: vec![],
            last_entry: CompletedTimeEntry::new(now, now + Duration::minutes(5)),
        };
        let mut task2 = RunningTask::new("Test20", now + Duration::minutes(15));
        let mut task_manager = TaskManager {
            stopped: vec![task1.clone()],
            running: Some(task2.clone()),
        };
        let result = task_manager.rename_task("Test2".to_string(), "Test".to_string());
        task2.name = "Test".to_string();
        assert_eq!(task_manager.running, Some(task2));
        assert_eq!(task_manager.stopped, vec![task1]);
        assert!(result.is_ok());
        let (old, new) = result.unwrap();
        assert_eq!("Test20", old);
        assert_eq!("Test", new);
    }

    #[test]
    fn test_task_manager_rename_task_when_ambiguous_name1() {
        let now = Local::now();
        let task1 = StoppedTask {
            name: "Test1".to_string(),
            entries: vec![],
            last_entry: CompletedTimeEntry::new(now, now + Duration::minutes(5)),
        };
        let task2 = StoppedTask {
            name: "Test2".to_string(),
            entries: vec![],
            last_entry: CompletedTimeEntry::new(
                now + Duration::minutes(5),
                now + Duration::minutes(10),
            ),
        };
        let mut task_manager = TaskManager {
            stopped: vec![task1.clone(), task2.clone()],
            running: None,
        };
        let result = task_manager.rename_task("Test".to_string(), "Abc".to_string());
        assert_eq!(task_manager.running, None);
        assert_eq!(task_manager.stopped, vec![task1, task2]);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, TaskError::MultipleTasksFound));
    }

    #[test]
    fn test_task_manager_rename_task_when_ambiguous_name2() {
        let now = Local::now();
        let task1 = StoppedTask {
            name: "Test1".to_string(),
            entries: vec![],
            last_entry: CompletedTimeEntry::new(now, now + Duration::minutes(5)),
        };
        let task2 = RunningTask::new("Test2", now + Duration::minutes(15));
        let mut task_manager = TaskManager {
            stopped: vec![task1.clone()],
            running: Some(task2.clone()),
        };
        let result = task_manager.rename_task("Test".to_string(), "Abc".to_string());
        assert_eq!(task_manager.running, Some(task2));
        assert_eq!(task_manager.stopped, vec![task1]);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, TaskError::MultipleTasksFound));
    }

    #[test]
    fn test_task_manager_delete_task_when_no_match() {
        let now = Local::now();
        let task1 = StoppedTask {
            name: "Test1".to_string(),
            entries: vec![],
            last_entry: CompletedTimeEntry::new(now, now + Duration::minutes(5)),
        };
        let task2 = RunningTask::new("Test2", now + Duration::minutes(15));
        let mut task_manager = TaskManager {
            stopped: vec![task1.clone()],
            running: Some(task2.clone()),
        };
        let result = task_manager.delete_task("Abc".to_string());
        assert_eq!(task_manager.running, Some(task2));
        assert_eq!(task_manager.stopped, vec![task1]);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, TaskError::TaskNotFound(_)));
    }

    #[test]
    fn test_task_manager_delete_task_when_stopped() {
        let now = Local::now();
        let mut task1 = StoppedTask {
            name: "Test10".to_string(),
            entries: vec![],
            last_entry: CompletedTimeEntry::new(now, now + Duration::minutes(5)),
        };
        let task2 = RunningTask::new("Test2", now + Duration::minutes(15));
        let mut task_manager = TaskManager {
            stopped: vec![task1.clone()],
            running: Some(task2.clone()),
        };
        let result = task_manager.delete_task("Test1".to_string());
        task1.name = "Test".to_string();
        assert_eq!(task_manager.running, Some(task2));
        assert_eq!(task_manager.stopped, vec![]);
        assert!(result.is_ok());
        let task_name = result.unwrap();
        assert_eq!("Test10", task_name);
    }

    #[test]
    fn test_task_manager_delete_task_when_running() {
        let now = Local::now();
        let task1 = StoppedTask {
            name: "Test1".to_string(),
            entries: vec![],
            last_entry: CompletedTimeEntry::new(now, now + Duration::minutes(5)),
        };
        let mut task2 = RunningTask::new("Test20", now + Duration::minutes(15));
        let mut task_manager = TaskManager {
            stopped: vec![task1.clone()],
            running: Some(task2.clone()),
        };
        let result = task_manager.delete_task("Test2".to_string());
        task2.name = "Test".to_string();
        assert_eq!(task_manager.running, None);
        assert_eq!(task_manager.stopped, vec![task1]);
        assert!(result.is_ok());
        let task_name = result.unwrap();
        assert_eq!("Test20", task_name);
    }

    #[test]
    fn test_task_manager_delete_task_when_ambiguous_name1() {
        let now = Local::now();
        let task1 = StoppedTask {
            name: "Test1".to_string(),
            entries: vec![],
            last_entry: CompletedTimeEntry::new(now, now + Duration::minutes(5)),
        };
        let task2 = StoppedTask {
            name: "Test2".to_string(),
            entries: vec![],
            last_entry: CompletedTimeEntry::new(
                now + Duration::minutes(5),
                now + Duration::minutes(10),
            ),
        };
        let mut task_manager = TaskManager {
            stopped: vec![task1.clone(), task2.clone()],
            running: None,
        };
        let result = task_manager.delete_task("Test".to_string());
        assert_eq!(task_manager.running, None);
        assert_eq!(task_manager.stopped, vec![task1, task2]);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, TaskError::MultipleTasksFound));
    }

    #[test]
    fn test_task_manager_delete_task_when_ambiguous_name2() {
        let now = Local::now();
        let task1 = StoppedTask {
            name: "Test1".to_string(),
            entries: vec![],
            last_entry: CompletedTimeEntry::new(now, now + Duration::minutes(5)),
        };
        let task2 = RunningTask::new("Test2", now + Duration::minutes(15));
        let mut task_manager = TaskManager {
            stopped: vec![task1.clone()],
            running: Some(task2.clone()),
        };
        let result = task_manager.delete_task("Test".to_string());
        assert_eq!(task_manager.running, Some(task2));
        assert_eq!(task_manager.stopped, vec![task1]);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, TaskError::MultipleTasksFound));
    }

    #[test]
    fn test_task_manager_generate_report_when_no_tasks() {
        let now = Local::now();
        let today = NaiveDate::from_ymd_opt(2024, 7, 16).unwrap();
        let task_manager = TaskManager {
            stopped: vec![],
            running: None,
        };
        let report = task_manager.generate_report(today, now);
        assert!(report.contains("2024-07-16"));
        assert!(report.contains("Total | 00:00 | 100.0%"));
        assert_eq!(3, report.lines().count());
    }

    #[test]
    fn test_task_manager_generate_report_when_no_running_task() {
        let now = Local::now();
        let today = NaiveDate::from_ymd_opt(2024, 7, 16).unwrap();
        let task1 = StoppedTask {
            name: "Test1".to_string(),
            entries: vec![],
            last_entry: CompletedTimeEntry::new(now, now + Duration::minutes(5)),
        };
        let task2 = StoppedTask {
            name: "Test2".to_string(),
            entries: vec![],
            last_entry: CompletedTimeEntry::new(
                now + Duration::minutes(5),
                now + Duration::minutes(15),
            ),
        };
        let task_manager = TaskManager {
            stopped: vec![task1, task2],
            running: None,
        };
        let report = task_manager.generate_report(today, now + Duration::minutes(20));
        assert!(report.contains("2024-07-16"));
        assert!(report.contains("  Test1 | 00:05 |  33.3%"));
        assert!(report.contains("  Test2 | 00:10 |  66.7%"));
        assert!(report.contains("  ======================"));
        assert!(report.contains("  Total | 00:15 | 100.0%"));
        assert_eq!(5, report.lines().count());
    }

    #[test]
    fn test_task_manager_generate_report_when_running_task() {
        let now = Local::now();
        let today = NaiveDate::from_ymd_opt(2024, 7, 16).unwrap();
        let task1 = StoppedTask {
            name: "Test1".to_string(),
            entries: vec![],
            last_entry: CompletedTimeEntry::new(now, now + Duration::minutes(5)),
        };
        let task2 = RunningTask::new("Test2", now + Duration::minutes(15));
        let task_manager = TaskManager {
            stopped: vec![task1],
            running: Some(task2),
        };
        let report = task_manager.generate_report(today, now + Duration::minutes(20));
        assert!(report.contains("2024-07-16"));
        assert!(report.contains("  Test1 | 00:05 |  50.0%"));
        assert!(report.contains("  Test2 | 00:05 |  50.0%"));
        assert!(report.contains("  ======================"));
        assert!(report.contains("  Total | 00:10 | 100.0%"));
        assert_eq!(5, report.lines().count());
    }

    #[test]
    fn test_task_manager_generate_report_when_long_task_name() {
        let now = Local::now();
        let today = NaiveDate::from_ymd_opt(2024, 7, 16).unwrap();
        let task1 = StoppedTask {
            name: "Test1".to_string(),
            entries: vec![],
            last_entry: CompletedTimeEntry::new(now, now + Duration::minutes(5)),
        };
        let task2 = StoppedTask {
            name: "Test2 is a very long name".to_string(),
            entries: vec![],
            last_entry: CompletedTimeEntry::new(
                now + Duration::minutes(5),
                now + Duration::minutes(15),
            ),
        };
        let task_manager = TaskManager {
            stopped: vec![task1, task2],
            running: None,
        };
        let report = task_manager.generate_report(today, now + Duration::minutes(20));
        assert!(report.contains("2024-07-16"));
        assert!(report.contains("  Test1                     | 00:05 |  33.3%"));
        assert!(report.contains("  Test2 is a very long name | 00:10 |  66.7%"));
        assert!(report.contains("  =========================================="));
        assert!(report.contains("  Total                     | 00:15 | 100.0%"));
        assert_eq!(5, report.lines().count());
    }
}

#[test]
fn test_format_duration() {
    assert_eq!(format_duration(Duration::minutes(10)), "00:10");
    assert_eq!(format_duration(Duration::minutes(140)), "02:20");
}

#[test]
fn test_percent() {
    assert_eq!(percent(0, 10), 0.0);
    assert_eq!(percent(1, 10), 10.0);
    assert_eq!(percent(10, 10), 100.0);
}
