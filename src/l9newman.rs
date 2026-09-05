//! §4.3 機械的 Newman 通道:
//!   **SN ∧ WCR ⇒ CR ⇒ 唯一正規形**(Newman 引理,1942)。
//!
//! 這裡不是「引一句定理」,而是把前提逐一**機械驗證**:
//!   (T8) 終止性(SN):μ = (|E_red|, |Err_rustc|) 良基 + 每步嚴格遞減
//!        ⇒ 重寫序列不可能無限;
//!   (T9) 局部合流(WCR):對每個狀態與每對「臨界步」(s→a, s→b),
//!        機械搜索共同後繼(可回合性);
//!   (T10) 推論:唯一正規形 —— 對每個狀態,窮舉所有極大歸約序列,
//!        終點必為同一個狀態(唯一正規形)。
//!
//! 對 `CommutativeTrim` 菜單,三項全部機械通過;對 `Naive` 菜單,
//! 機械檢查**如實報告**其不可回合臨界對與多正規形(這正是報告 §4.3 的
//! 「同一錯誤、兩次修復、兩種結果」)。

use crate::rep::{apply, enumerate_states, l8_check, AState, Menu, Policy, Rule, K};
use std::collections::{BTreeSet, HashMap};

/// 正規形/狀態閉包的規範化鍵:統一引用 `rep::canon_state_key`
/// (D-04:與 dd_checker / rep_dd 的鍵構造共享同一份單一真相)。
type StateKey = crate::rep::StateKey<K>;

#[derive(Clone, Debug)]
pub struct NewmanReport {
    pub menu: Menu,
    pub policy: Policy,
    pub states: usize,
    pub l8_violations: Vec<(AState, AState, Rule)>,
    pub critical_pairs: usize,
    pub non_joinable: Vec<(AState, Rule, Rule, AState, AState)>,
    pub unique_nf_states: usize,
    pub multi_nf: Vec<(AState, Vec<AState>)>,
    pub conclusion: &'static str,
}

/// 從 s 出發的 bf(深度 ≤ depth)狀態閉包。
fn closure(s: &AState, menu: Menu, policy: Policy, depth: usize) -> HashMap<StateKey, AState> {
    let mut seen: HashMap<_, AState> = HashMap::new();
    seen.insert(s.canon_key(), s.clone());
    let mut frontier = vec![s.clone()];
    for _ in 0..depth {
        if frontier.is_empty() {
            break;
        }
        let mut next = Vec::new();
        for st in frontier {
            for r in menu.applicable(&st, policy) {
                if let Some(s2) = apply(&st, r) {
                    let k = s2.canon_key();
                    if let std::collections::hash_map::Entry::Vacant(e) = seen.entry(k) {
                        e.insert(s2.clone());
                        next.push(s2);
                    }
                }
            }
        }
        frontier = next;
    }
    seen
}

/// 可回合性:兩個狀態的閉包有交集(深度 depth)。
pub fn joinable(a: &AState, b: &AState, menu: Menu, policy: Policy, depth: usize) -> bool {
    let ca = closure(a, menu, policy, depth);
    let cb = closure(b, menu, policy, depth);
    ca.keys().any(|k| cb.contains_key(k))
}

/// 從 s 出發收集所有正規形(極大歸約序列的終點)。
pub fn normal_forms(s: &AState, menu: Menu, policy: Policy, depth: usize) -> Vec<StateKey> {
    let mut nfs: BTreeSet<StateKey> = BTreeSet::new();
    let mut stack = vec![(s.clone(), 0usize)];
    while let Some((st, d)) = stack.pop() {
        if d >= depth {
            nfs.insert(st.canon_key());
            continue;
        }
        let rules = menu.applicable(&st, policy);
        if rules.is_empty() {
            nfs.insert(st.canon_key());
            continue;
        }
        for r in rules {
            if let Some(s2) = apply(&st, r) {
                stack.push((s2, d + 1));
            }
        }
    }
    nfs.into_iter().collect()
}

pub fn newman_check(
    menu: Menu,
    policy: Policy,
    n_events: usize,
    max_coord: u32,
    depth: usize,
) -> NewmanReport {
    let all = enumerate_states(n_events, max_coord);
    // 排除重複 start(演示域:start 嚴格遞增 —— 詳見 rep.rs 的註記)
    let states: Vec<AState> = all
        .into_iter()
        .filter(|s| {
            let mut starts: Vec<u32> = s.evs.iter().map(|e| e.it.start).collect();
            starts.sort();
            starts.windows(2).all(|w| w[0] != w[1])
        })
        .collect();

    let mut l8_violations = Vec::new();
    let mut critical_pairs = 0usize;
    let mut non_joinable = Vec::new();
    let mut unique_ok = 0usize;
    let mut multi_nf = Vec::new();

    for s in &states {
        if let Some(v) = l8_check(s, menu, policy) {
            l8_violations.push(v);
        }
        let rules = menu.applicable(s, policy);
        let mut x = 0usize;
        while x < rules.len() {
            let mut y = x + 1;
            while y < rules.len() {
                critical_pairs += 1;
                if let (Some(a), Some(b)) = (apply(s, rules[x]), apply(s, rules[y])) {
                    if !joinable(&a, &b, menu, policy, depth) {
                        non_joinable.push((s.clone(), rules[x], rules[y], a, b));
                    }
                }
                y += 1;
            }
            x += 1;
        }
        if non_joinable.len() > 32 {
            // 已找到足夠反例
            break;
        }
        let nfs = normal_forms(s, menu, policy, depth);
        if nfs.len() == 1 {
            unique_ok += 1;
        } else {
            multi_nf.push((
                s.clone(),
                nfs.iter()
                    .map(|k| {
                        // 重新構造狀態僅用於報告
                        let evs = k
                            .iter()
                            .map(|&(id, storage, kind, st, en)| crate::rep::Ev {
                                id,
                                storage,
                                kind,
                                it: crate::ast::Interval { start: st, end: en },
                            })
                            .collect();
                        AState::new(evs)
                    })
                    .collect(),
            ));
        }
    }

    let conclusion = if l8_violations.is_empty() && non_joinable.is_empty() && multi_nf.is_empty() {
        "SN ∧ WCR ⇒ CR ⇒ 唯一正規形(機械驗證通過)"
    } else if !l8_violations.is_empty() {
        "L8 違反:存在不嚴格遞減 μ 的施用(側條件是定律的載體)"
    } else if !non_joinable.is_empty() {
        "WCR 違反:存在不可回合臨界對 ⇒ 不滿足 Newman 前提 ⇒ 正規形不唯一"
    } else {
        "多正規形:同一狀態存在不同極大歸約終點"
    };

    NewmanReport {
        menu,
        policy,
        states: states.len(),
        l8_violations,
        critical_pairs,
        non_joinable,
        unique_nf_states: unique_ok,
        multi_nf,
        conclusion,
    }
}

// ===========================================================================
// 測試(圖鑑 D-1 / DL-001:l9newman.rs 冷點補測)
// ===========================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Interval;
    use crate::rep::Ev;

    fn ev(id: u32, storage: u32, kind: K, s: u32, e: u32) -> Ev {
        Ev {
            id,
            storage,
            kind,
            it: Interval { start: s, end: e },
        }
    }

    #[test]
    fn joinable_is_reflexive_and_trims_to_common_successor() {
        let s = AState::new(vec![ev(0, 0, K::Mut, 0, 4), ev(1, 0, K::Sh, 2, 4)]);
        assert!(
            joinable(&s, &s, Menu::CommutativeTrim, Policy::Guarded, 8),
            "自反"
        );
        // 修剪後的共同後繼:兩個狀態在 CommutativeTrim 下閉包相交
        let a = AState::new(vec![ev(0, 0, K::Mut, 0, 4), ev(1, 0, K::Sh, 2, 4)]);
        let b = AState::new(vec![ev(0, 0, K::Mut, 0, 2), ev(1, 0, K::Sh, 2, 4)]);
        assert!(joinable(&a, &b, Menu::CommutativeTrim, Policy::Guarded, 8));
    }

    #[test]
    fn normal_forms_singleton_when_no_rules_apply() {
        let calm = AState::new(vec![ev(0, 0, K::Sh, 0, 2), ev(1, 0, K::Sh, 1, 3)]);
        let nfs = normal_forms(&calm, Menu::CommutativeTrim, Policy::Guarded, 6);
        assert_eq!(nfs.len(), 1, "sh-sh 無衝突 ⇒ 唯一正規形(自身)");
    }

    #[test]
    fn commutative_trim_passes_newman_mechanically() {
        // 與 bin/l9newman 同參數:n=3, coord<4, depth=8
        let r = newman_check(Menu::CommutativeTrim, Policy::Guarded, 3, 4, 8);
        assert!(r.states > 0);
        assert!(r.l8_violations.is_empty(), "封閉菜單:L8 嚴格遞減");
        assert!(r.non_joinable.is_empty(), "WCR:臨界對全部可回合");
        assert!(r.multi_nf.is_empty(), "唯一正規形");
        assert!(r.conclusion.contains("機械驗證通過"));
        assert!(r.critical_pairs > 0, "演示域內應存在臨界對");
    }

    #[test]
    fn naive_menu_reports_counterexamples_honestly() {
        let r = newman_check(Menu::Naive, Policy::Guarded, 3, 4, 8);
        let broken = !r.non_joinable.is_empty() || !r.multi_nf.is_empty();
        assert!(broken, "樸素菜單必須如實暴露反例(側條件是定律的載體)");
        assert!(!r.conclusion.contains("機械驗證通過"));
    }

    #[test]
    fn report_fields_are_consistent() {
        let r = newman_check(Menu::CommutativeTrim, Policy::Guarded, 2, 3, 6);
        assert_eq!(r.menu, Menu::CommutativeTrim);
        assert_eq!(r.policy, Policy::Guarded);
        assert!(r.unique_nf_states <= r.states);
    }
}
