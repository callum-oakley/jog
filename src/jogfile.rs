use std::{
    fs,
    iter::Peekable,
    process::{Command, ExitStatus},
};

use anyhow::{bail, Result};

pub struct Task {
    pub name: String,
    pub params: Vec<String>,
    pub star: bool,
    pub body: String,
}

impl Task {
    pub fn run(&self, path: &str, args: &[String]) -> Result<ExitStatus> {
        let mut cmd = Command::new(std::env::var("SHELL")?);
        for (param, arg) in self.params.iter().zip(args) {
            cmd.env(param, arg);
        }
        cmd.args(["-c", &self.body, path]);
        cmd.args(&args[self.params.len()..]);
        Ok(cmd.spawn()?.wait()?)
    }
}

pub fn read(path: &str) -> Result<Vec<Task>> {
    let mut tasks = Vec::new();
    let s = fs::read_to_string(path)?;
    let mut lines = s
        .lines()
        .enumerate()
        .map(|(i, line)| (i + 1, line))
        .peekable();
    while let Some(task) = parse_task(path, &mut lines)? {
        tasks.push(task);
    }
    Ok(tasks)
}

fn parse_task<'a>(
    path: &str,
    lines: &mut Peekable<impl Iterator<Item = (usize, &'a str)>>,
) -> Result<Option<Task>> {
    while lines
        .peek()
        .is_some_and(|(_, line)| line.starts_with('#') || line.chars().all(char::is_whitespace))
    {
        lines.next();
    }

    if let Some((line_no, header)) = lines.next() {
        if header.starts_with(char::is_whitespace) {
            bail!("{path}:{line_no}: malformed task: indented header");
        }

        let mut header = header.split_whitespace().map(String::from);

        let name = header.next().expect("header is nonempty by construction");

        let mut params: Vec<_> = header.collect();

        let mut star = false;
        if params.last().is_some_and(|p| p == "*") {
            params.pop();
            star = true;
        }

        let mut body = String::new();
        while lines
            .peek()
            .is_some_and(|(_, line)| line.is_empty() || line.starts_with(char::is_whitespace))
        {
            let (_, line) = lines.next().expect("lines is nonempty");
            body.push_str(line);
            body.push('\n');
        }

        Ok(Some(Task {
            name,
            params,
            star,
            body,
        }))
    } else {
        Ok(None)
    }
}
