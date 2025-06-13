use std::io::{self, IsTerminal, Write};

use anyhow::{Error, Result};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use crate::jogfile::Task;

macro_rules! write_with_color {
    ($dst:expr, $color:expr, $($arg:tt)*) => {
        $dst.set_color(&$color)
            .and_then(|_| write!($dst, $($arg)*))
            .and_then(|_| $dst.reset())
    };
}

fn color_choice(t: &impl IsTerminal) -> ColorChoice {
    if t.is_terminal() {
        ColorChoice::Auto
    } else {
        ColorChoice::Never
    }
}

pub fn help() -> Result<()> {
    let mut stderr = StandardStream::stderr(color_choice(&io::stderr()));
    write!(&mut stderr, "Run a task defined in a jogfile\n\n")?;

    write_with_color!(
        &mut stderr,
        ColorSpec::new().set_bold(true).set_underline(true),
        "Usage:"
    )?;
    write!(&mut stderr, " ")?;
    write_with_color!(&mut stderr, ColorSpec::new().set_bold(true), "jog")?;
    write!(&mut stderr, " [OPTIONS] [TASK] [ARGS]\n\n")?;

    write_with_color!(
        &mut stderr,
        ColorSpec::new().set_bold(true).set_underline(true),
        "Arguments:"
    )?;
    writeln!(&mut stderr)?;
    writeln!(&mut stderr, "  [TASK]  The name of the task to run")?;
    writeln!(&mut stderr, "  [ARGS]  Arguments to pass to the task")?;
    writeln!(&mut stderr)?;

    write_with_color!(
        &mut stderr,
        ColorSpec::new().set_bold(true).set_underline(true),
        "Options:"
    )?;
    writeln!(&mut stderr)?;
    write!(&mut stderr, "  ")?;
    write_with_color!(&mut stderr, ColorSpec::new().set_bold(true), "-l")?;
    write!(&mut stderr, ", ")?;
    write_with_color!(&mut stderr, ColorSpec::new().set_bold(true), "--list")?;
    writeln!(&mut stderr, "     List tasks and their parameters")?;
    write!(&mut stderr, "  ")?;
    write_with_color!(&mut stderr, ColorSpec::new().set_bold(true), "-h")?;
    write!(&mut stderr, ", ")?;
    write_with_color!(&mut stderr, ColorSpec::new().set_bold(true), "--help")?;
    writeln!(&mut stderr, "     Print help")?;
    write!(&mut stderr, "  ")?;
    write_with_color!(&mut stderr, ColorSpec::new().set_bold(true), "-V")?;
    write!(&mut stderr, ", ")?;
    write_with_color!(&mut stderr, ColorSpec::new().set_bold(true), "--version")?;
    writeln!(&mut stderr, "  Print version")?;
    writeln!(&mut stderr)?;

    writeln!(
        &mut stderr,
        "Tasks are run in $SHELL. Arguments are passed as environment variables. For tasks defined with a"
    )?;
    writeln!(
        &mut stderr,
        "final parameter of '...', extra arguments are passed as positional arguments."
    )?;

    Ok(())
}

pub fn tasks(tasks: &[Task]) -> Result<()> {
    let mut stdout = StandardStream::stdout(color_choice(&io::stdout()));
    for task in tasks {
        write_with_color!(
            &mut stdout,
            ColorSpec::new().set_bold(true),
            "{}",
            task.name
        )?;
        for param in &task.params {
            write!(&mut stdout, " {param}")?;
        }
        if task.rest {
            write!(&mut stdout, " ...")?;
        }
        writeln!(&mut stdout)?;
    }
    Ok(())
}

pub fn error(err: &Error) -> Result<()> {
    let mut stderr = StandardStream::stderr(color_choice(&io::stderr()));
    write_with_color!(
        &mut stderr,
        ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true),
        "error"
    )?;
    writeln!(&mut stderr, ": {err:#}")?;
    Ok(())
}

pub fn mismatched_args_msg(tasks: &[Task], name: &str, args: &[String]) -> String {
    fn param_count(task: &Task) -> String {
        let mut res = task.params.len().to_string();
        if task.rest {
            res.push('+');
        }
        res
    }

    let mut params_msg = param_count(&tasks[0]);
    if tasks.len() == 2 {
        params_msg.push_str(" or ");
        params_msg.push_str(&param_count(&tasks[1]));
    } else {
        for i in 1..tasks.len() {
            params_msg.push_str(", ");
            if i == tasks.len() - 1 {
                params_msg.push_str("or ");
            }
            params_msg.push_str(&param_count(&tasks[i]));
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

    format!("task '{name}' takes {params_msg}, but {args_msg}")
}
