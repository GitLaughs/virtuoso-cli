use crate::client::bridge::VirtuosoClient;
use crate::error::{Result, VirtuosoError};
use crate::models::SessionInfo;
use serde_json::{json, Value};

pub fn exec(code: &str, timeout: u64, readonly: bool) -> Result<Value> {
    let mut client = VirtuosoClient::from_env()?;
    if readonly {
        client = client.with_sandbox_mode();
    }
    let result = client.execute_skill(code, Some(timeout))?;

    Ok(json!({
        "status": if result.ok() { "success" } else { "error" },
        "output": result.output,
        "errors": result.errors,
        "warnings": result.warnings,
        "execution_time": result.execution_time,
    }))
}

/// Run `code` concurrently against every live local session.
/// Each session gets its own TCP connection in a scoped thread.
/// Returns per-session results; exit is non-zero only when every session fails.
pub fn broadcast(code: &str, timeout: u64) -> Result<Value> {
    let sessions = SessionInfo::list_alive();
    if sessions.is_empty() {
        return Err(VirtuosoError::NotFound("no live sessions found".into()));
    }

    let mut results: Vec<Value> = std::thread::scope(|s| {
        let handles: Vec<_> = sessions
            .iter()
            .map(|session| {
                let id = session.id.clone();
                let port = session.port;
                s.spawn(move || {
                    let client = VirtuosoClient::new("127.0.0.1", port, timeout);
                    match client.execute_skill(code, Some(timeout)) {
                        Ok(r) => json!({
                            "session": id,
                            "ok": r.skill_ok(),
                            "output": r.output,
                        }),
                        Err(e) => json!({
                            "session": id,
                            "ok": false,
                            "error": e.to_string(),
                        }),
                    }
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|h| {
                h.join()
                    .unwrap_or_else(|_| json!({"ok": false, "error": "thread panicked"}))
            })
            .collect()
    });

    results.sort_by(|a, b| {
        a["session"]
            .as_str()
            .unwrap_or("")
            .cmp(b["session"].as_str().unwrap_or(""))
    });

    let n_ok = results
        .iter()
        .filter(|r| r["ok"].as_bool().unwrap_or(false))
        .count();
    let status = if n_ok == results.len() {
        "success"
    } else if n_ok == 0 {
        "error"
    } else {
        "partial"
    };

    if n_ok == 0 {
        return Err(VirtuosoError::Execution(format!(
            "broadcast failed on all {} sessions",
            results.len()
        )));
    }

    Ok(json!({
        "status": status,
        "sessions": results.len(),
        "ok": n_ok,
        "results": results,
    }))
}

pub fn load(file: &str) -> Result<Value> {
    let client = VirtuosoClient::from_env()?;

    if !std::path::Path::new(file).exists() {
        return Err(VirtuosoError::NotFound(format!("file not found: {file}")));
    }

    let result = client.load_il(file)?;

    Ok(json!({
        "status": if result.ok() { "success" } else { "error" },
        "file": file,
        "output": result.output,
        "errors": result.errors,
    }))
}

/// Execute inline SKILL expressions — companion to `load` for one-liners.
///
/// Wraps input in `progn(\n<user>\n)` to:
/// - Enable multi-statement execution without wrapping in progn yourself
/// - Prevent trailing `; comment` from swallowing the closing paren
///
/// Supports two input modes:
/// - `code` provided directly: single expression or multi-line block
/// - `stdin == true`: read from stdin (avoids shell quoting pain)
pub fn eval(code: Option<String>, stdin: bool) -> Result<Value> {
    use std::io::Read;

    // Input validation: mutually exclusive modes
    if stdin && code.is_some() {
        return Err(VirtuosoError::Config(
            "pass SKILL via argument OR --stdin, not both".into(),
        ));
    }

    let skill = if stdin {
        let mut input = String::new();
        std::io::stdin()
            .read_to_string(&mut input)
            .map_err(|e| VirtuosoError::Io(std::io::Error::other(e)))?;
        if input.trim().is_empty() {
            return Err(VirtuosoError::Config(
                "empty SKILL expression from stdin".into(),
            ));
        }
        input
    } else {
        let c = code.ok_or_else(|| VirtuosoError::Config("no SKILL expression provided".into()))?;
        if c.trim().is_empty() {
            return Err(VirtuosoError::Config("empty SKILL expression".into()));
        }
        c
    };

    // Wrap in progn on its own lines so that:
    // - Multi-statement inputs work without user adding progn
    // - Trailing `; comment` doesn't swallow the closing paren
    // - Embedded newlines flow through unchanged
    let wrapped = format!("progn(\n{}\n)", skill);

    let client = VirtuosoClient::from_env()?;
    let result = client.execute_skill(&wrapped, None)?;

    Ok(json!({
        "status": if result.ok() { "success" } else { "error" },
        "output": result.output,
        "errors": result.errors,
        "warnings": result.warnings,
        "execution_time": result.execution_time,
    }))
}
