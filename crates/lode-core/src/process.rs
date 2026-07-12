use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    process::{Command, ExitStatus, Output},
};

use crate::{LodeError, Result};

pub struct Process {
    command: Command,
    error_path: PathBuf,
}

impl Process {
    pub fn new(program: &str) -> Result<Self> {
        validate_program(program)?;
        Ok(Self {
            command: Command::new(program),
            error_path: program.into(),
        })
    }

    pub fn current_executable() -> Result<Self> {
        let program = std::env::current_exe().map_err(|source| LodeError::Io {
            path: "current_exe".into(),
            source,
        })?;
        Ok(Self {
            command: Command::new(&program),
            error_path: program,
        })
    }

    pub fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.command.args(args);
        self
    }

    pub fn current_dir(&mut self, dir: impl AsRef<Path>) -> &mut Self {
        self.command.current_dir(dir);
        self
    }

    pub fn envs<I, K, V>(&mut self, envs: I) -> &mut Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        self.command.envs(envs);
        self
    }

    pub fn status(&mut self) -> Result<ExitStatus> {
        self.command.status().map_err(|source| LodeError::Io {
            path: self.error_path.clone(),
            source,
        })
    }

    pub fn output(&mut self) -> Result<Output> {
        self.command.output().map_err(|source| LodeError::Io {
            path: self.error_path.clone(),
            source,
        })
    }
}

fn validate_program(program: &str) -> Result<()> {
    if program.is_empty()
        || program.contains('/')
        || program.contains('\\')
        || program.contains(':')
        || program.contains('\0')
        || program.contains('|')
        || program.contains(';')
        || program.contains('&')
        || program.contains('$')
        || program.contains('`')
        || program.contains('(')
        || program.contains(')')
        || program.contains('<')
        || program.contains('>')
        || program.contains('!')
        || program.contains('\'')
        || program.contains('"')
        || program.contains('#')
        || program.contains('*')
        || program.contains('?')
        || program.contains('[')
        || program.contains(']')
        || program.contains('{')
        || program.contains('}')
        || program.contains('~')
    {
        return Err(LodeError::Message(format!(
            "unsafe process program: {program}"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::Process;

    #[test]
    fn rejects_path_like_programs() {
        assert!(Process::new("git").is_ok());
        for program in [
            "",
            "../sh",
            "tools/run",
            r"C:\Windows\System32\cmd.exe",
            "cmd.exe\0ignored",
        ] {
            assert!(Process::new(program).is_err(), "{program:?}");
        }
    }

    #[test]
    fn rejects_shell_metacharacters() {
        for program in [
            "cmd|echo",
            "cmd;ls",
            "cmd&wait",
            "cmd$PATH",
            "cmd`ls`",
            "cmd(sub)",
            "cmd)sub(",
            "cmd<file",
            "cmd>file",
            "cmd!hist",
            "cmd'arg'",
            r#"cmd"arg""#,
            "cmd#comment",
            "cmd*glob",
            "cmd?glob",
            "cmd[glob]",
            "cmd{glob}",
            "cmd~home",
        ] {
            assert!(
                Process::new(program).is_err(),
                "shell metacharacter should be rejected: {program:?}"
            );
        }
    }

    #[test]
    fn rejects_empty_program_name() {
        assert!(Process::new("").is_err());
    }

    #[test]
    fn rejects_path_separators_in_program_name() {
        for program in ["foo/bar", r"foo\bar", "C:cmd"] {
            assert!(
                Process::new(program).is_err(),
                "path separator should be rejected: {program:?}"
            );
        }
    }

    #[test]
    fn accepts_valid_program_names() {
        for program in ["git", "cargo", "rustc", "node", "npm", "python3", "bash"] {
            assert!(
                Process::new(program).is_ok(),
                "valid program should be accepted: {program:?}"
            );
        }
    }
}
