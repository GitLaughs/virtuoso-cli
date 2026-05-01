use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SkillEntry {
    pub ts: String,
    pub skill: String,
    pub ok: bool,
    pub output: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CmdEntry {
    pub ts: String,
    pub session: Option<String>,
    pub cmd: Vec<String>,
    pub exit_code: i32,
}

pub fn history_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(".cache/virtuoso_bridge/history")
}

pub fn append_skill(session_id: &str, skill: &str, ok: bool, output: &str) {
    let dir = history_dir();
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join(format!("{session_id}.jsonl"));
    let entry = SkillEntry {
        ts: Utc::now().to_rfc3339(),
        skill: skill.to_string(),
        ok,
        output: output.chars().take(512).collect(),
    };
    if let (Ok(line), Ok(mut f)) = (
        serde_json::to_string(&entry),
        std::fs::OpenOptions::new().create(true).append(true).open(&path),
    ) {
        let _ = writeln!(f, "{line}");
    }
}

pub fn append_cmd(args: &[String], session: Option<&str>, exit_code: i32) {
    let dir = history_dir();
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("cmd.jsonl");
    let entry = CmdEntry {
        ts: Utc::now().to_rfc3339(),
        session: session.map(String::from),
        cmd: args.to_vec(),
        exit_code,
    };
    if let (Ok(line), Ok(mut f)) = (
        serde_json::to_string(&entry),
        std::fs::OpenOptions::new().create(true).append(true).open(&path),
    ) {
        let _ = writeln!(f, "{line}");
    }
}

pub fn load_skill(session_id: &str) -> Vec<SkillEntry> {
    let path = history_dir().join(format!("{session_id}.jsonl"));
    std::fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect()
}

pub fn load_cmd(session_filter: Option<&str>, limit: usize) -> Vec<CmdEntry> {
    let path = history_dir().join("cmd.jsonl");
    let all: Vec<CmdEntry> = std::fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .filter(|e: &CmdEntry| {
            session_filter.map_or(true, |id| e.session.as_deref() == Some(id))
        })
        .collect();
    if limit > 0 && all.len() > limit {
        all[all.len() - limit..].to_vec()
    } else {
        all
    }
}
