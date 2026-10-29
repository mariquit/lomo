//! Install a guidance-only skill. Legacy cleanup removes only LOMO-owned entries.

use lomo::LomoError;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const SKILL: &str = include_str!("../assets/SKILL.md");
const STEERING: &str = include_str!("../assets/STEERING.md");
const START: &str = "<!-- lomo:start -->";
const END: &str = "<!-- lomo:end -->";

pub fn run(target: &str) -> Result<(), LomoError> {
    let config = config_dir(target)?;
    cleanup_legacy(target, &config)?;
    let skill = skill_path(target)?;
    // Older installs symlinked the skill directory into the source tree. A
    // dangling link makes create_dir_all fail with EEXIST; a live link would
    // write the new skill back into the repository.
    if let Some(dir) = skill.parent() {
        if dir
            .symlink_metadata()
            .is_ok_and(|meta| meta.file_type().is_symlink())
        {
            fs::remove_file(dir)?;
        }
        fs::create_dir_all(dir)?;
    }
    if skill
        .symlink_metadata()
        .is_ok_and(|meta| meta.file_type().is_symlink())
    {
        fs::remove_file(&skill)?;
    }
    fs::write(&skill, SKILL)?;
    let steering = install_steering(target)?;
    let binary = install_binary()?;
    println!("skill    {}", skill.display());
    println!("steering {}", steering.display());
    println!("binary   {}", binary.display());
    println!("Guidance only: the agent reads and edits TODO.md directly. No hooks installed.");
    Ok(())
}

fn home() -> Result<PathBuf, LomoError> {
    env::var_os("HOME")
        .or_else(|| env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .ok_or_else(|| LomoError::Message("could not locate a home directory".into()))
}

fn config_dir(target: &str) -> Result<PathBuf, LomoError> {
    let (override_key, default) = match target {
        "opencode" => ("LOMO_OPENCODE_CONFIG", "opencode"),
        "claude" => ("LOMO_CLAUDE_CONFIG", ".claude"),
        "codex" => ("LOMO_CODEX_HOME", ".codex"),
        "cursor" => ("LOMO_CURSOR_CONFIG", ".cursor"),
        _ => {
            return Err(LomoError::Message(
                "choose opencode, claude, codex, or cursor".into(),
            ))
        }
    };
    if let Some(path) = env::var_os(override_key) {
        return Ok(path.into());
    }
    if target == "codex" {
        if let Some(path) = env::var_os("CODEX_HOME") {
            return Ok(path.into());
        }
    }
    if target == "opencode" {
        let base = match env::var_os("XDG_CONFIG_HOME") {
            Some(path) => path.into(),
            None => home()?.join(".config"),
        };
        return Ok(base.join(default));
    }
    Ok(home()?.join(default))
}

fn skill_path(target: &str) -> Result<PathBuf, LomoError> {
    if target == "codex" {
        let base = match env::var_os("LOMO_AGENTS_SKILLS") {
            Some(path) => path.into(),
            None => home()?.join(".agents/skills"),
        };
        return Ok(base.join("lomo/SKILL.md"));
    }
    Ok(config_dir(target)?.join("skills/lomo/SKILL.md"))
}

fn install_binary() -> Result<PathBuf, LomoError> {
    let source = env::current_exe()?;
    let dirs: Vec<_> = env::var_os("PATH")
        .map(|path| env::split_paths(&path).collect())
        .unwrap_or_default();
    let directory = if let Some(path) = env::var_os("LOMO_BIN_DIR") {
        PathBuf::from(path)
    } else if let Some(path) = dirs.iter().find(|path| {
        path.to_string_lossy()
            .replace('\\', "/")
            .trim_end_matches('/')
            .ends_with("/.bun/bin")
    }) {
        path.clone()
    } else {
        home()?.join(".local/bin")
    };
    fs::create_dir_all(&directory)?;
    let destination = directory.join(if cfg!(windows) { "lomo.exe" } else { "lomo" });
    if source.canonicalize().ok() != destination.canonicalize().ok() {
        fs::copy(&source, &destination)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&destination, fs::Permissions::from_mode(0o755))?;
        }
    }
    if !dirs
        .iter()
        .any(|path| path.canonicalize().ok() == directory.canonicalize().ok())
    {
        println!("note    add {} to PATH", directory.display());
    }
    Ok(destination)
}

fn install_steering(target: &str) -> Result<PathBuf, LomoError> {
    let path = steering_path(target)?;
    if target == "cursor" && !path.exists() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(
            &path,
            "---\ndescription: TODO.md notation\nalwaysApply: true\n---\n\n",
        )?;
    }
    upsert_block(&path, STEERING.trim_end())?;
    Ok(path)
}

fn steering_path(target: &str) -> Result<PathBuf, LomoError> {
    match target {
        // OpenCode's own global instructions, not ~/.claude/CLAUDE.md and not
        // this repository's scope file.
        "opencode" => Ok(config_dir("opencode")?.join("AGENTS.md")),
        "claude" => {
            let dir = env::var_os("LOMO_CLAUDE_CONFIG")
                .map(PathBuf::from)
                .unwrap_or(home()?.join(".claude"));
            Ok(dir.join("CLAUDE.md"))
        }
        "codex" => Ok(config_dir("codex")?.join("AGENTS.md")),
        "cursor" => Ok(config_dir("cursor")?.join("rules/lomo.mdc")),
        _ => Err(LomoError::Message(
            "choose opencode, claude, codex, or cursor".into(),
        )),
    }
}

/// Insert or replace only the marked LOMO section. All other text stays.
fn upsert_block(path: &Path, body: &str) -> Result<(), LomoError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let existing = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(err) if err.kind() == io::ErrorKind::NotFound => String::new(),
        Err(err) => return Err(err.into()),
    };
    let block = format!("{START}\n{body}\n{END}");
    let updated = if let Some(start) = existing.find(START) {
        let Some(rel_end) = existing[start..].find(END) else {
            return Err(LomoError::Message(format!(
                "incomplete LOMO block in {}; not overwritten",
                path.display()
            )));
        };
        let end = start + rel_end + END.len();
        format!("{}{block}{}", &existing[..start], &existing[end..])
    } else if existing.trim().is_empty() {
        format!("{block}\n")
    } else {
        let mut out = existing;
        if !out.ends_with('\n') {
            out.push('\n');
        }
        if !out.ends_with("\n\n") {
            out.push('\n');
        }
        out.push_str(&block);
        out.push('\n');
        out
    };
    fs::write(path, updated)?;
    Ok(())
}

fn cleanup_legacy(target: &str, config: &Path) -> Result<(), LomoError> {
    // Do not strip instruction files here. Steering is installed separately and
    // must not be removed as legacy cleanup.
    if target == "cursor" {
        remove_instruction_block(&config.join("rules/lomo.mdc"), true)?;
    }
    if target == "opencode" {
        let plugin = config.join("plugins/lomo/index.ts");
        if let Ok(text) = fs::read_to_string(&plugin) {
            if text.contains("id: \"lomo\"") && text.contains("SPDX-License-Identifier: MIT") {
                fs::remove_file(&plugin)?;
                // Remove the directory only if empty; never remove user files.
                let _ = fs::remove_dir(plugin.parent().unwrap());
            }
        }
    } else {
        remove_hooks(&config.join(if target == "claude" {
            "settings.json"
        } else {
            "hooks.json"
        }))?;
    }
    Ok(())
}

fn remove_instruction_block(path: &Path, cursor: bool) -> io::Result<()> {
    let mut text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(err) => return Err(err),
    };
    let original = text.clone();
    while let Some(start) = text.find(START) {
        let Some(end) = text[start..].find(END) else {
            return Err(io::Error::other(format!(
                "incomplete LOMO block in {}; inspect it before setup",
                path.display()
            )));
        };
        text.replace_range(start..start + end + END.len(), "");
    }
    if text != original {
        let empty_rule = cursor
            && text
                .strip_prefix("---")
                .and_then(|rest| rest.find("---").map(|end| &rest[end + 3..]))
                .is_some_and(|body| body.trim().is_empty());
        if cursor && (text.trim().is_empty() || empty_rule) {
            fs::remove_file(path)?;
        } else {
            fs::write(path, text)?;
        }
    }
    Ok(())
}

fn owned_hook(hook: &serde_json::Value) -> bool {
    let Some(command) = hook.get("command").and_then(|value| value.as_str()) else {
        return false;
    };
    if let Some(args) = hook.get("args").and_then(|value| value.as_array()) {
        return lomo_binary(command)
            && args.len() == 2
            && args[0] == "hook"
            && matches!(args[1].as_str(), Some("claude" | "codex" | "cursor"));
    }
    [" hook claude", " hook codex", " hook cursor"]
        .iter()
        .any(|suffix| {
            let Some(prefix) = command.strip_suffix(suffix) else {
                return false;
            };
            let prefix = prefix.strip_prefix("& ").unwrap_or(prefix);
            let executable = prefix
                .strip_prefix('\'')
                .and_then(|path| path.strip_suffix('\''))
                .unwrap_or(prefix);
            lomo_binary(executable)
        })
}

fn lomo_binary(path: &str) -> bool {
    let name = path.rsplit(['/', '\\']).next().unwrap_or(path);
    matches!(name, "lomo" | "lomo.exe")
        || name.starts_with("lomo-x86_64-")
        || name.starts_with("lomo-aarch64-")
}

fn remove_hooks(path: &Path) -> Result<(), LomoError> {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(err) => return Err(err.into()),
    };
    let mut value: serde_json::Value = serde_json::from_str(&text).map_err(|err| {
        LomoError::Message(format!(
            "{} is not valid JSON: {err}; not overwritten",
            path.display()
        ))
    })?;
    let original = value.clone();
    if let Some(events) = value
        .get_mut("hooks")
        .and_then(|value| value.as_object_mut())
    {
        for groups in events.values_mut().filter_map(|value| value.as_array_mut()) {
            groups.retain_mut(|group| {
                if owned_hook(group) {
                    return false;
                }
                if let Some(hooks) = group
                    .get_mut("hooks")
                    .and_then(|value| value.as_array_mut())
                {
                    let before = hooks.len();
                    hooks.retain(|hook| !owned_hook(hook));
                    return before == hooks.len() || !hooks.is_empty();
                }
                true
            });
        }
    }
    if value != original {
        fs::write(path, serde_json::to_string_pretty(&value).unwrap() + "\n")?;
    }
    Ok(())
}
