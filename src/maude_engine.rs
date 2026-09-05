//! # Maude 重寫邏輯引擎 (Maude Rewriting Logic System Engine)
//!
//! 參考文獻:
//!   - José Meseguer (1992), "Conditional Rewriting Logic as a Unified Model of Concurrency" (TCS)
//!   - Manuel Clavel et al. (2007), "All About Maude - A High-Performance Logical Framework" (LNCS 4350)
//!
//! 提供完整的重寫邏輯 (Rewriting Logic, RL) 執行與模型檢驗環境：
//!   1. **Functional Module (`fmod ... endfm`)**: 代數特徵、種類 (Sorts)、算子 (Ops) 與等式簡化 (`eq lhs = rhs`)
//!   2. **System Module (`mod ... endm`)**: 併發狀態轉移重寫規則 (`rl [label] : lhs => rhs`)
//!   3. **規約引擎 (`reduce`)**: 基於等式將項計算至唯一正規形 ($t \downarrow_E$)
//!   4. **模型檢驗搜索 (`search`)**: 全局可達狀態空間廣度搜索與死鎖/目標不變量模型檢驗
//!   5. **Maude 規範導出 (`export_maude`)**: 導出為標準 Maude 3.x 兼容腳本

use crate::dag_term::{DagNode, DagPool, TermId};
use crate::discrimination_tree::DiscriminationTree;
use crate::unification::unify;
use std::collections::{HashSet, VecDeque};
use std::fmt::Write;

/// 重寫邏輯等式 (Equation: eq lhs = rhs)
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaudeEquation {
    pub label: Option<String>,
    pub lhs: TermId,
    pub rhs: TermId,
}

/// 重寫邏輯規則 (Rule: rl [label] : lhs => rhs)
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaudeRule {
    pub label: String,
    pub lhs: TermId,
    pub rhs: TermId,
}

/// Maude 重寫邏輯模組 (Maude System Module: `mod NAME is ... endm`)
#[derive(Clone, Debug)]
pub struct MaudeModule {
    pub name: String,
    pub sorts: Vec<String>,
    pub equations: Vec<MaudeEquation>,
    pub rules: Vec<MaudeRule>,
    eq_index: DiscriminationTree<usize>,
    rule_index: DiscriminationTree<usize>,
}

impl MaudeModule {
    /// 構造新的 Maude 模組
    pub fn new(name: &str, sorts: Vec<&str>) -> Self {
        MaudeModule {
            name: name.to_string(),
            sorts: sorts.into_iter().map(|s| s.to_string()).collect(),
            equations: Vec::new(),
            rules: Vec::new(),
            eq_index: DiscriminationTree::new(),
            rule_index: DiscriminationTree::new(),
        }
    }

    /// 添加等式 (`eq lhs = rhs`)
    pub fn add_equation(&mut self, pool: &DagPool, label: Option<&str>, lhs: TermId, rhs: TermId) {
        let eq_idx = self.equations.len();
        self.equations.push(MaudeEquation {
            label: label.map(|s| s.to_string()),
            lhs,
            rhs,
        });
        self.eq_index.insert(pool, lhs, eq_idx);
    }

    /// 添加重寫規則 (`rl [label] : lhs => rhs`)
    pub fn add_rule(&mut self, pool: &DagPool, label: &str, lhs: TermId, rhs: TermId) {
        let rule_idx = self.rules.len();
        self.rules.push(MaudeRule {
            label: label.to_string(),
            lhs,
            rhs,
        });
        self.rule_index.insert(pool, lhs, rule_idx);
    }

    /// Maude 規約命令 (`reduce`): 透過等式將項簡化至等式正規形
    pub fn reduce(&self, pool: &mut DagPool, term: TermId) -> TermId {
        let mut curr = term;
        let mut steps = 0usize;

        while steps < 500 {
            steps += 1;
            let mut rewritten = false;

            // 頂層辨別樹泛化匹配
            let candidate_eqs = self.eq_index.query_generalizations(pool, curr);
            for eq_idx in candidate_eqs {
                let eq = &self.equations[eq_idx];
                if let Some(subst) = unify(pool, eq.lhs, curr) {
                    curr = subst.apply(pool, eq.rhs);
                    rewritten = true;
                    break;
                }
            }

            if !rewritten {
                // 子項遞歸規約
                if let DagNode::App(f, args) = pool.get(curr).clone() {
                    let mut new_args = Vec::new();
                    let mut arg_changed = false;
                    for &arg in &args {
                        let red_arg = self.reduce(pool, arg);
                        if red_arg != arg {
                            arg_changed = true;
                        }
                        new_args.push(red_arg);
                    }
                    if arg_changed {
                        curr = pool.app(&f, new_args);
                        rewritten = true;
                    }
                }
            }

            if !rewritten {
                break;
            }
        }

        curr
    }

    /// 單步非確定性重寫: 枚舉當前狀態出發的所有後繼狀態
    pub fn rewrite_step(&self, pool: &mut DagPool, term: TermId) -> Vec<(String, TermId)> {
        let mut successors = Vec::new();

        // 頂層規則匹配
        let candidate_rules = self.rule_index.query_generalizations(pool, term);
        for rule_idx in candidate_rules {
            let rule = &self.rules[rule_idx];
            if let Some(subst) = unify(pool, rule.lhs, term) {
                let next_term = subst.apply(pool, rule.rhs);
                let normalized = self.reduce(pool, next_term);
                successors.push((rule.label.clone(), normalized));
            }
        }

        // 子項位置重寫 (Context Rewriting)
        if let DagNode::App(f, args) = pool.get(term).clone() {
            for (i, &arg) in args.iter().enumerate() {
                let sub_succs = self.rewrite_step(pool, arg);
                for (lbl, next_sub) in sub_succs {
                    let mut next_args = args.clone();
                    next_args[i] = next_sub;
                    let next_term = pool.app(&f, next_args);
                    let normalized = self.reduce(pool, next_term);
                    successors.push((lbl, normalized));
                }
            }
        }

        successors
    }

    /// Maude 狀態空間模型檢驗命令 (`search`): 搜索可達狀態空間中滿足模式的狀態
    pub fn search(
        &self,
        pool: &mut DagPool,
        initial: TermId,
        target_pattern: TermId,
        max_depth: usize,
    ) -> Vec<TermId> {
        let mut matches = Vec::new();
        let mut visited: HashSet<TermId> = HashSet::new();
        let mut queue: VecDeque<(TermId, usize)> = VecDeque::new();

        let initial_norm = self.reduce(pool, initial);
        visited.insert(initial_norm);
        queue.push_back((initial_norm, 0));

        while let Some((curr, depth)) = queue.pop_front() {
            // 檢查當前狀態是否匹配目標模式
            if unify(pool, target_pattern, curr).is_some() {
                matches.push(curr);
            }

            if depth >= max_depth {
                continue;
            }

            for (_, next_state) in self.rewrite_step(pool, curr) {
                if visited.insert(next_state) {
                    queue.push_back((next_state, depth + 1));
                }
            }
        }

        matches
    }

    /// 導出為標準 Maude 3.x 系統腳本 (`.maude`)
    pub fn export_maude(&self, pool: &DagPool) -> String {
        let mut out = String::new();
        writeln!(out, "--- ==========================================")
            .expect("不變式:寫入 String 緩衝,fmt::Error 不可能");
        writeln!(out, "--- Maude Formal System Specification: {}", self.name)
            .expect("不變式:寫入 String 緩衝,fmt::Error 不可能");
        writeln!(out, "--- Generated by Core_engine cl0r0 Rewriting Logic")
            .expect("不變式:寫入 String 緩衝,fmt::Error 不可能");
        writeln!(out, "--- ==========================================")
            .expect("不變式:寫入 String 緩衝,fmt::Error 不可能");
        writeln!(out, "mod {} is", self.name).expect("不變式:寫入 String 緩衝,fmt::Error 不可能");
        writeln!(out, "  sorts {} .", self.sorts.join(" "))
            .expect("不變式:寫入 String 緩衝,fmt::Error 不可能");

        // 導出等式
        for eq in &self.equations {
            let lhs_str = pool.format_term(eq.lhs);
            let rhs_str = pool.format_term(eq.rhs);
            if let Some(ref lbl) = eq.label {
                writeln!(out, "  eq [{}] : {} = {} .", lbl, lhs_str, rhs_str)
                    .expect("不變式:寫入 String 緩衝,fmt::Error 不可能");
            } else {
                writeln!(out, "  eq {} = {} .", lhs_str, rhs_str)
                    .expect("不變式:寫入 String 緩衝,fmt::Error 不可能");
            }
        }

        // 導出規則
        for rl in &self.rules {
            let lhs_str = pool.format_term(rl.lhs);
            let rhs_str = pool.format_term(rl.rhs);
            writeln!(out, "  rl [{}] : {} => {} .", rl.label, lhs_str, rhs_str)
                .expect("不變式:寫入 String 緩衝,fmt::Error 不可能");
        }

        writeln!(out, "endm").expect("不變式:寫入 String 緩衝,fmt::Error 不可能");
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_maude_module_reduce_and_rewrite() {
        let mut pool = DagPool::new();
        let mut m = MaudeModule::new("CL0-REPAIR", vec!["State", "Interval"]);

        // 等式 1: add(0, x) = x
        let zero = pool.constant("0");
        let x = pool.var(0);
        let add_0_x = pool.app("add", vec![zero, x]);
        m.add_equation(&pool, Some("id-add"), add_0_x, x);

        // 規約驗證: add(0, a) => a
        let a = pool.constant("a");
        let add_0_a = pool.app("add", vec![zero, a]);
        let reduced = m.reduce(&mut pool, add_0_a);
        assert_eq!(reduced, a);

        // 規則 1: step: a => b
        let b = pool.constant("b");
        m.add_rule(&pool, "step-ab", a, b);

        let succs = m.rewrite_step(&mut pool, a);
        assert_eq!(succs.len(), 1);
        assert_eq!(succs[0].0, "step-ab");
        assert_eq!(succs[0].1, b);

        // 狀態搜索 search: 從 a 搜索 b
        let found = m.search(&mut pool, a, b, 5);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0], b);

        let maude_script = m.export_maude(&pool);
        assert!(maude_script.contains("mod CL0-REPAIR is"));
        assert!(maude_script.contains("rl [step-ab] : a => b ."));
    }
}
