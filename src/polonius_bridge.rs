//! §7.4 Rust 官方 Polonius 借用检查器 Datalog 事实层双向桥接器。
//!
//! 对齐关系:
//!   - `loan_issued_at(Origin, Loan, Point)`
//!   - `borrow_live_at(Origin, Point)`
//!   - `invalidates(Point, Loan)`

use crate::ast::Interval;
use crate::rep_dd::{AState, Ev, K};
use std::fmt::Write;

pub struct PoloniusBridge;

impl PoloniusBridge {
    /// 将事实层 AState 几何配置导出为标准 Polonius Datalog 格式
    pub fn export_to_polonius_facts(s: &AState) -> String {
        let mut out = String::new();
        writeln!(
            out,
            "/// ==================================================="
        )
        .unwrap();
        writeln!(out, "/// CL0 Generated Polonius Datalog Facts").unwrap();
        writeln!(
            out,
            "/// ==================================================="
        )
        .unwrap();

        for e in &s.evs {
            let origin = format!("'orig_{}", e.storage);
            let loan_id = e.id;
            let point_start = e.it.start;
            let point_end = e.it.end;

            writeln!(
                out,
                "loan_issued_at({}, {}, {}).",
                origin, loan_id, point_start
            )
            .unwrap();

            for pt in point_start..point_end {
                writeln!(out, "borrow_live_at({}, {}).", origin, pt).unwrap();
            }

            if e.kind == K::Mut {
                writeln!(out, "invalidates({}, {}).", point_start, loan_id).unwrap();
            }
        }

        out
    }

    /// 从 Polonius Datalog 事实反向重构为 AState
    pub fn import_from_polonius_facts(facts_str: &str) -> AState {
        let mut evs = Vec::new();
        for line in facts_str.lines() {
            let line = line.trim();
            if line.starts_with("loan_issued_at(") && line.ends_with(").") {
                let inner = &line[15..line.len() - 2];
                let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
                if parts.len() == 3 {
                    let storage: u32 = parts[0].trim_start_matches("'orig_").parse().unwrap_or(0);
                    let id: u32 = parts[1].parse().unwrap_or(0);
                    let start: u32 = parts[2].parse().unwrap_or(0);

                    evs.push(Ev {
                        id,
                        storage,
                        kind: K::Sh,
                        it: Interval {
                            start,
                            end: start + 2,
                        },
                    });
                }
            }
        }
        AState::new(evs)
    }
}
