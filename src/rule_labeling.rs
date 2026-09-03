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
                    let deg = in_degree.get_mut(&v).unwrap();
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
