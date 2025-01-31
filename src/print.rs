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
        if task.star {
            write!(&mut stdout, " *")?;
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
