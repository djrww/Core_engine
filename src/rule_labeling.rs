//! §4.9 van Oostrom 規則標號啟發式求解器 (Rule Labeling Heuristic Solver for Decreasing Diagrams)。
//!
//! 參考文獻:
//!   - Vincent van Oostrom (2008), "Confluence by Decreasing Diagrams"
//!   - Nao Hirokawa & Aart Middeldorp (2008), "Automating Decreasing Diagrams by Rule Labeling"
//!
//! 核心算法:
//!   1. 給定一組重寫規則標號與枚舉出的全部局部峰值 (Local Peaks);
//!   2. 構造標號偏序約束有向圖 (Poset Constraint DAG);
//!   3. 通過拓撲排序 (Topological Sorting) 自動求解良基偏序 $(I, \succ)$；
//!   4. 輸出通過認證的 CPF-DD 證書與標號偏序關係。

use crate::cpf_cert::CPFCertificate;
use crate::rep_dd::Label;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// 遞減圖標號宇宙的單一真相(審計 D-02:五處手抄 vec!["Trim","Split","Runtime"] 收編)。
///
/// 標號良基序是全系統的公理級常量;字串散落多處時,輸入錯一處(如
/// "Runnime")並非所有消費方都會報錯。所有需要該宇宙的代碼一律引用此處。
pub const DD_LABEL_UNIVERSE: &[&str] = &["Trim", "Split", "Runtime"];

/// 標號嚴格偏序(Runtime ≻ Split ≻ Trim 及其傳遞邊)—— 與
/// `rep_dd::Label::rank`(Trim=1, Split=2, Runtime=3)的良基階數一致。
pub const DD_STRICT_ORDER_PAIRS: &[(&str, &str)] =
    &[("Split", "Trim"), ("Runtime", "Split"), ("Runtime", "Trim")];

/// 標號宇宙(字串形式;供 CPF-DD 證書構造)
pub fn dd_label_universe() -> Vec<String> {
    DD_LABEL_UNIVERSE.iter().map(|s| s.to_string()).collect()
}

/// 標號嚴格偏序對(字串形式;供 CPF-DD 證書構造)
pub fn dd_strict_order_pairs() -> Vec<(String, String)> {
    DD_STRICT_ORDER_PAIRS
        .iter()
        .map(|(a, b)| (a.to_string(), b.to_string()))
        .collect()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrecedenceConstraint {
    pub greater: Label,
    pub lesser: Label,
}

#[derive(Clone, Debug)]
pub struct RuleLabelingResult {
    pub is_solvable: bool,
    pub topological_order: Vec<Label>,
    pub precedence_pairs: Vec<(String, String)>,
    pub certificate: Option<CPFCertificate>,
}

pub struct RuleLabelingSolver;

impl RuleLabelingSolver {
    /// 自動從局部峰值與後繼路徑約束中求解良基偏序
    pub fn solve_rule_labeling(
        labels: &[Label],
        constraints: &[PrecedenceConstraint],
    ) -> RuleLabelingResult {
        let mut adj: BTreeMap<Label, BTreeSet<Label>> = BTreeMap::new();
        let mut in_degree: BTreeMap<Label, usize> = BTreeMap::new();

        for &lbl in labels {
            adj.entry(lbl).or_default();
            in_degree.entry(lbl).or_insert(0);
        }

        for c in constraints {
            if c.greater == c.lesser {
                // 自環，不可滿足
                return RuleLabelingResult {
                    is_solvable: false,
                    topological_order: Vec::new(),
                    precedence_pairs: Vec::new(),
                    certificate: None,
                };
            }
            if adj.entry(c.greater).or_default().insert(c.lesser) {
                *in_degree.entry(c.lesser).or_insert(0) += 1;
            }
        }

        // Kahn 算法求拓撲排序 (檢測是否有環)
        let mut queue = VecDeque::new();
        for (&lbl, &deg) in &in_degree {
            if deg == 0 {
                queue.push_back(lbl);
            }
        }

        let mut topo_order = Vec::new();
        while let Some(u) = queue.pop_front() {
            topo_order.push(u);
            if let Some(neighbors) = adj.get(&u) {
                for &v in neighbors {
                    let deg = in_degree
                        .get_mut(&v)
                        .expect("不變式:v 已預先插入 in_degree");
                    *deg -= 1;
                    if *deg == 0 {
                        queue.push_back(v);
                    }
                }
            }
        }

        let is_solvable = topo_order.len() == labels.len();

        let mut precedence_pairs = Vec::new();
        for (u, vs) in &adj {
            for v in vs {
                precedence_pairs.push((format!("{:?}", u), format!("{:?}", v)));
            }
        }

        let certificate = if is_solvable {
            let label_strings = labels.iter().map(|l| format!("{:?}", l)).collect();
            Some(CPFCertificate::new_decreasing_diagrams(
                "CL0-RuleLabeling-AutoCertified",
                label_strings,
                precedence_pairs.clone(),
            ))
        } else {
            None
        };

        RuleLabelingResult {
            is_solvable,
            topological_order: topo_order,
            precedence_pairs,
            certificate,
        }
    }
}

#[cfg(test)]
mod dd_universe_tests {
    use super::*;

    /// D-02 迴歸:標號宇宙與偏序必須自洽(偏序中的每個標號都在宇宙內,
    /// 且宇宙恰為 Trim/Split/Runtime 三元)。
    #[test]
    fn dd_label_universe_is_the_single_source_of_truth() {
        assert_eq!(dd_label_universe(), vec!["Trim", "Split", "Runtime"]);
        let pairs = dd_strict_order_pairs();
        assert_eq!(pairs.len(), 3);
        for (hi, lo) in &pairs {
            assert!(DD_LABEL_UNIVERSE.contains(&hi.as_str()));
            assert!(DD_LABEL_UNIVERSE.contains(&lo.as_str()));
            assert_ne!(hi, lo);
        }
        // 與 rep_dd::Label 的良基階數(Trim=1, Split=2, Runtime=3)一致:
        // Runtime ≻ Split ≻ Trim
        assert!(pairs.contains(&("Runtime".to_string(), "Split".to_string())));
        assert!(pairs.contains(&("Split".to_string(), "Trim".to_string())));
        // 由該宇宙構造的證書必須通過 F-02 修復後的完整判環
        let cert = CPFCertificate::new_decreasing_diagrams(
            "DD-Universe-SingleTruth",
            dd_label_universe(),
            dd_strict_order_pairs(),
        );
        assert_eq!(cert.verify(), crate::cpf_cert::CertResult::Certified);
    }
}
