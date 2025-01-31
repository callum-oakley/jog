#![warn(clippy::pedantic)]

mod jogfile;
mod print;
mod shell;

use anyhow::{bail, Result};

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
        bail!("{path}: no task named '{name}'");
    }

    let Some(task) = tasks.iter().find(|&task| {
        task.params.len() == args.len() || task.star && task.params.len() < args.len()
    }) else {
        let mut params_msg = tasks[0].param_count();
        if tasks.len() == 2 {
            params_msg.push_str(" or ");
            params_msg.push_str(&tasks[1].param_count());
        } else {
            for i in 1..tasks.len() {
                params_msg.push_str(", ");
                if i == tasks.len() - 1 {
                    params_msg.push_str("or ");
                }
                params_msg.push_str(&tasks[i].param_count());
            }
        }
        params_msg.push_str(" parameter");
        if tasks.len() > 1 || tasks[0].params.len() != 1 {
            params_msg.push('s');
        }

        let mut args_msg = args.len().to_string();
        if args.len() == 1 {
            args_msg.push_str(" was given");
        } else {
            args_msg.push_str(" were given");
        }

        bail!("{path}: task '{name}' takes {params_msg}, but {args_msg}");
    };

    std::process::exit(shell::run(task, args)?)
}

fn main() {
    if let Err(err) = try_main() {
        print::error(&err).expect("printing error");
        std::process::exit(1);
    }
}
