#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

extern crate os_pipe;
#[macro_use]
extern crate clap;

use clap::App;
use std::process::{Command, ExitStatus};
use std::io::Result;
use os_pipe::{pipe, IntoStdio};

fn make_cmd(prg: &str, use_shell: bool) -> Command {
    if !use_shell {
        Command::new(prg)
    } else {
        #[cfg(not(windows))]
        let (shell, flag) = ("sh", "-c");
        #[cfg(windows)]
        let (shell, flag) = ("cmd", "/c");

        let mut cmd = Command::new(shell);
        cmd.arg(flag).arg(prg);
        cmd
    }

}

fn run_cmd(prg1: &str, prg2: &str, use_shell: bool) -> Result<(ExitStatus, ExitStatus)> {
    let mut cmd1 = make_cmd(prg1, use_shell);
    let mut cmd2 = make_cmd(prg2, use_shell);

    let (to2, from1) = pipe()?;
    let (to1, from2) = pipe()?;

    cmd1.stdin(to1.into_stdio());
    cmd2.stdin(to2.into_stdio());
    cmd1.stdout(from1.into_stdio());
    cmd2.stdout(from2.into_stdio());

    let mut handle1 = cmd1.spawn()?;
    let mut handle2 = cmd2.spawn()?;

    drop(cmd1);
    drop(cmd2);
    Ok((handle1.wait()?, handle2.wait()?))
}

fn main() {
    let args = load_yaml!("args_en.yaml");
    let matches = App::from_yaml(args).version(crate_version!()).get_matches();
    let prg1 = matches.value_of("prg1").unwrap();
    let prg2 = matches.value_of("prg2").unwrap();

    let (r1, r2) = run_cmd(prg1, prg2, matches.is_present("use_shell"))
        .expect("There's something wrong.");

    if matches.is_present("show_status") {
        println!("{}: {}", prg1, r1);
        println!("{}: {}", prg2, r2);
    }
}
