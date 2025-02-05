use std::{
    fs,
    iter::Peekable,
    path::{Path, PathBuf},
    process::{Command, ExitStatus},
};

use anyhow::{bail, ensure, Context, Result};

pub struct Task {
    pub name: String,
    pub params: Vec<String>,
    pub star: bool,
    pub body: String,
}

impl Task {
    pub fn run(&self, path: &Path, args: &[String]) -> Result<ExitStatus> {
        let mut cmd = Command::new(std::env::var("SHELL")?);
        for (param, arg) in self.params.iter().zip(args) {
            cmd.env(param, arg);
        }
        cmd.args(["-c", &self.body, path.to_str().context("non-UTF-8 path")?]);
        cmd.args(&args[self.params.len()..]);
        Ok(cmd.spawn()?.wait()?)
    }
}

pub fn find() -> Result<PathBuf> {
    if Path::new("jogfile").try_exists()? {
        Ok(PathBuf::from("jogfile"))
    } else {
        let mut dir = PathBuf::from("..");
        while !dir.join("jogfile").try_exists()? {
            ensure!(dir.canonicalize()?.parent().is_some(), "jogfile not found");
            dir = Path::new("..").join(dir);
        }
        Ok(dir.join("jogfile"))
    }
}

pub fn read(path: &Path) -> Result<Vec<Task>> {
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
    path: &Path,
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
            bail!(
                "{}:{}: malformed task: indented header",
                path.to_string_lossy(),
                line_no
            );
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
