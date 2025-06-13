#![warn(clippy::pedantic)]

mod jogfile;
mod print;

use anyhow::{Context, Result, bail};

fn try_main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() || args[0] == "--help" || args[0] == "-h" {
        print::help()?;
        return Ok(());
    }

    if args[0] == "--version" || args[0] == "-V" {
        println!("jog {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    if args[0] == "--list" || args[0] == "-l" {
        print::tasks(&jogfile::read(&jogfile::find()?)?)?;
        return Ok(());
    }

    if args[0].starts_with('-') {
        bail!("unknown option '{}'", args[0]);
    }

    let path = jogfile::find()?;
    let name = &args[0];
    let args = &args[1..];

    let tasks: Vec<_> = jogfile::read(&path)?
        .into_iter()
        .filter(|task| &task.name == name)
        .collect();

    if tasks.is_empty() {
        bail!("{}: unknown task '{}'", path.to_string_lossy(), name);
    }

    let Some(task) = tasks.iter().find(|&task| {
        task.params.len() == args.len() || task.rest && task.params.len() < args.len()
    }) else {
        bail!(
            "{}: {}",
            path.to_string_lossy(),
            print::mismatched_args_msg(&tasks, name, args)
        );
    };

    std::process::exit(
        task.run(&path, args)?
            .code()
            .context("terminated by a signal")?,
    )
}

fn main() {
    if let Err(err) = try_main() {
        print::error(&err).expect("printing error");
        std::process::exit(1);
    }
}
