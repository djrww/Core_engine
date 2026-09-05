//! §6.3 强制 rustc 输出 .json 诊断格式，并套入重写自动机 (ARS Automaton)。
//!
//! 流程: rustc --error-format=json ──> RustcDiagnostic JSON ──> Fact Layer AState ──> DD Automaton

use crate::edit::{apply, Edit};
use std::process::Command;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiagnosticSpan {
    pub file_name: String,
    pub byte_start: usize,
    pub byte_end: usize,
    pub line_start: usize,
    pub line_end: usize,
    pub is_primary: bool,
    pub label: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RustcDiagnostic {
    pub code: Option<String>,
    pub message: String,
    pub level: String,
    pub spans: Vec<DiagnosticSpan>,
}

pub struct RustcJsonAutomaton;

impl RustcJsonAutomaton {
    /// 运行 rustc 编译并强制要求 .json 格式输出
    pub fn compile_and_extract_json_errors(source_file: &str) -> Vec<RustcDiagnostic> {
        let output = Command::new("rustc")
            .arg("--error-format=json")
            .arg("--json=diagnostic-rendered-ansi")
            .arg("--crate-type=lib")
            .arg(source_file)
            .output();

        let mut diagnostics = Vec::new();
        if let Ok(out) = output {
            let stderr_str = String::from_utf8_lossy(&out.stderr);
            for line in stderr_str.lines() {
                if let Some(diag) = Self::parse_single_json_line(line) {
                    if diag.level == "error" {
                        diagnostics.push(diag);
                    }
                }
            }
        }
        diagnostics
    }

    /// 轻量级零依赖 JSON 字段提取器 (解析 code 与 spans)
    pub fn parse_single_json_line(json: &str) -> Option<RustcDiagnostic> {
        if !json.contains("\"level\":") || !json.contains("\"spans\":") {
            return None;
        }

        let code = if let Some(pos) = json.find("\"code\":{\"code\":\"") {
            let rest = &json[pos + 16..];
            rest.split('"').next().map(|s| s.to_string())
        } else {
            None
        };

        let message = if let Some(pos) = json.find("\"message\":\"") {
            let rest = &json[pos + 11..];
            rest.split('"').next().unwrap_or("").to_string()
        } else {
            "Unknown error".to_string()
        };

        let mut spans = Vec::new();
        if let Some(pos) = json.find("\"byte_start\":") {
            let rest = &json[pos + 13..];
            if let Some(val_str) = rest.split(',').next() {
                if let Ok(b_start) = val_str.trim().parse::<usize>() {
                    if let Some(end_pos) = json.find("\"byte_end\":") {
                        let end_rest = &json[end_pos + 11..];
                        if let Some(end_val_str) = end_rest.split(',').next() {
                            if let Ok(b_end) = end_val_str.trim().parse::<usize>() {
                                spans.push(DiagnosticSpan {
                                    file_name: "".to_string(),
                                    byte_start: b_start,
                                    byte_end: b_end,
                                    line_start: 0,
                                    line_end: 0,
                                    is_primary: true,
                                    label: None,
                                });
                            }
                        }
                    }
                }
            }
        }

        Some(RustcDiagnostic {
            code,
            message,
            level: "error".to_string(),
            spans,
        })
    }

    /// 将 JSON 诊断套入 ARS 自动机：将 JSON 错误点映射为事实层事件并执行闭环修复
    pub fn drive_automaton_step(src: &str, diag: &RustcDiagnostic) -> Option<String> {
        let error_code = diag.code.as_deref().unwrap_or("E0502");
        let span = diag.spans.first()?;
        let pos = span.byte_end.min(src.len()) as u32;

        let edit = match error_code {
            "E0502" | "E0499" | "E0503" => Some(Edit {
                start: pos,
                old_end: pos,
                text: "; drop(_ref);".to_string(),
            }),
            "E0505" | "E0382" => Some(Edit {
                start: pos,
                old_end: pos,
                text: ".clone()".to_string(),
            }),
            _ => None,
        }?;

        Some(apply(src, &edit))
    }
}
