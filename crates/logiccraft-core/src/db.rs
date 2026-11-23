//! Bundled SQLite persistence for LogicCraft (`rusqlite` 0.40).

use crate::error::LogicError;
use crate::model::{CpuRegisterFile, GateKind, GateNode, WireEdge};
use crate::LogicEngine;
use rusqlite::{params, Connection};
use std::path::{Path, PathBuf};

pub fn default_db_path() -> PathBuf {
    let base = std::env::var_os("LOCALAPPDATA")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("logiccraft").join("logiccraft.db")
}

pub fn save_to_db(engine: &LogicEngine, path: &Path) -> Result<(), LogicError> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .map_err(|e| LogicError::Storage(format!("create_dir_all failed: {e}")))?;
        }
    }
    let mut conn =
        Connection::open(path).map_err(|e| LogicError::Storage(format!("open db failed: {e}")))?;

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS gates (
            id TEXT PRIMARY KEY,
            label TEXT NOT NULL,
            kind TEXT NOT NULL,
            state INTEGER NOT NULL,
            x REAL NOT NULL,
            y REAL NOT NULL
        );
        CREATE TABLE IF NOT EXISTS wires (
            id TEXT PRIMARY KEY,
            from_gate TEXT NOT NULL,
            to_gate TEXT NOT NULL,
            signal INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS cpu_meta (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            preset_name TEXT NOT NULL,
            tick_count INTEGER NOT NULL,
            regs_json TEXT NOT NULL
        );",
    )
    .map_err(|e| LogicError::Storage(format!("schema init failed: {e}")))?;

    let tx = conn
        .transaction()
        .map_err(|e| LogicError::Storage(format!("tx start failed: {e}")))?;

    tx.execute("DELETE FROM gates", [])
        .map_err(|e| LogicError::Storage(format!("clear gates: {e}")))?;
    tx.execute("DELETE FROM wires", [])
        .map_err(|e| LogicError::Storage(format!("clear wires: {e}")))?;
    tx.execute("DELETE FROM cpu_meta", [])
        .map_err(|e| LogicError::Storage(format!("clear cpu_meta: {e}")))?;

    for g in &engine.gates {
        tx.execute(
            "INSERT INTO gates (id, label, kind, state, x, y) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                g.id,
                g.label,
                g.kind.as_str(),
                i32::from(g.state),
                f64::from(g.x),
                f64::from(g.y)
            ],
        )
        .map_err(|e| LogicError::Storage(format!("insert gate {}: {e}", g.id)))?;
    }

    for w in &engine.wires {
        tx.execute(
            "INSERT INTO wires (id, from_gate, to_gate, signal) VALUES (?1, ?2, ?3, ?4)",
            params![w.id, w.from_gate, w.to_gate, i32::from(w.signal)],
        )
        .map_err(|e| LogicError::Storage(format!("insert wire {}: {e}", w.id)))?;
    }

    let regs_json = serde_json::to_string(&engine.registers)
        .map_err(|e| LogicError::Serialization(e.to_string()))?;
    tx.execute(
        "INSERT INTO cpu_meta (id, preset_name, tick_count, regs_json) VALUES (1, ?1, ?2, ?3)",
        params![engine.active_preset, engine.tick_count as i64, regs_json],
    )
    .map_err(|e| LogicError::Storage(format!("insert cpu_meta: {e}")))?;

    tx.commit()
        .map_err(|e| LogicError::Storage(format!("tx commit: {e}")))?;
    Ok(())
}

pub fn load_from_db(path: &Path) -> Result<Option<LogicEngine>, LogicError> {
    if !path.exists() {
        return Ok(None);
    }
    let conn =
        Connection::open(path).map_err(|e| LogicError::Storage(format!("open db failed: {e}")))?;

    let has_table: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='gates')",
            [],
            |row| row.get(0),
        )
        .map_err(|e| LogicError::Storage(format!("table check failed: {e}")))?;
    if !has_table {
        return Ok(None);
    }

    let mut stmt = conn
        .prepare("SELECT id, label, kind, state, x, y FROM gates ORDER BY rowid ASC")
        .map_err(|e| LogicError::Storage(format!("prepare gates: {e}")))?;
    let gate_rows = stmt
        .query_map([], |row| {
            let kind_str: String = row.get(2)?;
            let state_i: i32 = row.get(3)?;
            let x_f: f64 = row.get(4)?;
            let y_f: f64 = row.get(5)?;
            Ok(GateNode {
                id: row.get(0)?,
                label: row.get(1)?,
                kind: GateKind::parse(&kind_str).unwrap_or(GateKind::And),
                state: state_i != 0,
                x: x_f as f32,
                y: y_f as f32,
            })
        })
        .map_err(|e| LogicError::Storage(format!("query gates: {e}")))?;

    let mut gates = Vec::new();
    for g in gate_rows {
        gates.push(g.map_err(|e| LogicError::Storage(format!("gate row: {e}")))?);
    }
    if gates.is_empty() {
        return Ok(None);
    }

    let mut w_stmt = conn
        .prepare("SELECT id, from_gate, to_gate, signal FROM wires ORDER BY rowid ASC")
        .map_err(|e| LogicError::Storage(format!("prepare wires: {e}")))?;
    let wire_rows = w_stmt
        .query_map([], |row| {
            let sig_i: i32 = row.get(3)?;
            Ok(WireEdge {
                id: row.get(0)?,
                from_gate: row.get(1)?,
                to_gate: row.get(2)?,
                signal: sig_i != 0,
            })
        })
        .map_err(|e| LogicError::Storage(format!("query wires: {e}")))?;

    let mut wires = Vec::new();
    for w in wire_rows {
        wires.push(w.map_err(|e| LogicError::Storage(format!("wire row: {e}")))?);
    }

    let meta: Option<(String, i64, String)> = conn
        .query_row(
            "SELECT preset_name, tick_count, regs_json FROM cpu_meta WHERE id = 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .ok();

    let (active_preset, tick_count, registers) = match meta {
        Some((p, t, rj)) => {
            let regs: CpuRegisterFile = serde_json::from_str(&rj).unwrap_or_default();
            (p, t.max(0) as u64, regs)
        }
        None => ("full_adder".to_string(), 0, CpuRegisterFile::default()),
    };

    let mut engine = LogicEngine {
        active_preset,
        gates,
        wires,
        tick_count,
        registers,
    };
    let _ = engine.evaluate_circuit();
    Ok(Some(engine))
}
