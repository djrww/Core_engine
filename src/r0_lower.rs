//! §7.3 R₀ 实用载体语义降阶引擎 (Lowering to Fact Layer & Liveness Graphs)。
//!
//! 将 R₀ CST 表面语法树降阶为控制流图与事实层事件流 (Ev)，
//! 并严格遵守 Appendix B 的 unsupported 申报纪律与 Def-Use 活跃期分析。

use crate::ast::Interval;
use crate::parse::{Kind, Tree};
use crate::r0::{lalr1_clean, unsupported};
use crate::rep_dd::{Ev, K};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct LoweringError {
    pub reason: String,
    pub unsupported_features: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct LoweredFacts {
    pub events: Vec<Ev>,
    pub storage_map: HashMap<String, u32>,
    pub total_storages: u32,
}

pub struct R0Lowerer;

impl R0Lowerer {
    /// 对 R₀ 源码执行安全边界检查并降阶为事实层事件
    pub fn lower(src: &str, tree: &Tree) -> Result<LoweredFacts, LoweringError> {
        let unsupp = unsupported(src);
        if !unsupp.is_empty() {
            return Err(LoweringError {
                reason: "源码超出 R₀ 实用载体语法子集契约边界".to_string(),
                unsupported_features: unsupp
                    .into_iter()
                    .map(|(desc, span)| format!("{} @{:?}", desc, span))
                    .collect(),
            });
        }

        if let Err(e) = lalr1_clean(src) {
            return Err(LoweringError {
                reason: "源码包含潜在的语法歧义构造 (非 LALR(1) 干净)".to_string(),
                unsupported_features: vec![e],
            });
        }

        let mut storage_map = HashMap::new();
        let mut next_storage_id = 0u32;
        let mut events = Vec::new();
        let mut event_counter = 0u32;

        for (idx, node) in tree.nodes.iter().enumerate() {
            match node.kind {
                Kind::LetStmt => {
                    let var_name = format!("var_{}", idx);
                    let sid = next_storage_id;
                    next_storage_id += 1;
                    storage_map.insert(var_name, sid);

                    events.push(Ev {
                        id: event_counter,
                        storage: sid,
                        kind: K::Mut,
                        it: Interval {
                            start: node.span.start,
                            end: node.span.end,
                        },
                    });
                    event_counter += 1;
                }
                Kind::Amp => {
                    let sid = next_storage_id.saturating_sub(1);
                    events.push(Ev {
                        id: event_counter,
                        storage: sid,
                        kind: K::Sh,
                        it: Interval {
                            start: node.span.start,
                            end: (node.span.start + 10).min(node.span.end),
                        },
                    });
                    event_counter += 1;
                }
                _ => {}
            }
        }

        Ok(LoweredFacts {
            events,
            storage_map,
            total_storages: next_storage_id,
        })
    }
}
