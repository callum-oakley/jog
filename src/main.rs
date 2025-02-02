#![warn(clippy::pedantic)]

mod jogfile;
mod print;

use anyhow::{bail, Context, Result};

fn try_main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    // TODO look for jogfile in parent directories
    // TODO nice error if jogfile doesn't exist
    let path = "jogfile";
    let tasks = jogfile::read(path);

    if args.is_empty() || args[0] == "--help" || args[0] == "-h" {
        println!("TODO help");
        return Ok(());
    }

    if args[0] == "--version" || args[0] == "-V" {
        println!("jog {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    if args[0] == "--list" || args[0] == "-l" {
        print::tasks(&tasks?)?;
        return Ok(());
    }

    if args[0].starts_with('-') {
        bail!("unknown option '{}'", args[0]);
    }

    let name = &args[0];
    let args = &args[1..];

    let tasks: Vec<_> = tasks?
        .into_iter()
        .filter(|task| &task.name == name)
        .collect();

    if tasks.is_empty() {
        bail!("{path}: unknown task '{name}'");
    }

    let Some(task) = tasks.iter().find(|&task| {
        task.params.len() == args.len() || task.star && task.params.len() < args.len()
    }) else {
        bail!("{path}: {}", print::mismatched_args_msg(&tasks, name, args));
    };

    std::process::exit(
        task.run(path, args)?
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
