use std::{
    env,
    process::{exit, Command, Stdio},
};

use anyhow::{Context, Result};

// Usage: your_docker.sh run <image> <command> <arg1> <arg2> ...
fn main() -> Result<()> {
    let args: Vec<_> = env::args().collect();
    let command = &args[3];
    let command_args = &args[4..];

    let mut child = Command::new(command)
        .args(command_args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .with_context(|| {
            format!(
                "Tried to run '{}' with arguments {:?}",
                command, command_args
            )
        })?;

    let status = child.wait()?;
    exit(status.code().unwrap_or(1));
}
