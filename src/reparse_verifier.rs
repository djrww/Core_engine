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
