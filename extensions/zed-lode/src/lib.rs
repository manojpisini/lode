use zed_extension_api::{self as zed, SlashCommand, SlashCommandOutput, Worktree};

struct LodeExtension;

impl zed::Extension for LodeExtension {
    fn new() -> Self {
        LodeExtension
    }

    fn complete_slash_command_argument(
        &self,
        _command: SlashCommand,
        _args: Vec<String>,
    ) -> Result<Vec<zed::SlashCommandArgumentCompletion>, String> {
        Ok(Vec::new())
    }

    fn run_slash_command(
        &self,
        command: SlashCommand,
        args: Vec<String>,
        worktree: Option<&Worktree>,
    ) -> Result<SlashCommandOutput, String> {
        let root = worktree
            .map(|wt| wt.root_path())
            .unwrap_or_default();

        let mut cmd = zed::process::Command::new("lode");
        cmd = cmd.arg(&command.name);

        match command.name.as_str() {
            "check" => {
                if let Some(path) = args.first() {
                    if !path.is_empty() {
                        cmd = cmd.arg(path);
                    }
                }
                if !root.is_empty() {
                    cmd = cmd.env("LODE_PROJECT_ROOT", &root);
                }
            }
            "scan" => {
                cmd = cmd.arg("secrets");
                if !root.is_empty() {
                    cmd = cmd.arg("--path").arg(&root);
                }
                for arg in &args {
                    cmd = cmd.arg(arg.as_str());
                }
            }
            "status" => {
                cmd = cmd.arg("--json");
            }
            "init" => {
                if let Some(name) = args.first() {
                    cmd = cmd.arg(name);
                    if !root.is_empty() {
                        cmd = cmd.arg("--path").arg(&root);
                    }
                }
            }
            _ => {}
        }

        let output = cmd.output()?;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let text = if stderr.is_empty() { stdout } else { format!("{stdout}\n{stderr}") };

        Ok(SlashCommandOutput {
            text,
            sections: Vec::new(),
        })
    }
}

zed::register_extension!(LodeExtension);
