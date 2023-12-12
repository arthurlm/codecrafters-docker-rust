use std::{
    env, fs,
    os::unix::{self, process::CommandExt},
    path::{Path, PathBuf},
    process::{exit, Command, ExitStatus, Stdio},
};

use anyhow::Context;
use tempfile::{tempdir, TempDir};

// Usage: your_docker.sh run <image> <command> <arg1> <arg2> ...
fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let container = Container::new(cli)?;
    let status = container.exec()?;
    exit(status.code().unwrap_or(1));
}

#[derive(Debug)]
struct Cli {
    command: String,
    args: Vec<String>,
}

impl Cli {
    fn parse() -> Self {
        let args: Vec<_> = env::args().collect();
        Self {
            command: args[3].clone(),
            args: args[4..].to_vec(),
        }
    }
}

#[derive(Debug)]
struct Container {
    command: PathBuf,
    args: Vec<String>,
    chroot_dir: TempDir,
}

impl Container {
    fn new(cli: Cli) -> anyhow::Result<Self> {
        // Prepare chroot.
        let chroot_dir =
            tempdir().with_context(|| format!("Cannot create temporary chroot dir"))?;

        let chroot_path = chroot_dir.path();

        // Create /dev/null in chroot
        fs::create_dir_all(chroot_path.join("dev"))?;
        fs::write(chroot_path.join("dev/null"), "")?;

        // Copy program to chroot
        let program_basename = Path::new(&cli.command)
            .file_name()
            .with_context(|| format!("Missing program basename"))?;

        fs::copy(&cli.command, chroot_path.join(program_basename))
            .with_context(|| "Cannot copy bin to chroot dir")?;

        Ok(Self {
            command: PathBuf::from("/").join(program_basename),
            args: cli.args,
            chroot_dir,
        })
    }

    fn exec(&self) -> anyhow::Result<ExitStatus> {
        // Isolate PID namespace
        // NOTE: Need to be called on parent process.
        assert_eq!(
            unsafe { libc::unshare(libc::CLONE_NEWPID) },
            0,
            "unshare fail"
        );

        // Pipe file descriptor and clean env.
        let mut ps = Command::new(&self.command);
        ps.args(&self.args)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .env_clear();

        let chroot_path = self.chroot_dir.path().to_path_buf();
        assert!(chroot_path.exists());

        unsafe {
            ps.pre_exec(move || {
                // Isolate process before spawning it.
                unix::fs::chroot(&chroot_path)?;
                env::set_current_dir("/")?;
                Ok(())
            });
        }

        // Spawn process.
        let mut child = ps.spawn()?;

        // Wait for its completion.
        let status = child.wait()?;

        Ok(status)
    }
}
