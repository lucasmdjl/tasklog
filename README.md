# Tasklog

Tasklog is a simple CLI-based task management tool written in Rust. It allows you to start, stop, switch between tasks, and generate reports of your tasks. This tool is useful for tracking time spent on various tasks throughout the day.

## Features

- **Start a Task:** Begin a new task or resume an existing one.
- **Stop a Task:** Stop the current running task.
- **Switch Tasks:** Switch from the current task to a new one.
- **Generate Reports:** Generate a report of tasks worked on for a specific day.
- **Get the Current Task:** Print the task currently running, if any.

## Installation

To use Task Tracker, you need to have Rust installed on your system. If you don't have Rust installed, you can get it from [here](https://www.rust-lang.org/).

Clone the repository and build the project using Cargo:

```sh
cargo install tasklog
```
The executable will be located in the target/release directory.

## Usage

### Start a Task

To start a new task or resume the last task:
```sh
tasklog start <TASK_NAME>
```
If <TASK_NAME> is omitted, the last stopped task will be resumed.

### Stop a Task

To stop the current task:
```sh
tasklog stop
```

### Switch Tasks

To switch to a different task:
```sh
tasklog switch <TASK_NAME>
```
This will stop the current task and start the new task `<TASK_NAME>`.

### Generate a Report

To generate a report of tasks worked on today or a specific day:
```sh
tasklog report -n=<DAYS_AGO>
```
`<DAYS_AGO>` is the number of days between today and the day to report. For example, `-n=0` for today, `-n=1` for yesterday, and so on.

### Get Current Task

To get the task currently running:
```sh
tasklog current
```


## Examples

Start a new task named coding:
```sh
tasklog start coding
```

Stop the current task:
```sh
tasklog stop
```

Switch to a task named meeting:
```sh
tasklog switch meeting
```

Generate a report for today:
```sh
tasklog report -n=0
```

Generate a report for yesterday:
```sh
tasklog report -n=1
```

## Contributing
Contributions are welcome! Feel free to open an issue or submit a pull request on GitHub.

## License
This project is licensed under the GPL-v3.0 License. See the LICENSE file for details.
