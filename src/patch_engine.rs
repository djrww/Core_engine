//! §4.8 端到端补丁合成与闭环自验引擎。
//!
//! Facts (ARS Normalized) ──> Source Patch (Edit) ──> Reparse & Validate

use crate::edit::{apply, Edit};
use crate::parse::{parse, reparse, Tree};

pub struct PatchEngine;

impl PatchEngine {
    /// 执行右修剪并生成源码补丁
    pub fn apply_shorten_repair(
        orig_src: &str,
        orig_tree: &Tree,
        new_end_pos: usize,
    ) -> Result<(String, Tree), String> {
        let pos = (new_end_pos as u32).min(orig_src.len() as u32);
        let edit = Edit {
            start: pos,
            old_end: pos,
            text: "\n    // [cl0r0 auto-drop]: borrow region shortened\n".to_string(),
        };

        let patched_src = apply(orig_src, &edit);

        let full_tree = parse(&patched_src).map_err(|e| format!("全量解析失败: {:?}", e))?;
        let incr_out = reparse(orig_tree, &patched_src, &[edit])
            .map_err(|e| format!("增量重析失败: {:?}", e))?;

        if full_tree.sexp() != incr_out.tree.sexp() {
            return Err("违反 L3/L4 增量重析等价性！".to_string());
        }

        Ok((patched_src, full_tree))
    }
}
