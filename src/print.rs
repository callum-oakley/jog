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

pub fn mismatched_args_msg(tasks: &[Task], name: &str, args: &[String]) -> String {
    fn param_count(task: &Task) -> String {
        let mut res = task.params.len().to_string();
        if task.star {
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
