use std::{
    ffi::OsStr,
    fmt::Write,
    iter::Peekable,
    path::{Path, PathBuf},
    process::{Command, ExitStatus},
    str::FromStr,
};

use anyhow::{Context, Result, anyhow, bail, ensure};

pub struct Jogfile {
    pub path: PathBuf,
    pub tasks: Vec<Task>,
}

impl Jogfile {
    pub fn read_iter(current_dir: &Path) -> Result<impl Iterator<Item = Result<Self>>> {
        let mut jogfiles = current_dir
            .ancestors()
            .filter_map(|dir| {
                let path = dir.join("jogfile");
                match path.try_exists() {
                    Err(err) => Some(Err(err.into())),
                    Ok(false) => None,
                    Ok(true) => Some(Jogfile::read(path)),
                }
            })
            .peekable();
        if jogfiles.peek().is_none() {
            bail!("jogfile not found");
        }
        Ok(jogfiles)
    }

    fn read(path: PathBuf) -> Result<Self> {
        let mut tasks = Vec::new();
        let s = std::fs::read_to_string(&path)?;
        let mut lines = s
            .lines()
            .enumerate()
            .map(|(i, line)| (i + 1, line))
            .peekable();
        while let Some(task) = parse_task(&path, &mut lines)? {
            tasks.push(task);
        }
        validate(&path, &tasks)?;
        Ok(Self { path, tasks })
    }
}

pub struct Task {
    pub name: String,
    pub params: Vec<String>,
    pub rest: bool,
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
        cmd.env("JOG_DEPTH", (depth + 1).to_string());
        cmd.args(["-c", &self.body, path.to_str().context("non-UTF-8 path")?]);
        cmd.args(args);
        Ok(cmd.spawn()?.wait()?)
    }
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

        let mut rest = false;
        if params.last().is_some_and(|p| p == "...") {
            params.pop();
            rest = true;
        }

        let mut body = String::new();
        for param in &params {
            writeln!(&mut body, "{param}=\"$1\"; shift")?;
        }
        while lines
            .peek()
            .is_some_and(|(_, line)| line.is_empty() || line.starts_with(char::is_whitespace))
        {
            let (_, line) = lines.next().expect("lines is nonempty");
            writeln!(&mut body, "{line}")?;
        }

        Ok(Some(Task {
            name,
            params,
            rest,
            body,
            line_no,
        }))
    } else {
        Ok(None)
    }
}

fn validate(path: &Path, tasks: &[Task]) -> Result<()> {
    for j in 1..tasks.len() {
        for i in 0..j {
            let a = &tasks[i];
            let b = &tasks[j];
            if a.name == b.name {
                // Check if 'a' would always run instead of 'b'
                let redundant = match (a.rest, b.rest) {
                    (true, _) => a.params.len() <= b.params.len(),
                    (false, true) => false,
                    (false, false) => a.params.len() == b.params.len(),
                };
                ensure!(
                    !redundant,
                    "{}:{}: redundant definition for '{}', already covered by {}:{}",
                    path.to_string_lossy(),
                    b.line_no,
                    a.name,
                    path.to_string_lossy(),
                    a.line_no,
                );
            }
        }
    }
    Ok(())
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
