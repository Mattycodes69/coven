use std::env;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HarnessSummary {
    pub id: &'static str,
    pub label: &'static str,
    pub executable: &'static str,
    pub available: bool,
    pub install_hint: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HarnessLaunchMode {
    Interactive,
    NonInteractive,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HarnessCommandSpec {
    pub id: &'static str,
    pub label: &'static str,
    pub executable: &'static str,
    pub interactive_prompt_prefix_args: &'static [&'static str],
    pub non_interactive_prompt_prefix_args: &'static [&'static str],
    pub install_hint: &'static str,
}

impl HarnessCommandSpec {
    pub fn prompt_args(&self, prompt: &str, mode: HarnessLaunchMode) -> Vec<String> {
        let prefix_args = match mode {
            HarnessLaunchMode::Interactive => self.interactive_prompt_prefix_args,
            HarnessLaunchMode::NonInteractive => self.non_interactive_prompt_prefix_args,
        };

        prefix_args
            .iter()
            .map(|arg| (*arg).to_string())
            .chain(std::iter::once(prompt.to_string()))
            .collect()
    }
}

pub fn built_in_harnesses() -> Vec<HarnessSummary> {
    built_in_harness_specs()
        .into_iter()
        .map(|spec| HarnessSummary {
            id: spec.id,
            label: spec.label,
            executable: spec.executable,
            available: executable_exists(spec.executable),
            install_hint: spec.install_hint,
        })
        .collect()
}

pub fn built_in_harness_specs() -> Vec<HarnessCommandSpec> {
    vec![
        HarnessCommandSpec {
            id: "codex",
            label: "Codex",
            executable: "codex",
            interactive_prompt_prefix_args: &[],
            non_interactive_prompt_prefix_args: &[
                "exec",
                "--skip-git-repo-check",
                "--color",
                "never",
            ],
            install_hint: "Install Codex with `npm install -g @openai/codex` or `brew install --cask codex`; if it is already installed, make sure `codex` is on PATH and run `codex login` or `codex` once to authenticate, then retry `coven doctor`.",
        },
        HarnessCommandSpec {
            id: "claude",
            label: "Claude Code",
            executable: "claude",
            interactive_prompt_prefix_args: &[],
            non_interactive_prompt_prefix_args: &["--print"],
            install_hint: "Install Claude Code with `npm install -g @anthropic-ai/claude-code`; if it is already installed, make sure `claude` is on PATH and run `claude doctor` to finish local auth/setup, then retry `coven doctor`.",
        },
    ]
}

pub fn command_parts_for_harness(
    harness_id: &str,
    prompt: &str,
    mode: HarnessLaunchMode,
) -> Result<(&'static str, Vec<String>)> {
    let spec = built_in_harness_specs()
        .into_iter()
        .find(|spec| spec.id == harness_id)
        .ok_or_else(|| anyhow!("unsupported harness `{harness_id}`"))?;

    Ok((spec.executable, spec.prompt_args(prompt, mode)))
}

fn executable_exists(executable: &str) -> bool {
    env::var_os("PATH")
        .map(|paths| executable_exists_in_paths(executable, env::split_paths(&paths)))
        .unwrap_or(false)
}

fn executable_exists_in_paths<I>(executable: &str, paths: I) -> bool
where
    I: IntoIterator<Item = PathBuf>,
{
    if executable.contains('/') || executable.contains('\\') {
        return false;
    }

    paths.into_iter().any(|path| {
        executable_candidates(&path, executable)
            .any(|candidate| candidate_is_executable(&candidate))
    })
}

#[cfg(unix)]
fn candidate_is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;

    path.metadata()
        .map(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(not(unix))]
fn candidate_is_executable(path: &Path) -> bool {
    path.is_file()
}

#[cfg(windows)]
fn executable_candidates<'a>(
    path: &'a Path,
    executable: &'a str,
) -> impl Iterator<Item = PathBuf> + 'a {
    let extensions = env::var_os("PATHEXT")
        .map(|value| {
            env::split_paths(&value)
                .map(|path| path.to_string_lossy().into_owned())
                .collect::<Vec<_>>()
        })
        .unwrap_or_else(|| vec![".COM".into(), ".EXE".into(), ".BAT".into(), ".CMD".into()]);

    let base = path.join(executable);
    let has_extension = Path::new(executable).extension().is_some();
    std::iter::once(base.clone()).chain(extensions.into_iter().filter_map(move |extension| {
        if has_extension {
            None
        } else {
            Some(path.join(format!("{executable}{extension}")))
        }
    }))
}

#[cfg(not(windows))]
fn executable_candidates<'a>(
    path: &'a Path,
    executable: &'a str,
) -> impl Iterator<Item = PathBuf> + 'a {
    std::iter::once(path.join(executable))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn executable_exists_in_paths_finds_matching_file() -> anyhow::Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let executable = temp_dir.path().join("codex");
        fs::write(&executable, "")?;
        make_executable(&executable)?;

        assert!(executable_exists_in_paths(
            "codex",
            vec![temp_dir.path().to_path_buf()]
        ));
        Ok(())
    }

    #[test]
    fn executable_exists_in_paths_returns_false_when_missing() -> anyhow::Result<()> {
        let temp_dir = tempfile::tempdir()?;

        assert!(!executable_exists_in_paths(
            "claude",
            vec![temp_dir.path().to_path_buf()]
        ));
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn executable_exists_in_paths_rejects_non_executable_file() -> anyhow::Result<()> {
        let temp_dir = tempfile::tempdir()?;
        fs::write(temp_dir.path().join("codex"), "")?;

        assert!(!executable_exists_in_paths(
            "codex",
            vec![temp_dir.path().to_path_buf()]
        ));
        Ok(())
    }

    #[test]
    fn executable_exists_in_paths_rejects_paths() -> anyhow::Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let executable = temp_dir.path().join("codex");
        fs::write(&executable, "")?;
        make_executable(&executable)?;

        assert!(!executable_exists_in_paths(
            temp_dir.path().join("codex").to_string_lossy().as_ref(),
            vec![temp_dir.path().to_path_buf()]
        ));
        Ok(())
    }

    #[test]
    fn built_in_harnesses_returns_codex_and_claude() {
        let harnesses = built_in_harnesses();

        assert_eq!(harnesses.len(), 2);
        assert_eq!(harnesses[0].id, "codex");
        assert_eq!(harnesses[0].label, "Codex");
        assert_eq!(harnesses[0].executable, "codex");
        assert_eq!(harnesses[1].id, "claude");
        assert_eq!(harnesses[1].label, "Claude Code");
        assert_eq!(harnesses[1].executable, "claude");
    }

    #[test]
    fn built_in_harnesses_include_first_run_recovery_commands() {
        let harnesses = built_in_harnesses();
        let codex = harnesses
            .iter()
            .find(|harness| harness.id == "codex")
            .expect("codex harness should exist");
        let claude = harnesses
            .iter()
            .find(|harness| harness.id == "claude")
            .expect("claude harness should exist");

        assert!(codex.install_hint.contains("npm install -g @openai/codex"));
        assert!(codex.install_hint.contains("brew install --cask codex"));
        assert!(codex.install_hint.contains("codex"));
        assert!(codex.install_hint.contains("PATH"));

        assert!(claude
            .install_hint
            .contains("npm install -g @anthropic-ai/claude-code"));
        assert!(claude.install_hint.contains("claude doctor"));
        assert!(claude.install_hint.contains("claude"));
        assert!(claude.install_hint.contains("PATH"));
    }

    #[test]
    fn command_parts_for_known_harnesses_append_interactive_prompt() -> anyhow::Result<()> {
        assert_eq!(
            command_parts_for_harness("codex", "fix tests", HarnessLaunchMode::Interactive)?,
            ("codex", vec!["fix tests".to_string()])
        );
        assert_eq!(
            command_parts_for_harness("claude", "polish ui", HarnessLaunchMode::Interactive)?,
            ("claude", vec!["polish ui".to_string()])
        );
        Ok(())
    }

    #[test]
    fn command_parts_for_known_harnesses_use_noninteractive_entrypoints() -> anyhow::Result<()> {
        assert_eq!(
            command_parts_for_harness("codex", "fix tests", HarnessLaunchMode::NonInteractive)?,
            (
                "codex",
                vec![
                    "exec".to_string(),
                    "--skip-git-repo-check".to_string(),
                    "--color".to_string(),
                    "never".to_string(),
                    "fix tests".to_string(),
                ]
            )
        );
        assert_eq!(
            command_parts_for_harness("claude", "polish ui", HarnessLaunchMode::NonInteractive)?,
            (
                "claude",
                vec!["--print".to_string(), "polish ui".to_string()]
            )
        );
        Ok(())
    }

    #[test]
    fn command_spec_supports_prefix_args_for_future_harnesses() {
        let spec = HarnessCommandSpec {
            id: "future",
            label: "Future Harness",
            executable: "future",
            interactive_prompt_prefix_args: &["chat"],
            non_interactive_prompt_prefix_args: &["exec", "-q"],
            install_hint: "Install the future harness.",
        };

        assert_eq!(
            spec.prompt_args("hello", HarnessLaunchMode::Interactive),
            vec!["chat".to_string(), "hello".to_string()]
        );
        assert_eq!(
            spec.prompt_args("hello", HarnessLaunchMode::NonInteractive),
            vec!["exec".to_string(), "-q".to_string(), "hello".to_string()]
        );
    }

    #[test]
    fn command_parts_reject_unknown_harnesses() {
        assert!(
            command_parts_for_harness("hermes", "hello", HarnessLaunchMode::Interactive)
                .unwrap_err()
                .to_string()
                .contains("unsupported harness")
        );
    }

    #[cfg(unix)]
    fn make_executable(path: &Path) -> anyhow::Result<()> {
        let mut permissions = fs::metadata(path)?.permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions)?;
        Ok(())
    }

    #[cfg(not(unix))]
    fn make_executable(_path: &Path) -> anyhow::Result<()> {
        Ok(())
    }
}
