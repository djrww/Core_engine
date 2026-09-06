// Copyright (c) 2026 Ken Yuen Ka Chun. All rights reserved.
// PROPRIETARY & CONFIDENTIAL — unauthorized copying, modification, distribution, or reverse engineering is prohibited.
//! DL-016:第三方 CPF(Certification Problem Format)消費入口 —— 獨立複核者定位。
//!
//! 【規格聲明 — documented subset】
//! 本模組消費**自申報子集**:以 CPF 詞彙(certificationProblem/proof/crProof)
//! 承載三種合流証明(decreasingDiagrams / knuthBendixCriticalPairs / orthogonal),
//! 解析後交 `cpf_cert::verify()`(三色 DFS 環檢測)做**獨立複核**——
//! 唔信任來源任何自稱,只重放見證再判定。
//!
//! 誠實邊界:
//! * CeTA 完整 CPF 2.x 互操作屬後續(範圍大:TRS/復雜度/終止性等);
//! * 識別子子集:標籤/見證為簡單標識符,**不消費實體轉義**(`&` 出現即 Malformed);
//! * 未支援証明類型(如 ruleLabeling/loop)⇒ `Unsupported`,如實申報。

use crate::cpf_cert::{CPFCertificate, CertResult, CriticalPairWitness};

/// 消費判定(四態,如實申報)
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CpfImportVerdict {
    /// 解析成功且獨立複核通過
    Verified {
        system_id: String,
        proof_kind: &'static str,
    },
    /// 解析成功但獨立複核否決(如偏序含環)
    Rejected { reason: String },
    /// 証明類型不在本引擎消費子集內
    Unsupported { reason: String },
    /// XML 結構不良(缺元素/截斷/非法轉義)
    Malformed { reason: String },
}

impl std::fmt::Display for CpfImportVerdict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CpfImportVerdict::Verified {
                system_id,
                proof_kind,
            } => write!(
                f,
                "VERIFIED(獨立複核通過):系統 {system_id} · 証明類型 {proof_kind}"
            ),
            CpfImportVerdict::Rejected { reason } => {
                write!(f, "REJECTED(獨立複核否決):{reason}")
            }
            CpfImportVerdict::Unsupported { reason } => {
                write!(f, "UNSUPPORTED(不支援類型,如實申報):{reason}")
            }
            CpfImportVerdict::Malformed { reason } => {
                write!(f, "MALFORMED(結構不良):{reason}")
            }
        }
    }
}

/// 取第一個 `<tag>...</tag>` 之內文(未找到或標籤不閉合 ⇒ None)
fn inner_first(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let start = xml.find(&open)? + open.len();
    let end = xml[start..].find(&close)? + start;
    Some(xml[start..end].to_string())
}

/// 取所有 `<tag>...</tag>` 之內文清單
fn inner_all(xml: &str, tag: &str) -> Vec<String> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let mut out = Vec::new();
    let mut rest = xml;
    while let Some(s) = rest.find(&open) {
        let start = s + open.len();
        match rest[start..].find(&close) {
            Some(rel) => {
                let end = start + rel;
                out.push(rest[start..end].to_string());
                rest = &rest[end + close.len()..];
            }
            None => break,
        }
    }
    out
}

/// 識別子邊界檢查:子集不消費實體轉義
fn ident_ok(s: &str) -> bool {
    !s.is_empty() && !s.contains('&') && !s.contains('<') && !s.contains('>')
}

/// 消費並獨立複核一份 CPF(自申報子集)XML 字串。
pub fn import_and_verify(cpf_xml: &str) -> CpfImportVerdict {
    // 根元素必須存在且閉合
    if !cpf_xml.contains("<certificationProblem") || !cpf_xml.contains("</certificationProblem>") {
        return CpfImportVerdict::Malformed {
            reason: "缺 <certificationProblem> 根元素或未閉合".into(),
        };
    }
    // proof 容器(namespace 寬容:cpf:proof 或 proof)
    let proof = inner_first(cpf_xml, "cpf:proof")
        .or_else(|| inner_first(cpf_xml, "proof"))
        .ok_or(CpfImportVerdict::Malformed {
            reason: "缺 <proof> 容器".into(),
        });
    let proof = match proof {
        Ok(p) => p,
        Err(v) => return v,
    };
    // crProof 容器
    let cr = match inner_first(&proof, "crProof") {
        Some(c) => c,
        None => {
            return CpfImportVerdict::Unsupported {
                reason: "缺 <crProof>(非合流性証明,例如終止性/復雜度)".into(),
            }
        }
    };
    // 系統標識:<systemId>(缺 ⇒ 用根內 text 位置聲明;本子集要求提供)
    let system_id = match inner_first(cpf_xml, "systemId") {
        Some(s) if ident_ok(&s) => s,
        Some(_) => {
            return CpfImportVerdict::Malformed {
                reason: "<systemId> 含非法字符(子集不消費轉義)".into(),
            }
        }
        None => "unnamed_third_party".to_string(),
    };

    // 分支一:decreasingDiagrams
    if let Some(dd) = inner_first(&cr, "decreasingDiagrams") {
        let labels: Vec<String> = inner_all(&dd, "name");
        if labels.is_empty() {
            return CpfImportVerdict::Malformed {
                reason: "decreasingDiagrams 缺 <name> 標籤".into(),
            };
        }
        if labels.iter().any(|l| !ident_ok(l)) {
            return CpfImportVerdict::Malformed {
                reason: "標籤含非法字符(子集不消費轉義)".into(),
            };
        }
        let mut pairs = Vec::new();
        for p in inner_all(&dd, "pair") {
            let a = inner_first(&p, "arg");
            let b = inner_all(&p, "arg");
            match (a, b.get(1)) {
                (Some(a), Some(b)) if ident_ok(&a) && ident_ok(b) => {
                    pairs.push((a.clone(), b.clone()));
                }
                _ => {
                    return CpfImportVerdict::Malformed {
                        reason: "strict <pair> 需恰好兩個合法 <arg>".into(),
                    }
                }
            }
        }
        let cert = CPFCertificate::new_decreasing_diagrams(&system_id, labels, pairs);
        return finish(cert, "decreasingDiagrams");
    }

    // 分支二:knuthBendixCriticalPairs
    if let Some(kb) = inner_first(&cr, "knuthBendixCriticalPairs") {
        let sn = match inner_first(&kb, "snWitness") {
            Some(s) if ident_ok(&s) => s,
            Some(_) => {
                return CpfImportVerdict::Malformed {
                    reason: "<snWitness> 含非法字符".into(),
                }
            }
            None => {
                return CpfImportVerdict::Malformed {
                    reason: "knuthBendixCriticalPairs 缺 <snWitness>".into(),
                }
            }
        };
        let mut cps = Vec::new();
        for c in inner_all(&kb, "criticalPair") {
            let l = inner_first(&c, "peakLeft");
            let r = inner_first(&c, "peakRight");
            let j = inner_first(&c, "joined");
            match (l, r, j) {
                (Some(l), Some(r), Some(j)) if ident_ok(&l) && ident_ok(&r) && ident_ok(&j) => {
                    cps.push(CriticalPairWitness::new(&l, &r, &j));
                }
                _ => {
                    return CpfImportVerdict::Malformed {
                        reason: "<criticalPair> 需 peakLeft/peakRight/joined 全齐且合法".into(),
                    }
                }
            }
        }
        if cps.is_empty() {
            return CpfImportVerdict::Malformed {
                reason: "knuthBendixCriticalPairs 缺 <criticalPair> 見證(F-03:拒絕無見證申報)"
                    .into(),
            };
        }
        let cert = CPFCertificate::new_knuth_bendix(&system_id, &sn, cps);
        return finish(cert, "knuthBendixCriticalPairs");
    }

    // 分支三:orthogonal
    if cr.contains("<orthogonal") {
        let cert = CPFCertificate::new_orthogonal(&system_id);
        return finish(cert, "orthogonal");
    }

    // 其他 crProof 類型 ⇒ 如實 Unsupported
    CpfImportVerdict::Unsupported {
        reason: "crProof 類型不在消費子集(decreasingDiagrams/knuthBendixCriticalPairs/orthogonal)"
            .into(),
    }
}

fn finish(cert: CPFCertificate, kind: &'static str) -> CpfImportVerdict {
    match cert.verify() {
        CertResult::Certified => CpfImportVerdict::Verified {
            system_id: cert.system_id.clone(),
            proof_kind: kind,
        },
        CertResult::Rejected(reason) => CpfImportVerdict::Rejected { reason },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const THIRD_PARTY_DD: &str = r#"<certificationProblem>
  <systemId>cofesco_2026_entry_17</systemId>
  <input><!-- 第三方 TRS,消費方不信任內容 --></input>
  <cpf:proof>
    <crProof>
      <decreasingDiagrams>
        <labels><name>Trim</name><name>Split</name><name>Runtime</name></labels>
        <strict>
          <pair><arg>Split</arg><arg>Trim</arg></pair>
        </strict>
      </decreasingDiagrams>
    </crProof>
  </cpf:proof>
</certificationProblem>"#;

    #[test]
    fn third_party_dd_certificate_verified() {
        let v = import_and_verify(THIRD_PARTY_DD);
        assert_eq!(
            v,
            CpfImportVerdict::Verified {
                system_id: "cofesco_2026_entry_17".into(),
                proof_kind: "decreasingDiagrams",
            }
        );
    }

    #[test]
    fn cyclic_strict_order_rejected_by_independent_check() {
        let cyclic = r#"<certificationProblem>
  <cpf:proof><crProof>
    <decreasingDiagrams>
      <labels><name>A</name><name>B</name></labels>
      <strict>
        <pair><arg>A</arg><arg>B</arg></pair>
        <pair><arg>B</arg><arg>A</arg></pair>
      </strict>
    </decreasingDiagrams>
  </crProof></cpf:proof>
</certificationProblem>"#;
        match import_and_verify(cyclic) {
            CpfImportVerdict::Rejected { .. } => {}
            v => panic!("環偏序應被獨立複核否決,實得:{v:?}"),
        }
    }

    #[test]
    fn unknown_proof_type_is_unsupported() {
        let unknown = r#"<certificationProblem>
  <cpf:proof><crProof><ruleLabeling><rule/>What</ruleLabeling></crProof></cpf:proof>
</certificationProblem>"#;
        assert!(matches!(
            import_and_verify(unknown),
            CpfImportVerdict::Unsupported { .. }
        ));
        // 非合流証明(終止性)同樣 Unsupported
        let term = r#"<certificationProblem>
  <cpf:proof><dpProof><depGraphProc/></dpProof></cpf:proof>
</certificationProblem>"#;
        assert!(matches!(
            import_and_verify(term),
            CpfImportVerdict::Unsupported { .. }
        ));
    }

    #[test]
    fn malformed_structures_are_rejected() {
        // 截斷
        let truncated = r#"<certificationProblem><cpf:proof><crProof>"#;
        assert!(matches!(
            import_and_verify(truncated),
            CpfImportVerdict::Malformed { .. }
        ));
        // 非法轉義字符(子集邊界)
        let escaped = THIRD_PARTY_DD.replace("Trim", "Tri&amp;m");
        assert!(matches!(
            import_and_verify(&escaped),
            CpfImportVerdict::Malformed { .. }
        ));
        // KB 缺見證(F-03)
        let kb_no_witness = r#"<certificationProblem>
  <cpf:proof><crProof><knuthBendixCriticalPairs><snWitness>W</snWitness></knuthBendixCriticalPairs></crProof></cpf:proof>
</certificationProblem>"#;
        assert!(matches!(
            import_and_verify(kb_no_witness),
            CpfImportVerdict::Malformed { .. }
        ));
    }

    #[test]
    fn kb_and_orthogonal_samples_verified() {
        let kb = r#"<certificationProblem>
  <systemId>kb_entry_42</systemId>
  <cpf:proof><crProof><knuthBendixCriticalPairs>
    <snWitness>LivenessBounded</snWitness>
    <criticalPair><peakLeft>L</peakLeft><peakRight>R</peakRight><joined>J</joined></criticalPair>
  </knuthBendixCriticalPairs></crProof></cpf:proof>
</certificationProblem>"#;
        assert_eq!(
            import_and_verify(kb),
            CpfImportVerdict::Verified {
                system_id: "kb_entry_42".into(),
                proof_kind: "knuthBendixCriticalPairs",
            }
        );
        let orth = r#"<certificationProblem>
  <systemId>orth_entry_7</systemId>
  <cpf:proof><crProof><orthogonal/></crProof></cpf:proof>
</certificationProblem>"#;
        assert_eq!(
            import_and_verify(orth),
            CpfImportVerdict::Verified {
                system_id: "orth_entry_7".into(),
                proof_kind: "orthogonal",
            }
        );
    }
}
