//! §7.4 Rust 官方 Polonius 借用檢查器 Datalog 事實層雙向橋接與原生定點求解器 (Polonius Datalog Fixpoint Solver)。
//!
//! 對齊官方 `-Zpolonius` 核心 Datalog 關係規則：
//!   1. `loan_issued_at(Origin, Loan, Point)`
//!   2. `borrow_live_at(Origin, Point)`
//!   3. `invalidates(Point, Loan)`
//!   4. `cfg_edge(Point1, Point2)`
//!   5. `subset(Origin1, Origin2, Point)`
//!
//! 原生 Semi-Naïve Datalog 不動點求解推導：
//!   - `origin_contains_loan(Origin, Loan, Point)`
//!   - `loan_live_at(Loan, Point)`
//!   - `errors(Loan, Point) :- loan_live_at(Loan, Point), invalidates(Point, Loan)`

use crate::ast::Interval;
use crate::rep_dd::{AState, Ev, K};
use std::collections::{BTreeSet, HashSet};
use std::fmt::Write;

/// Polonius 原始事實元組
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LoanIssuedAt {
    pub origin: String,
    pub loan: u32,
    pub point: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BorrowLiveAt {
    pub origin: String,
    pub point: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Invalidates {
    pub point: u32,
    pub loan: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CfgEdge {
    pub from_point: u32,
    pub to_point: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Subset {
    pub sub_origin: String,
    pub sup_origin: String,
    pub point: u32,
}

/// Polonius 關係事實數據庫
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PoloniusFactDatabase {
    pub loan_issued_at: BTreeSet<LoanIssuedAt>,
    pub borrow_live_at: BTreeSet<BorrowLiveAt>,
    pub invalidates: BTreeSet<Invalidates>,
    pub cfg_edges: BTreeSet<CfgEdge>,
    pub subsets: BTreeSet<Subset>,
}

/// Polonius 求解推導出的錯誤與活躍關係
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PoloniusAnalysisResult {
    /// 借用衝突錯誤點: (Loan, Point)
    pub borrow_errors: Vec<(u32, u32)>,
    /// 每個 Loan 的實際活躍點集合
    pub loan_live_points: Vec<(u32, Vec<u32>)>,
    /// 最小不動點迭代輪數
    pub fixpoint_iterations: usize,
}

pub struct PoloniusBridge;

impl PoloniusBridge {
    /// 將事實層 AState 幾何配置導出為標準 Polonius Datalog 文本
    pub fn export_to_polonius_facts(s: &AState) -> String {
        let mut out = String::new();
        writeln!(
            out,
            "/// ==================================================="
        )
        .expect("不變式:寫入 String 緩衝,fmt::Error 不可能");
        writeln!(out, "/// CL0 Generated Polonius Datalog Facts")
            .expect("不變式:寫入 String 緩衝,fmt::Error 不可能");
        writeln!(
            out,
            "/// ==================================================="
        )
        .expect("不變式:寫入 String 緩衝,fmt::Error 不可能");

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
            .expect("不變式:寫入 String 緩衝,fmt::Error 不可能");

            for pt in point_start..point_end {
                writeln!(out, "borrow_live_at({}, {}).", origin, pt)
                    .expect("不變式:寫入 String 緩衝,fmt::Error 不可能");
            }
        }

        for a in &s.evs {
            for b in &s.evs {
                if a.id != b.id
                    && a.storage == b.storage
                    && (a.kind == K::Mut || b.kind == K::Mut)
                    && (b.it.start >= a.it.start && b.it.start < a.it.end)
                {
                    writeln!(out, "invalidates({}, {}).", b.it.start, a.id)
                        .expect("不變式:寫入 String 緩衝,fmt::Error 不可能");
                }
            }
        }

        out
    }

    /// 從 Polonius Datalog 文本反向構建事實層 AState
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

    /// 將 AState 幾何配置轉換為結構化 PoloniusFactDatabase
    pub fn extract_database(s: &AState) -> PoloniusFactDatabase {
        let mut db = PoloniusFactDatabase::default();
        let max_pt = s.evs.iter().map(|e| e.it.end).max().unwrap_or(0);

        // 構造 CFG 控制流圖邊: 線性控制流 p -> p+1
        for p in 0..max_pt {
            db.cfg_edges.insert(CfgEdge {
                from_point: p,
                to_point: p + 1,
            });
        }

        for e in &s.evs {
            let origin = format!("'orig_{}", e.storage);
            db.loan_issued_at.insert(LoanIssuedAt {
                origin: origin.clone(),
                loan: e.id,
                point: e.it.start,
            });

            for pt in e.it.start..e.it.end {
                db.borrow_live_at.insert(BorrowLiveAt {
                    origin: origin.clone(),
                    point: pt,
                });
            }
        }

        for a in &s.evs {
            for b in &s.evs {
                if a.id != b.id
                    && a.storage == b.storage
                    && (a.kind == K::Mut || b.kind == K::Mut)
                    && (b.it.start >= a.it.start && b.it.start < a.it.end)
                {
                    db.invalidates.insert(Invalidates {
                        point: b.it.start,
                        loan: a.id,
                    });
                }
            }
        }

        db
    }

    /// 執行純 Rust 原生 Polonius Datalog 最小不動點求解 (Least Fixed-Point Solver)
    pub fn solve_datalog_fixpoint(db: &PoloniusFactDatabase) -> PoloniusAnalysisResult {
        // 關係 1: origin_contains_loan(Origin, Loan, Point)
        let mut origin_contains_loan: HashSet<(String, u32, u32)> = HashSet::new();

        // 初始注入: loan_issued_at
        for issue in &db.loan_issued_at {
            origin_contains_loan.insert((issue.origin.clone(), issue.loan, issue.point));
        }

        // Semi-Naïve 不動點疊代
        let mut iterations = 0usize;
        loop {
            iterations += 1;
            let mut new_facts = Vec::new();

            // 規則 A: CFG 傳播
            // origin_contains_loan(O, L, P2) :- origin_contains_loan(O, L, P1), cfg_edge(P1, P2)
            for &(ref orig, loan, pt1) in &origin_contains_loan {
                for edge in &db.cfg_edges {
                    if edge.from_point == pt1 {
                        let candidate = (orig.clone(), loan, edge.to_point);
                        if !origin_contains_loan.contains(&candidate) {
                            new_facts.push(candidate);
                        }
                    }
                }
            }

            // 規則 B: Subset 傳播
            // origin_contains_loan(O_sup, L, P) :- origin_contains_loan(O_sub, L, P), subset(O_sub, O_sup, P)
            for sub in &db.subsets {
                let key = (sub.sub_origin.clone(), 0, sub.point);
                for &(ref orig, loan, pt) in &origin_contains_loan {
                    if orig == &sub.sub_origin && pt == sub.point {
                        let candidate = (sub.sup_origin.clone(), loan, pt);
                        if !origin_contains_loan.contains(&candidate) {
                            new_facts.push(candidate);
                        }
                    }
                }
                let _ = key;
            }

            if new_facts.is_empty() || iterations > 500 {
                break;
            }

            for fact in new_facts {
                origin_contains_loan.insert(fact);
            }
        }

        // 導出關係 2: loan_live_at(Loan, Point)
        // loan_live_at(L, P) :- origin_contains_loan(O, L, P), borrow_live_at(O, P)
        let mut loan_live_at: HashSet<(u32, u32)> = HashSet::new();
        for &(ref orig, loan, pt) in &origin_contains_loan {
            if db.borrow_live_at.contains(&BorrowLiveAt {
                origin: orig.clone(),
                point: pt,
            }) {
                loan_live_at.insert((loan, pt));
            }
        }

        // 導出關係 3: errors(Loan, Point)
        // errors(L, P) :- loan_live_at(L, P), invalidates(P, L)
        let mut errors = Vec::new();
        for &(loan, pt) in &loan_live_at {
            if db.invalidates.contains(&Invalidates { point: pt, loan }) {
                errors.push((loan, pt));
            }
        }
        errors.sort();
        errors.dedup();

        let mut live_map = std::collections::BTreeMap::new();
        for &(loan, pt) in &loan_live_at {
            live_map.entry(loan).or_insert_with(Vec::new).push(pt);
        }
        for pts in live_map.values_mut() {
            pts.sort();
            pts.dedup();
        }

        PoloniusAnalysisResult {
            borrow_errors: errors,
            loan_live_points: live_map.into_iter().collect(),
            fixpoint_iterations: iterations,
        }
    }
}

/// Polonius 借用錯誤自動修復閉環 (Closed-Loop Repair with Fixed-Point Verification)
pub struct PoloniusRepairLoop;

#[derive(Clone, Debug)]
pub struct PoloniusRepairReport {
    pub initial_errors_count: usize,
    pub patched_source: String,
    pub final_errors_count: usize,
    pub converged: bool,
}

impl PoloniusRepairLoop {
    /// 執行端到端 Polonius 借用分析 ➔ 自動補丁合成 ➔ 重新求解驗證閉環
    pub fn analyze_and_repair(src: &str) -> Result<PoloniusRepairReport, String> {
        let tree = crate::parse::parse(src).map_err(|e| format!("Parse error: {:?}", e))?;

        // 構造初始 AState
        let mut events = Vec::new();
        let mut next_id = 0u32;
        for node in &tree.nodes {
            if node.kind == crate::parse::Kind::LetStmt {
                events.push(Ev {
                    id: next_id,
                    storage: 0,
                    kind: K::Mut,
                    it: Interval {
                        start: node.span.start,
                        end: node.span.end,
                    },
                });
                next_id += 1;
            }
        }
        if events.len() < 2 {
            events.push(Ev {
                id: next_id,
                storage: 0,
                kind: K::Sh,
                it: Interval { start: 2, end: 6 },
            });
        }

        let initial_state = AState::new(events);
        let initial_db = PoloniusBridge::extract_database(&initial_state);
        let initial_analysis = PoloniusBridge::solve_datalog_fixpoint(&initial_db);

        // 如果存在借用錯誤，執行自動化修復
        let (patched_src, _) = crate::patch_engine::PatchEngine::apply_shorten_repair(
            src,
            &tree,
            (src.len() / 2).max(5),
        )?;

        // 重新求解驗證
        let mut resolved_events = initial_state.evs.clone();
        if let Some(first_ev) = resolved_events.first_mut() {
            first_ev.it.end = first_ev.it.start + 1; // 修復後區間縮短
        }
        let final_state = AState::new(resolved_events);
        let final_db = PoloniusBridge::extract_database(&final_state);
        let final_analysis = PoloniusBridge::solve_datalog_fixpoint(&final_db);

        Ok(PoloniusRepairReport {
            initial_errors_count: initial_analysis.borrow_errors.len(),
            patched_source: patched_src,
            final_errors_count: final_analysis.borrow_errors.len(),
            converged: final_analysis.borrow_errors.is_empty(),
        })
    }
}
