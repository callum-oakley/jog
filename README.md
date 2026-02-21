# jog

jog is a task runner with no string substitution.

## Help

```
Run a task defined in a jogfile

Usage: jog [OPTIONS] [TASK] [ARGS]

Arguments:
  [TASK]  The name of the task to run
  [ARGS]  Arguments to pass to the task

Options:
  -l, --list     List tasks and their parameters
  -h, --help     Print help
  -V, --version  Print version

Tasks are run in $SHELL. Arguments are passed as shell variables. For tasks defined with a final
parameter of '...', extra arguments are passed as positional arguments.
```

## Example

In a new `jogfile`, define a task to greet the caller by name.

```sh
greet name
  echo "Hello ${name}"
```

Which we can run by calling:

```sh
> jog greet Callum
Hello Callum
```

We can also define what to do if we call `greet` with no arguments. Note that it's fine to call a
task recursively.

```sh
greet
  jog greet World
```

So now:

```sh
> jog greet
Hello World
```

Finally we can use `...` to define a task which works for any number of arguments. Extra arguments
are passed as positional arguments.

```sh
greet ...
  for name in "$@"; do
    jog greet "${name}"
  done
```

```sh
> jog greet Alice Bob
Hello Alice
Hello Bob
```

## No string substitution?

Other task runners ([make][], [just][]) pass arguments to tasks by substituting them in to the body
of the task itself. jog instead passes arguments as shell variables, so tasks are standard shell
scripts with no additional templating syntax.

Concretely, a task like

```sh
greet name
  echo "Hello ${name}"
```

expands to

```sh
name="$1"; shift
  echo "Hello ${name}"
```

## Install

With [brew][]:

```
brew install callum-oakley/tap/jog
```

With [cargo][]:

```
cargo install jog
```

Alternatively, there are binaries for Linux, MacOS, and Windows [attached to each release][].

[attached to each release]: https://github.com/callum-oakley/jsq/releases
[brew]: https://brew.sh/
[cargo]: https://www.rust-lang.org/tools/install
[just]: https://github.com/casey/just
[make]: https://en.wikipedia.org/wiki/Make_(software)
