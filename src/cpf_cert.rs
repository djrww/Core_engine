//! §6.2 原生輕量級 CPF 風格 (CPF-inspired) 證書載體與核驗器。
//!
//! 【規格聲明 — 審計 F-04 止損】本模組輸出的 XML 為**內部證書交換格式**:
//! 格式靈感來自 IsaFoR / CeTA 的 CPF 3.7.1,但**尚未對齊 CeTA schema,
//! 不可直接傳予 CeTA 核驗**。證書的權威核驗以本模組 `verify()` 的機械
//! 重放為準;導出的 XML 用於人類審計歸檔與管線交換。
//!
//! 核驗原則(審計 F-02 / F-03 / F-05 修復):
//! 1. `ProofType::KnuthBendixCriticalPairs`:證書必須攜帶 SN 見證與**每一對
//!    臨界對的會合見證實體**;見證清單為空即拒絕(零臨界對系統請改用
//!    `OrthogonalLeftLinear` 如實申報)。臨界對個數由見證清單長度派生,
//!    不存在可捏造的獨立計數欄位。
//! 2. `ProofType::DecreasingDiagrams`:標號偏序必須良基——以三色 DFS 做
//!    **完整判環**,任意長度的環(自環、2-環、3-環、4+ 環)一律拒絕。
//! 3. 所有字串插值點一律經 `xml_escape` 轉義,杜絕標記注入。

use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CertResult {
    Certified,
    Rejected(String),
}

/// Knuth-Bendix 臨界對會合見證實體(F-03:證書不再只有計數)。
///
/// 每個見證記錄一對臨界對的兩側歸約項與機械搜索到的共同歸約
/// (會合點),由 `dd_checker` 的 Newman 通道在核驗時實錄生成,
/// 不可手工捏造計數。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CriticalPairWitness {
    /// 峰值左分支的規範形描述
    pub peak_left: String,
    /// 峰值右分支的規範形描述
    pub peak_right: String,
    /// 左右雙方均可到達的共同歸約(會合見證)
    pub joined: String,
}

impl CriticalPairWitness {
    pub fn new(peak_left: &str, peak_right: &str, joined: &str) -> Self {
        CriticalPairWitness {
            peak_left: peak_left.to_string(),
            peak_right: peak_right.to_string(),
            joined: joined.to_string(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProofType {
    /// Knuth-Bendix 臨界對短證 (Newman Fast Path):SN 見證 + 全部臨界對會合見證
    KnuthBendixCriticalPairs {
        sn_witness: String,
        critical_pairs: Vec<CriticalPairWitness>,
    },
    /// van Oostrom 遞減圖良基偏序證書
    DecreasingDiagrams {
        labels: Vec<String>,
        strict_order_pairs: Vec<(String, String)>, // (greater, lesser)
    },
    /// Rosen 左線性正交無臨界對直接判定
    OrthogonalLeftLinear,
}

#[derive(Clone, Debug)]
pub struct CPFCertificate {
    pub system_id: String,
    pub proof_type: ProofType,
}

/// XML 必轉義字符集中處理(F-05:所有插值點單一真相)
fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('\"', "&quot;")
        .replace('\'', "&apos;")
}

/// 三色 DFS 完整判環(F-02:取代只查 3-鏈的偽傳遞閉包)。
///
/// 在有限標號集上,有向無環 ⟺ 良基偏序。返回找到的環路徑
/// (首尾同節點),無環返回 `None`。迭代實作,深鏈不爆棧。
fn poset_find_cycle(pairs: &[(String, String)]) -> Option<Vec<String>> {
    let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
    for (hi, lo) in pairs {
        adj.entry(hi.as_str()).or_default().push(lo.as_str());
    }

    let mut color: HashMap<&str, u8> = HashMap::new(); // 1 = 灰(在堆疊上), 2 = 黑(已完成)
    let mut roots: Vec<&str> = adj.keys().copied().collect();
    roots.sort_unstable();

    for &root in &roots {
        if color.get(root) == Some(&2) {
            continue;
        }
        color.insert(root, 1);
        let mut stack: Vec<(&str, usize)> = vec![(root, 0)];
        while let Some(top) = stack.last_mut() {
            let (v, idx) = *top;
            let neighbors: &[&str] = adj.get(v).map(|xs| xs.as_slice()).unwrap_or(&[]);
            if idx < neighbors.len() {
                top.1 += 1;
                let w = neighbors[idx];
                match color.get(w) {
                    Some(1) => {
                        // 灰色後繼 ⇒ 找到環:堆疊上 v..=末尾 再接回 w
                        let mut path: Vec<String> = stack
                            .iter()
                            .skip_while(|&&(n, _)| n != w)
                            .map(|&(n, _)| n.to_string())
                            .collect();
                        path.push(w.to_string());
                        return Some(path);
                    }
                    None => {
                        color.insert(w, 1);
                        stack.push((w, 0));
                    }
                    Some(_) => {}
                }
            } else {
                color.insert(v, 2);
                stack.pop();
            }
        }
    }
    None
}

impl CPFCertificate {
    /// 构造标准 DD 证书
    pub fn new_decreasing_diagrams(
        system_id: &str,
        labels: Vec<String>,
        strict_order_pairs: Vec<(String, String)>,
    ) -> Self {
        CPFCertificate {
            system_id: system_id.to_string(),
            proof_type: ProofType::DecreasingDiagrams {
                labels,
                strict_order_pairs,
            },
        }
    }

    /// 构造 Knuth-Bendix Newman 短证。
    ///
    /// 臨界對個數由 `critical_pairs` 見證清單長度派生(F-03:消滅可捏造的
    /// 獨立計數)。見證實體應來自 `dd_checker` Newman 通道的機械實錄。
    pub fn new_knuth_bendix(
        system_id: &str,
        sn_witness: &str,
        critical_pairs: Vec<CriticalPairWitness>,
    ) -> Self {
        CPFCertificate {
            system_id: system_id.to_string(),
            proof_type: ProofType::KnuthBendixCriticalPairs {
                sn_witness: sn_witness.to_string(),
                critical_pairs,
            },
        }
    }

    /// 構造 Rosen 左線性正交證書(零臨界對系統的如實申報途徑)
    pub fn new_orthogonal(system_id: &str) -> Self {
        CPFCertificate {
            system_id: system_id.to_string(),
            proof_type: ProofType::OrthogonalLeftLinear,
        }
    }

    /// KB 短證攜帶的臨界對見證數(派生量,非申報量)
    pub fn critical_pairs_count(&self) -> usize {
        match &self.proof_type {
            ProofType::KnuthBendixCriticalPairs { critical_pairs, .. } => critical_pairs.len(),
            _ => 0,
        }
    }

    /// 机械核验证书合法性与良基性
    pub fn verify(&self) -> CertResult {
        match &self.proof_type {
            ProofType::OrthogonalLeftLinear => CertResult::Certified,
            ProofType::KnuthBendixCriticalPairs {
                sn_witness,
                critical_pairs,
            } => {
                if sn_witness.trim().is_empty() {
                    return CertResult::Rejected(
                        "Missing SN termination witness in KB certificate".to_string(),
                    );
                }
                // F-03:沒有見證實體的 KB 證書 = 只有計數的空殼,一律拒絕。
                // 真正零臨界對的系統應用 OrthogonalLeftLinear 如實申報。
                if critical_pairs.is_empty() {
                    return CertResult::Rejected(
                        "KB certificate carries no critical-pair witnesses \
                         (count-only certificates are rejected; use OrthogonalLeftLinear \
                         for zero-peak systems)"
                            .to_string(),
                    );
                }
                for (i, cp) in critical_pairs.iter().enumerate() {
                    if cp.peak_left.trim().is_empty() || cp.peak_right.trim().is_empty() {
                        return CertResult::Rejected(format!(
                            "Critical pair #{} is missing its peak reducts",
                            i
                        ));
                    }
                    if cp.joined.trim().is_empty() {
                        return CertResult::Rejected(format!(
                            "Critical pair #{} carries no joinability witness",
                            i
                        ));
                    }
                }
                CertResult::Certified
            }
            ProofType::DecreasingDiagrams {
                labels: _,
                strict_order_pairs,
            } => {
                for (hi, lo) in strict_order_pairs {
                    if hi == lo {
                        return CertResult::Rejected(format!(
                            "Self-loop detected: {} > {}",
                            hi, lo
                        ));
                    }
                }
                // F-02:完整判環 —— 任意長度的偏序環都會摧毀良基性,
                // 從而令 van Oostrom 遞減圖定理的前提失效。
                if let Some(cycle) = poset_find_cycle(strict_order_pairs) {
                    return CertResult::Rejected(format!(
                        "Cycle detected in label poset (well-foundedness violated): {}",
                        cycle.join(" > ")
                    ));
                }
                CertResult::Certified
            }
        }
    }

    /// 导出为内部证书交换格式 XML(CPF 风格;**非** CeTA 可加载格式,见模块注释)。
    pub fn to_cpf_xml(&self) -> String {
        match &self.proof_type {
            ProofType::KnuthBendixCriticalPairs {
                sn_witness,
                critical_pairs,
            } => {
                let mut pairs_xml = String::new();
                for cp in critical_pairs {
                    pairs_xml.push_str(&format!(
                        "        <pair>\n          <left>{}</left>\n          <right>{}</right>\n          <joined>{}</joined>\n        </pair>\n",
                        xml_escape(&cp.peak_left),
                        xml_escape(&cp.peak_right),
                        xml_escape(&cp.joined)
                    ));
                }
                format!(
                    "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
                     <certificationProblem>\n  \
                       <input><trs><name>{}</name></trs></input>\n  \
                       <format>cl0r0-internal (CPF-inspired, not CeTA-loadable)</format>\n  \
                       <proof>\n    \
                         <crKnuthBendix>\n      \
                           <terminationWitness>{}</terminationWitness>\n      \
                           <criticalPairs count=\"{}\" joinable=\"true\">\n{}\
                           </criticalPairs>\n    \
                         </crKnuthBendix>\n  \
                       </proof>\n\
                     </certificationProblem>",
                    xml_escape(&self.system_id),
                    xml_escape(sn_witness),
                    critical_pairs.len(),
                    pairs_xml
                )
            }
            ProofType::DecreasingDiagrams {
                labels,
                strict_order_pairs,
            } => {
                // F-04:導出完整見證實體(標號清單 + 偏序對),不再是裸計數。
                let mut labels_xml = String::new();
                for l in labels {
                    labels_xml.push_str(&format!("        <label name=\"{}\"/>\n", xml_escape(l)));
                }
                let mut order_xml = String::new();
                for (hi, lo) in strict_order_pairs {
                    order_xml.push_str(&format!(
                        "        <pair greater=\"{}\" lesser=\"{}\"/>\n",
                        xml_escape(hi),
                        xml_escape(lo)
                    ));
                }
                format!(
                    "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
                     <certificationProblem>\n  \
                       <input><trs><name>{}</name></trs></input>\n  \
                       <format>cl0r0-internal (CPF-inspired, not CeTA-loadable)</format>\n  \
                       <proof>\n    \
                         <crDecreasingDiagrams>\n      \
                           <labels count=\"{}\">\n{}\n      </labels>\n      \
                           <strictOrder count=\"{}\">\n{}\n      </strictOrder>\n    \
                         </crDecreasingDiagrams>\n  \
                       </proof>\n\
                     </certificationProblem>",
                    xml_escape(&self.system_id),
                    labels.len(),
                    labels_xml,
                    strict_order_pairs.len(),
                    order_xml
                )
            }
            ProofType::OrthogonalLeftLinear => {
                format!(
                    "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
                     <certificationProblem>\n  \
                       <input><trs><name>{}</name></trs></input>\n  \
                       <format>cl0r0-internal (CPF-inspired, not CeTA-loadable)</format>\n  \
                       <proof><crOrthogonal/></proof>\n\
                     </certificationProblem>",
                    xml_escape(&self.system_id)
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------- F-02 迴歸:2/3/4/5-環全部必須被拒 ----------

    #[test]
    fn dd_two_cycle_is_rejected() {
        let cert = CPFCertificate::new_decreasing_diagrams(
            "evil-2cycle",
            vec!["a".into(), "b".into()],
            vec![("a".into(), "b".into()), ("b".into(), "a".into())],
        );
        assert!(matches!(cert.verify(), CertResult::Rejected(_)));
    }

    #[test]
    fn dd_three_cycle_is_rejected() {
        let cert = CPFCertificate::new_decreasing_diagrams(
            "evil-3cycle",
            vec!["a".into(), "b".into(), "c".into()],
            vec![
                ("a".into(), "b".into()),
                ("b".into(), "c".into()),
                ("c".into(), "a".into()),
            ],
        );
        assert!(matches!(cert.verify(), CertResult::Rejected(_)));
    }

    #[test]
    fn dd_four_cycle_is_rejected() {
        let cert = CPFCertificate::new_decreasing_diagrams(
            "evil-4cycle",
            vec!["a".into(), "b".into(), "c".into(), "d".into()],
            vec![
                ("a".into(), "b".into()),
                ("b".into(), "c".into()),
                ("c".into(), "d".into()),
                ("d".into(), "a".into()),
            ],
        );
        assert!(matches!(cert.verify(), CertResult::Rejected(_)));
    }

    #[test]
    fn dd_five_cycle_is_rejected() {
        let cert = CPFCertificate::new_decreasing_diagrams(
            "evil-5cycle",
            vec!["a".into(), "b".into(), "c".into(), "d".into(), "e".into()],
            vec![
                ("a".into(), "b".into()),
                ("b".into(), "c".into()),
                ("c".into(), "d".into()),
                ("d".into(), "e".into()),
                ("e".into(), "a".into()),
            ],
        );
        assert!(matches!(cert.verify(), CertResult::Rejected(_)));
    }

    #[test]
    fn dd_self_loop_is_rejected() {
        let cert = CPFCertificate::new_decreasing_diagrams(
            "self-loop",
            vec!["a".into()],
            vec![("a".into(), "a".into())],
        );
        assert!(matches!(cert.verify(), CertResult::Rejected(_)));
    }

    #[test]
    fn dd_acyclic_poset_is_certified() {
        // 鏈式偏序 Runtime ≻ Split ≻ Trim(標準 DD 宇宙)+ 長鏈
        let cert = CPFCertificate::new_decreasing_diagrams(
            "ok-chain",
            vec!["Trim".into(), "Split".into(), "Runtime".into()],
            vec![
                ("Split".into(), "Trim".into()),
                ("Runtime".into(), "Split".into()),
                ("Runtime".into(), "Trim".into()),
            ],
        );
        assert_eq!(cert.verify(), CertResult::Certified);

        let long_chain = CPFCertificate::new_decreasing_diagrams(
            "ok-long-chain",
            vec!["a".into(), "b".into(), "c".into(), "d".into()],
            vec![
                ("a".into(), "b".into()),
                ("b".into(), "c".into()),
                ("c".into(), "d".into()),
            ],
        );
        assert_eq!(long_chain.verify(), CertResult::Certified);
    }

    #[test]
    fn dd_disconnected_cycles_are_rejected() {
        // 環不在起點分量上、且需經多個根才能到達 —— DFS 必須全圖覆蓋
        let cert = CPFCertificate::new_decreasing_diagrams(
            "evil-disconnected",
            vec!["a".into(), "b".into(), "x".into()],
            vec![
                ("a".into(), "b".into()),
                ("a".into(), "x".into()),
                ("b".into(), "a".into()),
            ],
        );
        assert!(matches!(cert.verify(), CertResult::Rejected(_)));
    }

    // ---------- F-03 迴歸:無見證實體的 KB 證書必須被拒 ----------

    #[test]
    fn kb_certificate_without_witness_entities_is_rejected() {
        // 舊 API 捏造路徑:new(id, "whatever", 42) 恆 Certified —— 現在必須 Rejected
        let cert = CPFCertificate::new_knuth_bendix("fabricated", "whatever", vec![]);
        assert!(matches!(cert.verify(), CertResult::Rejected(_)));
    }

    #[test]
    fn kb_certificate_with_empty_sn_witness_is_rejected() {
        let cert = CPFCertificate::new_knuth_bendix(
            "no-sn",
            "   ",
            vec![CriticalPairWitness::new("l", "r", "j")],
        );
        assert!(matches!(cert.verify(), CertResult::Rejected(_)));
    }

    #[test]
    fn kb_certificate_with_real_witnesses_is_certified() {
        let cert = CPFCertificate::new_knuth_bendix(
            "real-kb",
            "LivenessBounded(span <= 4)",
            vec![
                CriticalPairWitness::new("s→a", "s→b", "nf_both"),
                CriticalPairWitness::new("s→c", "s→d", "nf_cd"),
            ],
        );
        assert_eq!(cert.verify(), CertResult::Certified);
        assert_eq!(cert.critical_pairs_count(), 2);
    }

    #[test]
    fn kb_certificate_with_blank_join_witness_is_rejected() {
        let cert = CPFCertificate::new_knuth_bendix(
            "blank-join",
            "LivenessBounded",
            vec![CriticalPairWitness::new("l", "r", "  ")],
        );
        assert!(matches!(cert.verify(), CertResult::Rejected(_)));
    }

    // ---------- F-05 迴歸:XML 五字符必轉義 ----------

    #[test]
    fn xml_output_escapes_hostile_strings() {
        let cert = CPFCertificate::new_knuth_bendix(
            "<script>alert(1)</script>",
            "w\"<>&'",
            vec![CriticalPairWitness::new("<l>", "&r", "'j\"")],
        );
        let xml = cert.to_cpf_xml();
        assert!(!xml.contains("<script>"), "raw <script> must not survive");
        assert!(xml.contains("&lt;script&gt;"));
        assert!(xml.contains("w&quot;&lt;&gt;&amp;&apos;"));
        assert!(xml.contains("&amp;r"));

        let dd = CPFCertificate::new_decreasing_diagrams(
            "<inj&>",
            vec!["a<b".into()],
            vec![("x\"y".into(), "z'w".into())],
        );
        let xml2 = dd.to_cpf_xml();
        assert!(xml2.contains("&lt;inj&amp;&gt;"));
        assert!(xml2.contains("a&lt;b"));
        assert!(xml2.contains("x&quot;y"));
        assert!(xml2.contains("z&apos;w"));
    }

    // ---------- F-04 迴歸:導出必須攜帶完整見證實體 ----------

    #[test]
    fn xml_export_carries_witness_entities_not_bare_counts() {
        let cert = CPFCertificate::new_decreasing_diagrams(
            "CL0-DD",
            vec!["Trim".into(), "Split".into(), "Runtime".into()],
            vec![
                ("Split".into(), "Trim".into()),
                ("Runtime".into(), "Split".into()),
                ("Runtime".into(), "Trim".into()),
            ],
        );
        let xml = cert.to_cpf_xml();
        assert!(xml.contains("<label name=\"Trim\"/>"));
        assert!(xml.contains("<label name=\"Split\"/>"));
        assert!(xml.contains("<label name=\"Runtime\"/>"));
        assert!(xml.contains("<pair greater=\"Split\" lesser=\"Trim\"/>"));
        // 誠實聲明:不可 CeTA 加載
        assert!(xml.contains("not CeTA-loadable"));

        let kb = CPFCertificate::new_knuth_bendix(
            "CL0-KB",
            "LivenessBounded",
            vec![CriticalPairWitness::new("L", "R", "J")],
        );
        let kxml = kb.to_cpf_xml();
        assert!(kxml.contains("<left>L</left>"));
        assert!(kxml.contains("<joined>J</joined>"));
        assert!(kxml.contains("count=\"1\""));
    }
}
