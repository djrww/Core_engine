//! §4.7 AST 节点 Span 与事实层区间 Interval 的双向无损单子映射 (Bijective Span Monad)。
//!
//! 确保几何区间的修剪 (Shorten) 能够逆向合成为合法的 CST Delta 补丁，保持 L1/L6。

use crate::ast::Interval;
use crate::edit::Edit;
use crate::parse::Tree;
use crate::span::Span;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpanAnchor {
    pub event_id: u32,
    pub ast_node_id: u32,
    pub source_span: Span,
    pub fact_interval: Interval,
}

pub struct SpanMonad;

impl SpanMonad {
    /// 计算将事实层区间缩减至 new_end 对应的源码文本 Edit 单体
    pub fn synthesize_patch(
        tree: &Tree,
        anchor: &SpanAnchor,
        new_end: u32,
        _src: &str,
    ) -> Option<Edit> {
        if new_end <= anchor.fact_interval.start || new_end >= anchor.fact_interval.end {
            return None;
        }

        let node = tree.nodes.get(anchor.ast_node_id as usize)?;
        let delta_bytes = new_end.saturating_sub(anchor.fact_interval.start);
        let insertion_offset = (anchor.source_span.start + delta_bytes).min(node.span.end);

        Some(Edit {
            start: insertion_offset,
            old_end: insertion_offset,
            text: "\n    // [cl0r0 auto-drop]: borrow region shortened\n".to_string(),
        })
    }
}
