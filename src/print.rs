use std::{
    io::{self, IsTerminal, Write},
    path::Path,
};

use anyhow::{Error, Result};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use crate::jogfile::Jogfile;

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
    writeln!(&mut stderr, " [TASK]  List tasks and their parameters")?;
    write!(&mut stderr, "  ")?;
    write_with_color!(&mut stderr, ColorSpec::new().set_bold(true), "-h")?;
    write!(&mut stderr, ", ")?;
    write_with_color!(&mut stderr, ColorSpec::new().set_bold(true), "--help")?;
    writeln!(&mut stderr, "         Print help")?;
    write!(&mut stderr, "  ")?;
    write_with_color!(&mut stderr, ColorSpec::new().set_bold(true), "-V")?;
    write!(&mut stderr, ", ")?;
    write_with_color!(&mut stderr, ColorSpec::new().set_bold(true), "--version")?;
    writeln!(&mut stderr, "      Print version")?;
    writeln!(&mut stderr)?;

    writeln!(
        &mut stderr,
        "Tasks are run in $SHELL. Arguments are passed as shell variables. For tasks defined with a final"
    )?;
    writeln!(
        &mut stderr,
        "parameter of '...', extra arguments are passed as positional arguments."
    )?;

    Ok(())
}

pub fn list(current_dir: &Path, name: Option<&str>) -> Result<()> {
    let mut stdout = StandardStream::stdout(color_choice(&io::stdout()));
    for (i, jogfile) in Jogfile::read_iter(current_dir)?.enumerate() {
        let jogfile = jogfile?;
        if name.is_none() {
            if i > 0 {
                writeln!(&mut stdout)?;
            }
            write_with_color!(
                &mut stdout,
                ColorSpec::new().set_bold(true),
                "# {}\n",
                jogfile.path.display(),
            )?;
        }
        for task in jogfile.tasks {
            if name.is_none_or(|name| name == task.name) {
                write!(&mut stdout, "{}", task.name)?;
                for param in &task.params {
                    write!(&mut stdout, " {param}")?;
                }
                if task.rest {
                    write!(&mut stdout, " ...")?;
                }
                writeln!(&mut stdout)?;
            }
        }
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
