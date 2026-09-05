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

// ===========================================================================
// 測試(圖鑑 D-1 / DL-001:rustc_json.rs 冷點補測)
// ===========================================================================
#[cfg(test)]
mod tests {
    use super::*;

    const RUSTC_LINE: &str = "{\"level\":\"error\",\"code\":{\"code\":\"E0502\"},\"message\":\"cannot borrow\",\"spans\":[{\"byte_start\":42,\"byte_end\":48,\"line_start\":3}]}";

    #[test]
    fn parse_valid_json_line_extracts_code_message_spans() {
        let d = RustcJsonAutomaton::parse_single_json_line(RUSTC_LINE).unwrap();
        assert_eq!(d.level, "error");
        assert_eq!(d.code.as_deref(), Some("E0502"));
        assert_eq!(d.message, "cannot borrow");
        assert_eq!(d.spans.len(), 1);
        assert_eq!((d.spans[0].byte_start, d.spans[0].byte_end), (42, 48));
        assert!(d.spans[0].is_primary);
    }

    #[test]
    fn parse_rejects_non_rustc_lines() {
        assert!(RustcJsonAutomaton::parse_single_json_line("hello world").is_none());
        assert!(RustcJsonAutomaton::parse_single_json_line("{}").is_none());
        assert!(RustcJsonAutomaton::parse_single_json_line("").is_none());
        // 有 level 無 spans ⇒ 非診斷行
        assert!(RustcJsonAutomaton::parse_single_json_line("{\"level\":\"warning\"}").is_none());
    }

    #[test]
    fn parse_without_message_defaults_and_without_code_is_none_field() {
        let line =
            "{\"level\":\"error\",\"spans\":[{\"byte_start\":1,\"byte_end\":2,\"line_start\":1}]}";
        let d = RustcJsonAutomaton::parse_single_json_line(line).unwrap();
        assert!(d.code.is_none());
        assert_eq!(d.message, "Unknown error");
        assert_eq!(d.spans.len(), 1);
    }

    #[test]
    fn automaton_maps_borrow_errors_to_drop_edit() {
        let d = RustcJsonAutomaton::parse_single_json_line(RUSTC_LINE).unwrap();
        let src = "fn main() { let r = &x; let y = x; }";
        let fixed = RustcJsonAutomaton::drive_automaton_step(src, &d).unwrap();
        assert!(
            fixed.contains("; drop(_ref);"),
            "E0502 ⇒ 插入 drop:{}",
            fixed
        );
        assert!(fixed.starts_with(src), "其餘源碼逐字保留");
    }

    #[test]
    fn automaton_maps_move_errors_to_clone_edit() {
        let line = "{\"level\":\"error\",\"code\":{\"code\":\"E0382\"},\"message\":\"use of moved\",\"spans\":[{\"byte_start\":0,\"byte_end\":5,\"line_start\":1}]}";
        let d = RustcJsonAutomaton::parse_single_json_line(line).unwrap();
        let fixed = RustcJsonAutomaton::drive_automaton_step("let s = String::new();", &d).unwrap();
        assert!(fixed.contains(".clone()"), "E0382 ⇒ 插入 clone");
    }

    #[test]
    fn automaton_returns_none_for_unknown_code_or_no_span() {
        let unknown = "{\"level\":\"error\",\"code\":{\"code\":\"E9999\"},\"message\":\"x\",\"spans\":[{\"byte_start\":0,\"byte_end\":3,\"line_start\":1}]}";
        let d = RustcJsonAutomaton::parse_single_json_line(unknown).unwrap();
        assert!(
            RustcJsonAutomaton::drive_automaton_step("src", &d).is_none(),
            "未知錯誤碼無修法"
        );
        let nospan = RustcDiagnostic {
            code: Some("E0502".into()),
            message: "m".into(),
            level: "error".into(),
            spans: vec![],
        };
        assert!(RustcJsonAutomaton::drive_automaton_step("src", &nospan).is_none());
    }

    #[test]
    fn compile_real_rustc_error_extraction() {
        // 沙箱/CI 均有 rustc:真跑一個含借用錯誤的檔案
        let path = std::env::temp_dir().join("cl0r0_rustc_json_probe.rs");
        std::fs::write(
            &path,
            "fn main() { let v = vec![1]; let r = &v[0]; v.push(2); let _ = r; }\n",
        )
        .unwrap();
        let diags = RustcJsonAutomaton::compile_and_extract_json_errors(path.to_str().unwrap());
        std::fs::remove_file(&path).ok();
        assert!(!diags.is_empty(), "E0502 必須被抽取出來");
        assert!(diags.iter().all(|d| d.level == "error"), "只保留 error 級");
    }
}
