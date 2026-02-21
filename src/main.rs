#![warn(clippy::pedantic)]

mod jogfile;
mod print;

use anyhow::{Context, Result, bail};

use crate::jogfile::Jogfile;

fn try_main() -> Result<()> {
    let current_dir = std::env::current_dir()?;
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
        print::list(
            Jogfile::read_iter(&current_dir)?,
            args.get(1).map(String::as_str),
        )?;
        return Ok(());
    }

    if args[0].starts_with('-') {
        bail!("unknown option '{}'", args[0]);
    }

    let name = &args[0];
    let args = &args[1..];

    let mut mismatched_args = Vec::new();

    for jogfile in Jogfile::read_iter(&current_dir)? {
        let jogfile = jogfile?;

        if let Some(task) = jogfile.tasks.iter().find(|&task| {
            &task.name == name
                && (task.params.len() == args.len() || task.rest && task.params.len() < args.len())
        }) {
            std::process::exit(
                task.run(&jogfile.path, args)?
                    .code()
                    .context("terminated by a signal")?,
            )
        }

        mismatched_args.extend(jogfile.tasks.into_iter().filter(|task| &task.name == name));
    }

    if mismatched_args.is_empty() {
        bail!("unknown task '{name}'");
    }

    bail!(print::mismatched_args_msg(&mismatched_args, name, args));
}

fn main() {
    if let Err(err) = try_main() {
        print::error(&err).expect("printing error");
        std::process::exit(1);
    }
}
