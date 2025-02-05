use std::{
    ffi::OsStr,
    iter::Peekable,
    path::{Path, PathBuf},
    process::{Command, ExitStatus},
    str::FromStr,
};

use anyhow::{anyhow, bail, ensure, Context, Result};

pub struct Task {
    pub name: String,
    pub params: Vec<String>,
    pub star: bool,
    pub body: String,
    pub line_no: usize,
}

impl Task {
    pub fn run(&self, path: &Path, args: &[String]) -> Result<ExitStatus> {
        let depth = parse_env_or_default("JOG_DEPTH", 0)?;
        if depth > parse_env_or_default("JOG_MAX_DEPTH", 100)? {
            bail!(
                "{}:{}: maximum recursion depth exceeded running '{}' with {} {}",
                path.to_string_lossy(),
                self.line_no,
                self.name,
                args.len(),
                if args.len() == 1 {
                    "argument"
                } else {
                    "arguments"
                }
            );
        }
        let mut cmd = Command::new(std::env::var("SHELL")?);
        for (param, arg) in self.params.iter().zip(args) {
            cmd.env(param, arg);
        }
        cmd.env("JOG_DEPTH", (depth + 1).to_string());
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
    let s = std::fs::read_to_string(path)?;
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
            line_no,
        }))
    } else {
        Ok(None)
    }
}

fn parse_env_or_default<K, T>(key: K, default: T) -> Result<T>
where
    K: AsRef<OsStr>,
    T: FromStr,
    T::Err: std::fmt::Display,
{
    match std::env::var(key) {
        Ok(value) => value.parse().map_err(|err: T::Err| anyhow!("{err}")),
        Err(std::env::VarError::NotPresent) => Ok(default),
        Err(err) => Err(anyhow!(err)),
    }
}
