use crate::store::Config;
use std::fmt::Write as _;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::thread;
use tauri::{Emitter, Window};

/// Build the argv for "run this command line through the user's login shell".
/// -l makes it a login shell (sources profile -> PATH, ssh-agent), -i interactive,
/// -c runs the command string. This is what makes ssh/rsync behave like in Terminal.
fn shell_invocation(cfg: &Config, command: &str) -> (String, Vec<String>) {
    (
        cfg.shell.clone(),
        vec!["-lic".to_string(), command.to_string()],
    )
}

/// Run a command and stream stdout+stderr to the frontend line by line via events.
/// Emits "run:line" for each line and "run:done" with the exit code.
#[tauri::command]
pub fn run_stream(window: Window, cfg: Config, command: String) -> Result<(), String> {
    let (program, args) = shell_invocation(&cfg, &command);

    let mut child = Command::new(&program)
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("spawn failed: {e}"))?;

    let stdout = child.stdout.take().ok_or("no stdout")?;
    let stderr = child.stderr.take().ok_or("no stderr")?;

    let w1 = window.clone();
    let t_out = thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines().map_while(Result::ok) {
            let _ = w1.emit(
                "run:line",
                serde_json::json!({ "stream": "stdout", "line": line }),
            );
        }
    });

    let w2 = window.clone();
    let t_err = thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines().map_while(Result::ok) {
            let _ = w2.emit(
                "run:line",
                serde_json::json!({ "stream": "stderr", "line": line }),
            );
        }
    });

    thread::spawn(move || {
        let _ = t_out.join();
        let _ = t_err.join();
        let code = child.wait().ok().and_then(|s| s.code()).unwrap_or(-1);
        let _ = window.emit("run:done", serde_json::json!({ "code": code }));
    });

    Ok(())
}

/// Run a command, block until done, return combined output. Used for dry runs
/// where you want the full preview before deciding to execute for real.
#[tauri::command]
pub fn run_capture(cfg: Config, command: String) -> Result<String, String> {
    let (program, args) = shell_invocation(&cfg, &command);
    let out = Command::new(&program)
        .args(&args)
        .output()
        .map_err(|e| format!("spawn failed: {e}"))?;

    let mut s = String::new();
    s.push_str(&String::from_utf8_lossy(&out.stdout));
    let err = String::from_utf8_lossy(&out.stderr);
    if !err.trim().is_empty() {
        if !s.is_empty() {
            s.push('\n');
        }
        s.push_str(&err);
    }
    let code = out.status.code().unwrap_or(-1);
    s.push_str(&format!("\n[exit {code}]"));
    Ok(s)
}

/// Hand the command off to a real terminal so interactive things (ssh tunnels,
/// password prompts, long-lived processes) work properly.
#[tauri::command]
pub fn run_in_terminal(cfg: Config, command: String) -> Result<(), String> {
    run_in_terminal_impl(cfg, command)
}

#[cfg(target_os = "macos")]
fn run_in_terminal_impl(cfg: Config, command: String) -> Result<(), String> {
    if cfg.terminal == "warp" {
        let command = percent_encode_url_component(&command);
        let url = format!("warp://action/new_tab?command={command}");
        Command::new("open")
            .arg(url)
            .spawn()
            .map_err(|e| format!("warp handoff failed: {e}"))?;
        return Ok(());
    }

    // Escape double quotes and backslashes for embedding in AppleScript.
    let esc = command.replace('\\', "\\\\").replace('"', "\\\"");

    let script = if cfg.terminal == "iterm" {
        format!(
            r#"tell application "iTerm"
  activate
  set newWindow to (create window with default profile)
  tell current session of newWindow
    write text "{esc}"
  end tell
end tell"#
        )
    } else {
        format!(
            r#"tell application "Terminal"
  activate
  do script "{esc}"
end tell"#
        )
    };

    Command::new("osascript")
        .arg("-e")
        .arg(script)
        .spawn()
        .map_err(|e| format!("osascript failed: {e}"))?;
    Ok(())
}

#[cfg(target_os = "linux")]
fn run_in_terminal_impl(cfg: Config, command: String) -> Result<(), String> {
    run_in_terminal_linux(cfg, command)
}

#[cfg(target_os = "linux")]
fn run_in_terminal_linux(cfg: Config, command: String) -> Result<(), String> {
    let mut candidates: Vec<(&str, &[&str])> = vec![];
    
    // If user selected a specific terminal, prioritize it
    match cfg.terminal.as_str() {
        "gnome-terminal" => candidates.push(("gnome-terminal", &["--", "bash", "-lc"])),
        "konsole" => candidates.push(("konsole", &["-e", "bash", "-lc"])),
        "xfce4-terminal" => candidates.push(("xfce4-terminal", &["--command"])),
        "xterm" => candidates.push(("xterm", &["-e"])),
        "custom" | "default" | _ => {} // Fall through to standard list
    }

    candidates.extend_from_slice(&[
        ("xdg-terminal-exec", &[]),
        ("gnome-terminal", &["--", "bash", "-lc"]),
        ("konsole", &["-e", "bash", "-lc"]),
        ("xfce4-terminal", &["--command"]),
        ("xterm", &["-e"]),
    ]);

    for (program, prefix_args) in candidates {
        if command_exists(program) {
            let mut cmd = std::process::Command::new(program);
            cmd.args(prefix_args);
            cmd.arg(&command);
            cmd.spawn().map_err(|e| format!("{program} handoff failed: {e}"))?;
            return Ok(());
        }
    }

    Err("no supported Linux terminal found".into())
}

#[cfg(target_os = "linux")]
fn command_exists(program: &str) -> bool {
    std::process::Command::new("sh")
        .arg("-lc")
        .arg(format!("command -v {program} >/dev/null 2>&1"))
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

#[cfg(target_os = "windows")]
fn run_in_terminal_impl(_cfg: Config, _command: String) -> Result<(), String> {
    Err("terminal handoff is disabled on Windows by default".into())
}

fn percent_encode_url_component(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                encoded.push(byte as char);
            }
            _ => {
                let _ = write!(&mut encoded, "%{byte:02X}");
            }
        }
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::percent_encode_url_component;

    #[test]
    fn percent_encodes_warp_command_url_component() {
        assert_eq!(
            percent_encode_url_component("echo hi && ls ~/data/$USER"),
            "echo%20hi%20%26%26%20ls%20~%2Fdata%2F%24USER"
        );
    }
}
