use std::{os::unix::process::ExitStatusExt, process::Command};

use anyhow::{Context, Result};

use crate::jogfile::Task;

pub fn run(task: &Task, args: &[String]) -> Result<i32> {
    let mut cmd = Command::new(std::env::var("SHELL")?);
    for (param, arg) in task.params.iter().zip(args) {
        cmd.env(param, arg);
    }
    cmd.args(["-c", &task.body, &task.name]);
    cmd.args(&args[task.params.len()..]);
    let status = cmd.spawn()?.wait()?;
    status
        .code()
        .or(status.signal())
        .context("unable to determine exit code")
}
