# Tasklog

Tasklog is a simple CLI-based task management tool written in Rust. It allows you to start, stop, switch between tasks, generate reports of your tasks, and more.
This tool is useful for tracking time spent on various tasks throughout the day.

## Features

- **Start a Task:** Begin a new task.
- **Stop a Task:** Stop the current running task.
- **Resume a Task:** Resume a stopped task.
- **Switch Tasks:** Switch from the current task to a new or different one.
- **Generate Reports:** Generate a report of tasks worked on for a specific day.
- **Get the Current Task:** Print the task currently running, if any.
- **List Tasks:** Print a list of the tasks worked on a day.
- **Rename a Task**: Change the name of a task.
- **Delete a Task**: Remove a task.

## Installation

To use Task Tracker, you need to have Rust installed on your system. If you don't have Rust installed, you can get it from [here](https://www.rust-lang.org/).
Then run:

```sh
cargo install tasklog
```

## Examples

Start a new task named coding:
```sh
tasklog start coding
```

Switch to a new task named meeting:
```sh
tasklog switch -c meeting
```

Stop the current task:
```sh
tasklog stop
```

Generate a report for today:
```sh
tasklog report
```

Generate a report for yesterday:
```sh
tasklog report -n=1
```

## Contributing
Contributions are welcome! Feel free to open an issue or submit a pull request on GitHub.

## License
This project is licensed under the GPL-v3.0 License. See the LICENSE file for details.
