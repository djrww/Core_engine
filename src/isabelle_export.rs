//! §6.5 Isabelle/HOL 風格形式化理論草稿導出器(Isabelle-flavored Theory Exporter)。
//!
//! 【規格聲明 — 審計 F-04 止損】本導出器生成的是 Isabelle/HOL **風格的理論
//! 草稿**:`datatype` / `definition` 部分為合法 Isabelle 語法,但合流性定理
//! 的完整 Isabelle 證明腳本**尚未形式化**,故定理陳述以註釋形式記錄,
//! 並標明其機械核驗由 cl0r0 CPF 核驗器(`cpf_cert::verify`)執行。
//! **本產物不可被 Isabelle 2024 / IsaFoR 直接加載並通過。**

use crate::cpf_cert::CPFCertificate;

pub struct IsabelleExporter;

impl IsabelleExporter {
    /// 將重寫系統與合流性證明導出為 Isabelle/HOL 風格 (.thy) 理論草稿
    pub fn export_theory(theory_name: &str, cert: &CPFCertificate) -> String {
        let mut thy = String::new();

        thy.push_str(&format!(
            "theory {}\n\
             imports \"IsaFoR.Decreasing_Diagrams\" \"IsaFoR.Knuth_Bendix_Orders\"\n\
             begin\n\n\
             section \\<open>CL0 Dual Carrier Confluence Formalization (draft)\\<close>\n\n\
             (* -------------------------------------------------------------- *)\n\
             (* NOTE: This is an Isabelle-flavored DRAFT theory. The theorem  *)\n\
             (* statements below are machine-checked by the cl0r0 CPF          *)\n\
             (* verifier (cpf_cert::verify), NOT by Isabelle. Complete         *)\n\
             (* Isabelle proof scripts are pending formalization.             *)\n\
             (* -------------------------------------------------------------- *)\n\n\
             datatype cl0_sort = S_State | S_Interval | S_Storage\n\n\
             datatype cl0_fun =\n\
               F_Conf\n\
             | F_Pair\n\
             | F_Trim\n\
             | F_Split\n\
             | F_Runtime\n\n\
             definition cl0_signature :: \"cl0_fun \\<Rightarrow> nat\" where\n\
               \"cl0_signature f = (case f of\n\
                  F_Conf \\<Rightarrow> 3\n\
                | F_Pair \\<Rightarrow> 2\n\
                | F_Trim \\<Rightarrow> 1\n\
                | F_Split \\<Rightarrow> 1\n\
                | F_Runtime \\<Rightarrow> 2)\"\n\n\
             section \\<open>Proof Certificate\\<close>\n\n",
            theory_name
        ));

        match &cert.proof_type {
            crate::cpf_cert::ProofType::KnuthBendixCriticalPairs {
                sn_witness,
                critical_pairs,
            } => {
                thy.push_str(&format!(
                    "(* Certified via Newman Fast Path: SN Witness + WCR *)\n\
                     (* SN witness: {} *)\n\
                     (* Mechanically recorded joinability witnesses: {} critical pairs, *)\n\
                     (* each (peak_left, peak_right) converging to a common reduct.   *)\n\
                     (* Replay authority: cl0r0 cpf_cert::verify — Newman's lemma     *)\n\
                     (* (SN \\<and> WCR \\<Rightarrow> CR) is applied over the replayed witnesses.   *)\n\
                     (*\n\
                     theorem cl0_confluence:\n\
                       shows \"CR (cl0_rewrite_relation)\"\n\
                       using cl0_sn_witness cl0_critical_pairs_joinable\n\
                       by (rule newman_confluence)  (* full Isabelle proof pending *)\n\
                     *)\n\n",
                    sn_witness,
                    critical_pairs.len()
                ));
            }
            crate::cpf_cert::ProofType::DecreasingDiagrams {
                labels,
                strict_order_pairs,
            } => {
                thy.push_str(&format!(
                    "(* Certified via van Oostrom Decreasing Diagrams *)\n\
                     definition label_poset :: \"(string \\<times> string) set\" where\n\
                       \"label_poset = {:?}\"\n\n\
                     (* Well-foundedness of label_poset and the confluence theorem   *)\n\
                     (* below are machine-checked by cl0r0 cpf_cert::verify         *)\n\
                     (* (three-color DFS cycle detection). Full Isabelle proofs     *)\n\
                     (* are pending formalization.                                   *)\n\
                     (*\n\
                     lemma wf_label_poset: \"wfP_on ({:?}) label_poset\"\n\
                       by (auto simp: label_poset_def)\n\
                     theorem cl0_dd_confluence:\n\
                       shows \"CR (cl0_labeled_rewrite_system)\"\n\
                       using wf_label_poset\n\
                       by (rule decreasing_diagrams_confluence)\n\
                     *)\n\n",
                    strict_order_pairs, labels
                ));
            }
            crate::cpf_cert::ProofType::OrthogonalLeftLinear => {
                thy.push_str(
                    "(* Certified via Orthogonality (Rosen): the system is left-linear *)\n\
                     (* and has no critical pairs; confluence holds definitionally.    *)\n\
                     (*\n\
                     theorem cl0_orthogonal_confluence:\n\
                       shows \"CR (cl0_orthogonal_trs)\"\n\
                       by (rule rosen_orthogonality_confluence)\n\
                     *)\n\n",
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
        // 定理陳述以註釋記錄,並標明機械核驗權威
        assert!(thy.contains("cpf_cert::verify"));
        assert!(thy.contains("theorem cl0_confluence"));
        assert!(thy.contains("datatype cl0_fun"));
    }
}
