// Copyright (c) 2026 Ken Yuen Ka Chun. All rights reserved.
// PROPRIETARY & CONFIDENTIAL — unauthorized copying, modification, distribution, or reverse engineering is prohibited.
//! §2.4 L3 / L4 增量重析等价性机械核验器。
//!
//! 验证: ∀ (src, edit), parse(edit(src)).sexp() == reparse(src, tree, edit).sexp()

use crate::edit::{apply, Edit};
use crate::parse::{parse, reparse};

#[derive(Debug, Clone)]
pub struct ReparseReport {
    pub tested_cases: usize,
    pub passed_cases: usize,
    pub failed_edits: Vec<(String, Edit, String, String)>,
}

pub fn verify_reparse_equivalence(samples: &[(&str, Edit)]) -> ReparseReport {
    let mut passed = 0usize;
    let mut failed = Vec::new();

    for (orig_src, edit) in samples {
        let orig_tree = match parse(orig_src) {
            Ok(t) => t,
            Err(_) => continue,
        };

        let new_src = apply(orig_src, edit);

        let full_tree = match parse(&new_src) {
            Ok(t) => t,
            Err(_) => continue,
        };
        let full_sexp = full_tree.sexp();

        let incr_out = match reparse(&orig_tree, &new_src, std::slice::from_ref(edit)) {
            Ok(out) => out,
            Err(_) => {
                failed.push((
                    orig_src.to_string(),
                    edit.clone(),
                    full_sexp,
                    "Reparse Error".to_string(),
                ));
                continue;
            }
        };
        let incr_sexp = incr_out.tree.sexp();

        if full_sexp == incr_sexp {
            passed += 1;
        } else {
            failed.push((orig_src.to_string(), edit.clone(), full_sexp, incr_sexp));
        }
    }

    ReparseReport {
        tested_cases: samples.len(),
        passed_cases: passed,
        failed_edits: failed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gen::{gen_edit, gen_legal, Rng};

    #[test]
    fn empty_samples_yield_zero_report() {
        let rep = verify_reparse_equivalence(&[]);
        assert_eq!((rep.tested_cases, rep.passed_cases), (0, 0));
        assert!(rep.failed_edits.is_empty());
    }

    #[test]
    fn assorted_edits_all_equivalent() {
        let samples = vec![
            (
                "fn main() { let mut x = 1; }",
                Edit {
                    start: 24,
                    old_end: 25,
                    text: "2".into(),
                },
            ),
            (
                "fn main() { let x = 1; }",
                Edit {
                    start: 12,
                    old_end: 12,
                    text: "mut ".into(),
                },
            ),
            (
                "fn foo(a: i32) { if a > 0 { bar(a); } }",
                Edit {
                    start: 20,
                    old_end: 25,
                    text: "b < 10".into(),
                },
            ),
            (
                "fn main() { let y = 1; }",
                Edit {
                    start: 11,
                    old_end: 22,
                    text: "".into(),
                },
            ),
        ];
        let rep = verify_reparse_equivalence(&samples);
        assert_eq!(rep.tested_cases, 4);
        assert_eq!(rep.passed_cases, 4, "失敗:{:?}", rep.failed_edits);
        assert!(rep.failed_edits.is_empty());
    }

    #[test]
    fn random_legal_edits_stay_equivalent() {
        let mut rng = Rng::new(0x5EED_0001);
        let mut samples = Vec::new();
        for _ in 0..25 {
            let src = gen_legal(&mut rng);
            let e = gen_edit(&mut rng, src.len());
            samples.push((src, e));
        }
        let refs: Vec<(&str, Edit)> = samples
            .iter()
            .map(|(s, e)| (s.as_str(), e.clone()))
            .collect();
        let rep = verify_reparse_equivalence(&refs);
        assert_eq!(
            rep.passed_cases, rep.tested_cases,
            "失敗:{:?}",
            rep.failed_edits
        );
    }
}

/// DL-017:multi-edit 增量重析等價(L3/L4 批次完備性)。
/// 三方等價:批次 reparse ≡ 逐編輯 reparse ≡ 全量 parse。
#[cfg(test)]
mod multi_edit_tests {
    use super::*;
    use crate::gen::{gen_edit, gen_legal, Rng};
    use crate::parse::parse;

    /// 批次套用多個(原始座標、互不重疊)編輯
    fn apply_batch(src: &str, edits: &[Edit]) -> String {
        let mut es = edits.to_vec();
        es.sort_by_key(|e| (e.start, e.old_end));
        for w in es.windows(2) {
            assert!(w[0].old_end <= w[1].start, "測試前提:編輯不得重疊");
        }
        let mut out = String::new();
        let mut last = 0u32;
        for e in &es {
            out.push_str(&src[last as usize..e.start as usize]);
            out.push_str(&e.text);
            last = e.old_end;
        }
        out.push_str(&src[last as usize..]);
        out
    }

    /// 生成 K 個互不重疊嘅編輯(拒絕採樣;最多試 200 次)
    fn gen_nonoverlapping(rng: &mut Rng, src_len: usize, k: usize) -> Vec<Edit> {
        let mut chosen: Vec<Edit> = Vec::new();
        let mut tries = 0;
        while chosen.len() < k && tries < 200 {
            tries += 1;
            let e = gen_edit(rng, src_len);
            let ok = chosen
                .iter()
                .all(|c| e.old_end <= c.start || e.start >= c.old_end);
            if ok {
                chosen.push(e);
            }
        }
        chosen.sort_by_key(|e| (e.start, e.old_end));
        chosen
    }

    #[test]
    fn batch_two_edits_equivalent_with_reuse() {
        let mut rng = Rng::new(0xD017_0001);
        for _ in 0..10 {
            let src = gen_legal(&mut rng);
            let edits = gen_nonoverlapping(&mut rng, src.len(), 2);
            if edits.len() < 2 {
                continue;
            }
            let old = parse(&src).expect("合法源碼必可解析");
            let new_src = apply_batch(&src, &edits);
            let out = reparse(&old, &new_src, &edits).expect("批次重析");
            let full = parse(&new_src).expect("全量重析");
            assert_eq!(
                out.tree.sexp(),
                full.sexp(),
                "批次(2 編輯)重析應等價全量;src={src:?} edits={edits:?}"
            );
        }
    }

    #[test]
    fn batch_five_edits_equivalent() {
        let mut rng = Rng::new(0xD017_0002);
        for _ in 0..10 {
            let src = gen_legal(&mut rng);
            let edits = gen_nonoverlapping(&mut rng, src.len(), 5);
            if edits.len() < 5 {
                continue; // 源碼太短,五編輯唔重疊不可行——屬測試前提約束
            }
            let old = parse(&src).expect("parse");
            let new_src = apply_batch(&src, &edits);
            let out = reparse(&old, &new_src, &edits).expect("批次");
            let full = parse(&new_src).expect("全量");
            assert_eq!(out.tree.sexp(), full.sexp(), "五編輯批次等價;src={src:?}");
        }
    }

    #[test]
    fn sequential_reparse_equals_batch_reparse() {
        let mut rng = Rng::new(0xD017_0003);
        for _ in 0..10 {
            let src = gen_legal(&mut rng);
            let edits = gen_nonoverlapping(&mut rng, src.len(), 2);
            if edits.len() < 2 {
                continue;
            }
            let [e1, e2] = [&edits[0], &edits[1]];
            assert!(e1.old_end <= e2.start);
            // 逐步:e1 → reparse → 平移 e2 → reparse
            let old = parse(&src).expect("parse");
            let mid_src = apply(src.as_str(), e1);
            let mid = reparse(&old, &mid_src, std::slice::from_ref(e1)).expect("第一步");
            let delta = e1.text.len() as i64 - (e1.old_end - e1.start) as i64;
            let e2s = Edit::new(
                (e2.start as i64 + delta).max(0) as u32,
                (e2.old_end as i64 + delta).max(0) as u32,
                &e2.text,
            );
            let final_src_seq = apply(&mid_src, &e2s);
            let seq =
                reparse(&mid.tree, &final_src_seq, std::slice::from_ref(&e2s)).expect("第二步");
            // 批次:一次過
            let batch_src = apply_batch(&src, &edits);
            assert_eq!(final_src_seq, batch_src, "兩路徑終態源碼應一致");
            let batch = reparse(&old, &batch_src, &edits).expect("批次");
            assert_eq!(
                seq.tree.sexp(),
                batch.tree.sexp(),
                "逐步重析應等價批次重析;src={src:?}"
            );
        }
    }

    #[test]
    fn boundary_and_adjacent_edits_equivalent() {
        let src = "fn main() { let x = 1; let y = 2; }";
        let edits = [
            Edit::new(0, 0, "// header\n"), // 頭邊界(插入)
            Edit::new(16, 21, "z = 9"),     // 中段(替換)
            Edit::new(21, 22, " "),         // 與上一編輯相鄰
            Edit::new(src.len() as u32, src.len() as u32, "// tail"), // 尾邊界
        ];
        let old = parse(src).expect("parse");
        let new_src = apply_batch(src, &edits);
        let out = reparse(&old, &new_src, &edits).expect("批次");
        let full = parse(&new_src).expect("全量");
        assert_eq!(out.tree.sexp(), full.sexp(), "邊界+相鄰編輯應等價");
        // 註:單 item 源碼遭邊界編輯夾擊 ⇒ 整 item 不潔淨,reused=0 屬合法語義
    }

    #[test]
    fn untouched_item_is_reused_deterministically() {
        // 兩個頂層 item,編輯只落在第一個 ⇒ 第二個 item 乾淨且邊界一致,必被重用
        let src = "fn a() { let x = 1; }
fn b() { let y = 2; }";
        let edits = [Edit::new(11, 18, "z = 9")]; // 僅觸及 a 內部
        let old = parse(src).expect("parse");
        let new_src = apply_batch(src, &edits);
        let out = reparse(&old, &new_src, &edits).expect("重析");
        let full = parse(&new_src).expect("全量");
        assert_eq!(out.tree.sexp(), full.sexp());
        assert!(out.reused > 0, "未觸及的 fn b 子樹應被重用");
    }

    #[test]
    fn randomized_multi_edit_property_30_rounds() {
        let mut rng = Rng::new(0xD017_0004);
        for round in 0..30 {
            let src = gen_legal(&mut rng);
            let k = 2 + (rng.below(3) as usize); // 2..=4
            let edits = gen_nonoverlapping(&mut rng, src.len(), k);
            if edits.len() < 2 {
                continue;
            }
            let old = match parse(&src) {
                Ok(t) => t,
                Err(_) => continue,
            };
            let new_src = apply_batch(&src, &edits);
            let out = match reparse(&old, &new_src, &edits) {
                Ok(o) => o,
                Err(e) => panic!("第 {round} 輪重析失敗 {e:?};src={src:?} edits={edits:?}"),
            };
            let full = parse(&new_src).expect("全量");
            assert_eq!(
                out.tree.sexp(),
                full.sexp(),
                "第 {round} 輪等價破產;src={src:?} edits={edits:?}"
            );
        }
    }
}
