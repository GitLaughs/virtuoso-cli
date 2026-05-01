use crate::config::Config;
use crate::error::{Result, VirtuosoError};
use crate::models::SessionInfo;
use crate::output::OutputFormat;
use crate::transport::tunnel::SSHClient;
use serde_json::{json, Value};

pub fn list(format: OutputFormat) -> Result<Value> {
    // In remote mode, sync session files from remote host first.
    // Best effort: failures are silent so local cache still works.
    if let Ok(cfg) = Config::from_env() {
        if cfg.is_remote() {
            if let Ok(client) = SSHClient::from_env(cfg.keep_remote_files) {
                let _ = SessionInfo::sync_from_remote(&client.runner);
            }
        }
    }

    let mut sessions = SessionInfo::list()
        .map_err(|e| VirtuosoError::Execution(format!("failed to read sessions: {e}")))?;

    let sessions_dir = SessionInfo::sessions_dir();
    sessions.retain(|s| {
        if s.is_alive() {
            true
        } else {
            let _ = std::fs::remove_file(sessions_dir.join(format!("{}.json", s.id)));
            false
        }
    });

    if format == OutputFormat::Json {
        return Ok(json!({
            "status": "success",
            "count": sessions.len(),
            "sessions": sessions.iter().map(|s| json!({
                "id": s.id,
                "port": s.port,
                "pid": s.pid,
                "host": s.host,
                "user": s.user,
                "created": s.created,
            })).collect::<Vec<_>>(),
        }));
    }

    if sessions.is_empty() {
        println!("No active Virtuoso sessions found.");
        println!("Start Virtuoso and run RBStart() in CIW to register a session.");
        return Ok(json!({"status": "success", "count": 0}));
    }

    println!(
        "{:<20} {:>6}  {:>7}  {:<12}  CREATED",
        "SESSION ID", "PORT", "PID", "HOST"
    );
    println!("{}", "-".repeat(72));
    for s in &sessions {
        println!(
            "{:<20} {:>6}  {:>7}  {:<12}  {}",
            s.id, s.port, s.pid, s.host, s.created
        );
    }

    Ok(json!({"status": "success", "count": sessions.len()}))
}

pub fn current() -> Result<Value> {
    let live: Vec<_> = SessionInfo::list()
        .unwrap_or_default()
        .into_iter()
        .filter(|s| s.is_alive())
        .collect();
    match live.len() {
        0 => Ok(json!({"status": "success", "session": null, "note": "no live sessions; VB_PORT will be used"})),
        1 => Ok(json!({
            "status": "success",
            "session": live[0].id,
            "port": live[0].port,
            "auto_selected": true,
        })),
        _ => {
            let ids: Vec<&str> = live.iter().map(|s| s.id.as_str()).collect();
            Ok(json!({
                "status": "ambiguous",
                "sessions": ids,
                "note": "use --session <id> to select one",
            }))
        }
    }
}

pub fn cleanup() -> Result<Value> {
    let all = SessionInfo::list().unwrap_or_default();
    let dir = SessionInfo::sessions_dir();
    let mut removed = Vec::new();
    for s in &all {
        if !s.is_alive() {
            let path = dir.join(format!("{}.json", s.id));
            if std::fs::remove_file(&path).is_ok() {
                removed.push(s.id.clone());
            }
        }
    }
    Ok(json!({
        "status": "success",
        "removed": removed.len(),
        "sessions": removed,
    }))
}

pub fn history(id: &str, only_skill: bool, only_cmd: bool, limit: usize) -> Result<Value> {
    let show_skill = !only_cmd;
    let show_cmd = !only_skill;

    let skill_entries: Vec<Value> = if show_skill {
        crate::history::load_skill(id)
            .into_iter()
            .rev()
            .take(if limit > 0 { limit } else { usize::MAX })
            .rev()
            .map(|e| {
                serde_json::json!({
                    "type": "skill",
                    "ts": e.ts,
                    "ok": e.ok,
                    "skill": e.skill,
                    "output": e.output,
                })
            })
            .collect()
    } else {
        vec![]
    };

    let cmd_entries: Vec<Value> = if show_cmd {
        crate::history::load_cmd(Some(id), if limit > 0 { limit } else { 0 })
            .into_iter()
            .map(|e| {
                serde_json::json!({
                    "type": "cmd",
                    "ts": e.ts,
                    "cmd": e.cmd,
                    "exit_code": e.exit_code,
                })
            })
            .collect()
    } else {
        vec![]
    };

    Ok(json!({
        "status": "success",
        "session": id,
        "skill_count": skill_entries.len(),
        "cmd_count": cmd_entries.len(),
        "skill": skill_entries,
        "cmd": cmd_entries,
    }))
}

pub fn show(id: &str, _format: OutputFormat) -> Result<Value> {
    let s = SessionInfo::load(id)
        .map_err(|e| VirtuosoError::NotFound(format!("session '{id}' not found: {e}")))?;

    Ok(json!({
        "status": "success",
        "session": {
            "id": s.id,
            "port": s.port,
            "pid": s.pid,
            "host": s.host,
            "user": s.user,
            "created": s.created,
            "alive": s.is_alive(),
        }
    }))
}
