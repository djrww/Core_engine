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
