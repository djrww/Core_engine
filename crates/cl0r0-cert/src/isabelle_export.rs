// Copyright (c) 2026 Ken Yuen Ka Chun. All rights reserved.
// PROPRIETARY & CONFIDENTIAL — unauthorized copying, modification, distribution, or reverse engineering is prohibited.
//! §6.5 Isabelle/HOL 理論導出器 —— F-04 第二階段:sorry 顯式遺漏格式。
//!
//! 【規格聲明 — 審計 F-04 演進】
//! * 第一階段(歷史):定理陳述藏於 `(* ... *)` 註釋,不可機檢 —— 已淘汰。
//! * **第二階段(現行)**:定理陳述為**合法 Isabelle/HOL(ASCII 子集)語法**,
//!   僅含標準庫概念(`rtrancl`/`converse`/`wf`/`irrefl`),零外部依賴
//!   (不再 imports IsaFoR);每一定理以顯式 `sorry` 標記遺漏 —— Isabelle
//!   官方嘅「証明待補」佔位符,遺漏可被機器點算,唔再靠人眼搵。
//! * 証據重放權威:cl0r0 `cpf_cert::verify`(T1 機檢:三色 DFS 環檢測)。
//! * 結構良構由 `structural_audit` 機檢(定理數=sorry 數、註釋括號平衡、
//!   頭尾結構)。**本產物未經真 Isabelle 加載驗證**(CI 無 Isabelle);
//!   完整 Isabelle 証明腳本屬後續形式化(遠程),如實申報。

use crate::cpf_cert::CPFCertificate;

/// 理論結構良構審計結果(T1 機檢)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StructuralAudit {
    /// `theory <NAME>` 頭部存在且名稱一致
    pub theory_header: bool,
    /// `imports Main` 且僅依賴標準庫
    pub imports_main_only: bool,
    /// `begin` 存在且以 `end` 收尾
    pub begin_end_balanced: bool,
    /// `(* ... *)` 註釋括號平衡
    pub comments_balanced: bool,
    /// 非註釋區定理數 == 非註釋區 sorry 數(遺漏可點算)
    pub sorry_matches_theorems: bool,
    /// 無非法 `[[...]]` 屬性語法(F-04)
    pub no_attribute_syntax: bool,
}

impl StructuralAudit {
    /// 全部檢查通過
    pub fn ok(&self) -> bool {
        self.theory_header
            && self.imports_main_only
            && self.begin_end_balanced
            && self.comments_balanced
            && self.sorry_matches_theorems
            && self.no_attribute_syntax
    }
}

/// 剝掉 `(* ... *)` 註釋(支援嵌套)後嘅文本,用於點算定理與 sorry。
fn strip_comments(thy: &str) -> String {
    let mut out = String::with_capacity(thy.len());
    let mut depth = 0usize;
    let bytes: Vec<char> = thy.chars().collect();
    let mut i = 0;
    while i < bytes.len() {
        let two: String = bytes[i..(i + 2).min(bytes.len())].iter().collect();
        if two == "(*" {
            depth += 1;
            i += 2;
        } else if two == "*)" {
            depth = depth.saturating_sub(1);
            i += 2;
        } else {
            if depth == 0 {
                out.push(bytes[i]);
            }
            i += 1;
        }
    }
    out
}

/// 註釋詞法良構:無未閉合 `(*`,且代碼區(深度 0)不出現孤立 `*)`
/// —— 孤立 `*)` 喺真 Isabelle 同樣係詞法錯誤,故導出代碼必須避開。
fn comments_lexically_sound(thy: &str) -> bool {
    let mut depth = 0usize;
    let bytes: Vec<char> = thy.chars().collect();
    let mut i = 0;
    while i < bytes.len() {
        let two: String = bytes[i..(i + 2).min(bytes.len())].iter().collect();
        if two == "(*" {
            depth += 1;
            i += 2;
        } else if two == "*)" {
            if depth == 0 {
                return false; // 代碼區孤立 *)
            }
            depth -= 1;
            i += 2;
        } else {
            i += 1;
        }
    }
    depth == 0
}

/// 審計導出理論嘅結構良構性(純函數,可對任意文本執行)。
pub fn structural_audit(thy: &str, theory_name: &str) -> StructuralAudit {
    let code = strip_comments(thy);
    let theorem_count = code
        .lines()
        .filter(|l| l.trim_start().starts_with("theorem "))
        .count();
    let sorry_count = code.lines().filter(|l| l.trim() == "sorry").count();
    StructuralAudit {
        theory_header: thy.starts_with(&format!("theory {theory_name}\n")),
        imports_main_only: thy.contains("\n  imports Main\n") && !thy.contains("IsaFoR"),
        begin_end_balanced: thy.contains("\nbegin\n") && thy.ends_with("end\n"),
        comments_balanced: comments_lexically_sound(thy),
        sorry_matches_theorems: theorem_count > 0 && theorem_count == sorry_count,
        no_attribute_syntax: !thy.contains("[["),
    }
}

pub struct IsabelleExporter;

impl IsabelleExporter {
    /// 將重寫系統與合流性證明導出為 Isabelle/HOL 理論(sorry 顯式遺漏格式)。
    pub fn export_theory(theory_name: &str, cert: &CPFCertificate) -> String {
        let mut thy = String::new();

        // ---- 頭部 + 共同前奏(通用 ARS 概念,僅標準庫) ----
        thy.push_str(&format!(
            r#"theory {name}
  imports Main
begin

(* ------------------------------------------------------------------ *)
(* CL0 Dual Carrier Confluence — sorry-marked draft (F-04 stage 2).    *)
(* Theorem statements are valid Isabelle/HOL syntax over standard     *)
(* library concepts only (rtrancl/converse/wf/irrefl). Every theorem  *)
(* carries an EXPLICIT sorry placeholder: omissions are machine-      *)
(* countable, not hidden in comments. Replay authority for the       *)
(* underlying evidence: cl0r0 cpf_cert::verify (T1, three-color DFS). *)
(* Not yet loaded by real Isabelle (CI has none); full proof scripts  *)
(* are pending formalization — declared honestly.                    *)
(* ------------------------------------------------------------------ *)

section {{* Dual Carrier Signature *}}

datatype cl0_sort = S_State | S_Interval | S_Storage

datatype cl0_fun =
  F_Conf
| F_Pair
| F_Trim
| F_Split
| F_Runtime

definition cl0_signature :: "cl0_fun => nat" where
  "cl0_signature f = (case f of
     F_Conf => 3
   | F_Pair => 2
   | F_Trim => 1
   | F_Split => 1
   | F_Runtime => 2)"

section {{* Abstract Rewriting (standard library only) *}}

type_synonym 'a rel = "('a * 'a) set"

definition confluence :: "'a rel => bool" where
  "confluence R = (ALL x y z.
     (x, y) : R^* & (x, z) : R^* --> (EX u. (y, u) : R^* & (z, u) : R^* ))"

definition weakly_confluent :: "'a rel => bool" where
  "weakly_confluent R = (ALL x y z.
     (x, y) : R & (x, z) : R --> (EX u. (y, u) : R^* & (z, u) : R^* ))"

definition strongly_normalizing :: "'a rel => bool" where
  "strongly_normalizing R = wf (converse R)"

(* Newman: SN & WCR => CR — the shape our T1 engine applies to the
   certificate data below. Isabelle proof pending (sorry). *)
theorem newman_lemma_shape:
  "ALL R. strongly_normalizing R & weakly_confluent R --> confluence R"
  sorry

section {{* Proof Certificate: {system_id} *}}

"#,
            name = theory_name,
            system_id = cert.system_id,
        ));

        // ---- 分支特定:數據實錄定義 + sorry 定理 ----
        match &cert.proof_type {
            crate::cpf_cert::ProofType::KnuthBendixCriticalPairs {
                sn_witness,
                critical_pairs,
            } => {
                // 見證實錄投影:峰值對清單 + 會合邊清單(由 CriticalPairWitness 派生)
                let peaks: Vec<String> = critical_pairs
                    .iter()
                    .map(|w| format!("(\"{}\", \"{}\")", w.peak_left, w.peak_right))
                    .collect();
                let mut steps: Vec<String> = Vec::new();
                for w in critical_pairs {
                    steps.push(format!("(\"{}\", \"{}\")", w.peak_left, w.joined));
                    steps.push(format!("(\"{}\", \"{}\")", w.peak_right, w.joined));
                }
                thy.push_str(&format!(
                    r#"(* Certified via Newman Fast Path: SN witness '{sn}' + WCR.
   Peak/join edges below are MECHANICALLY RECORDED witnesses (F-03:
   derived, never declared). Replay: cpf_cert::verify. *)

definition kb_peaks :: "(string * string) list" where
  "kb_peaks = [{peaks}]"

definition kb_steps :: "string rel" where
  "kb_steps = set [{steps}]"

(* Every recorded peak converges over the recorded step relation. *)
theorem kb_peaks_joinable_witnessed:
  "ALL l r. (l, r) : set kb_peaks -->
     (EX u. (l, u) : kb_steps^* & (r, u) : kb_steps^* )"
  sorry

"#,
                    sn = sn_witness,
                    peaks = peaks.join(", "),
                    steps = steps.join(", "),
                ));
            }
            crate::cpf_cert::ProofType::DecreasingDiagrams {
                labels,
                strict_order_pairs,
            } => {
                let label_list = labels
                    .iter()
                    .map(|l| format!("\"{l}\""))
                    .collect::<Vec<_>>()
                    .join(", ");
                let pairs = strict_order_pairs
                    .iter()
                    .map(|(a, b)| format!("(\"{a}\", \"{b}\")"))
                    .collect::<Vec<_>>()
                    .join(", ");
                thy.push_str(&format!(
                    r#"(* Certified via van Oostrom Decreasing Diagrams.
   Label set and strict order pairs are recorded certificate data;
   acyclicity is machine-checked by cpf_cert::verify (three-color DFS),
   which for a finite relation entails well-foundedness (wf). *)

definition dd_labels :: "string list" where
  "dd_labels = [{label_list}]"

definition label_poset :: "(string * string) set" where
  "label_poset = set [{pairs}]"

theorem dd_label_poset_wf:
  "wf label_poset"
  sorry

theorem dd_label_poset_irrefl:
  "irrefl label_poset"
  sorry

"#,
                ));
            }
            crate::cpf_cert::ProofType::OrthogonalLeftLinear => {
                thy.push_str(
                    r#"(* Certified via Orthogonality (Rosen): left-linear and ZERO
   critical pairs — declared honestly via the dedicated orthogonal
   certificate channel. The recorded relation is peak-free. *)

definition cl0_orthogonal_trs :: "string rel" where
  "cl0_orthogonal_trs = {}"

theorem orthogonal_confluence:
  "confluence cl0_orthogonal_trs"
  sorry

"#,
                );
            }
        }

        thy.push_str("end\n");
        thy
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpf_cert::CriticalPairWitness;

    #[test]
    fn theory_draft_contains_no_illegal_attribute_syntax() {
        let cert = CPFCertificate::new_knuth_bendix(
            "CL0_KB_Theory_Test",
            "LivenessBounded",
            vec![CriticalPairWitness::new("L", "R", "J")],
        );
        let thy = IsabelleExporter::export_theory("CL0_Theory_Test", &cert);
        // F-04:`[[...]]` 是 Isabelle 定理屬性語法,不能承載 critical_pairs_count
        assert!(!thy.contains("[["));
        // 定理陳述為顯式 sorry 宣言,並標明機械核驗權威
        assert!(thy.contains("cpf_cert::verify"));
        assert!(thy.contains("theorem kb_peaks_joinable_witnessed"));
        assert!(thy.contains("datatype cl0_fun"));
        // 結構良構機檢通過
        assert!(structural_audit(&thy, "CL0_Theory_Test").ok());
    }
}

#[cfg(test)]
mod extra_tests {
    use super::*;
    use crate::cpf_cert::CriticalPairWitness;

    #[test]
    fn theory_header_imports_and_datatypes() {
        let cert = CPFCertificate::new_knuth_bendix(
            "K",
            "W",
            vec![CriticalPairWitness::new("L", "R", "J")],
        );
        let thy = IsabelleExporter::export_theory("T", &cert);
        assert!(thy.starts_with("theory T\n"));
        // F-04 第二階段:僅依賴標準庫,不再 imports IsaFoR
        assert!(thy.contains("imports Main"));
        assert!(!thy.contains("IsaFoR"));
        assert!(thy.contains("datatype cl0_sort"));
        assert!(thy.contains("begin"));
        assert!(thy.ends_with("end\n"));
        assert!(structural_audit(&thy, "T").ok());
    }

    #[test]
    fn dd_certificate_branch_rendering() {
        let cert = CPFCertificate::new_decreasing_diagrams(
            "CL0_DD_Isa",
            vec!["Trim".into(), "Split".into(), "Runtime".into()],
            vec![("Split".into(), "Trim".into())],
        );
        let thy = IsabelleExporter::export_theory("T_DD", &cert);
        assert!(
            thy.contains("Decreasing Diagrams"),
            "DD 證書應標註 van Oostrom DD"
        );
        assert!(
            thy.contains("theorem dd_label_poset_wf"),
            "DD 分支應陳述(顯式 sorry)良基定理"
        );
        assert!(
            thy.contains("theorem dd_label_poset_irrefl"),
            "DD 分支應陳述(顯式 sorry)非自反定理"
        );
        // 標籤清單與嚴格序對投影進實錄定義
        assert!(
            thy.contains("dd_labels = [\"Trim\", \"Split\", \"Runtime\"]"),
            "labels 應投影進 dd_labels: {}",
            thy
        );
        assert!(
            thy.contains("label_poset = set [(\"Split\", \"Trim\")]"),
            "嚴格序對應投影進 label_poset"
        );
        assert!(thy.starts_with("theory T_DD\n"), "理論名應出現於頭部");
        assert!(structural_audit(&thy, "T_DD").ok());
    }

    #[test]
    fn orthogonal_certificate_branch_rendering() {
        let cert = CPFCertificate {
            system_id: "O".into(),
            proof_type: crate::cpf_cert::ProofType::OrthogonalLeftLinear,
        };
        let thy = IsabelleExporter::export_theory("T_Orth", &cert);
        assert!(!thy.is_empty());
        assert!(thy.contains("begin"));
        assert!(thy.contains("theorem orthogonal_confluence"));
        assert!(thy.contains("cl0_orthogonal_trs = {}"));
        assert!(structural_audit(&thy, "T_Orth").ok());
    }

    #[test]
    fn structural_audit_catches_tampering() {
        let cert = CPFCertificate::new_knuth_bendix(
            "K",
            "W",
            vec![CriticalPairWitness::new("L", "R", "J")],
        );
        let good = IsabelleExporter::export_theory("T", &cert);
        // 多一個孤立 sorry → 定理數/sorry 數不匹配
        let extra_sorry = good.replace("end\n", "  sorry\nend\n");
        assert!(!structural_audit(&extra_sorry, "T").sorry_matches_theorems);
        // 註釋未閉合(刪走一個閉合)
        let mut unbalanced = good.clone();
        if let Some(p) = unbalanced.rfind("*)") {
            unbalanced.replace_range(p..p + 2, "");
        }
        assert!(!structural_audit(&unbalanced, "T").comments_balanced);
        // 代碼區孤立 *)(詞法錯誤)
        let stray = good.replace("end\n", "*)\nend\n");
        assert!(!structural_audit(&stray, "T").comments_balanced);
        // 破壞頭部
        let bad_head = format!("x{good}");
        assert!(!structural_audit(&bad_head, "T").theory_header);
        // 缺 end
        let no_end = good.trim_end_matches("end\n").to_string() + "\n";
        assert!(!structural_audit(&no_end, "T").begin_end_balanced);
        // 引入 IsaFoR 依賴 → imports_main_only 破產
        let with_isafor = good.replace("imports Main", "imports \"IsaFoR.X\"");
        assert!(!structural_audit(&with_isafor, "T").imports_main_only);
    }
}
