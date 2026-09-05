//! selfcheck —— verify_all / ci_verify 共用的門禁決策層(DL-004 下沉)。
//!
//! 過去兩個 bin 各自內聯十/七門禁的「檢查 + 判定 + 結論」邏輯,bin 層
//! 無法單測(llvm-cov 量測空洞)。本模組把決策層抽為可單測的單一真相:
//! * [`GateStatus`]/[`GateOutcome`] —— 三態結論(F-01:Skipped ≠ Passed);
//! * [`GateLedger`] —— 計數、結論渲染與退出碼(strict 語義:SKIPPED 即阻斷);
//! * `gate_*` —— verify_all 十門禁的檢查與判定;
//! * `ci_gate_*` —— ci_verify 七門禁的檢查與判定。
//!
//! bin 只剩:橫幅常量(F-06 口徑)+ 逐門禁調用 + 結論渲染。

use crate::cpf_cert::{CPFCertificate, CertResult};
use crate::dd_checker::{check_confluence_with_mode, CheckerMode, DDReport, SNWitness};
use crate::gen::{gen_garbage, gen_legal, Rng};
use crate::parse::parse;
use crate::rep_dd::AState;
use crate::rocq_export::{KernelCheck, RocqExporter};
use crate::tactic_scheduler::{Tactic, TacticScheduler};
use crate::testkit::fixtures;

// ===========================================================================
// 三態結論與帳本
// ===========================================================================

/// 門禁結論三態(F-01:Skipped ≠ Passed)
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GateStatus {
    Proven,
    Skipped,
    Failed,
}

/// 單一門禁的結論:狀態 + 證據註記(PASSED 括號內文案 / SKIPPED 原因)。
#[derive(Clone, Debug)]
pub struct GateOutcome {
    pub status: GateStatus,
    pub note: String,
}

impl GateOutcome {
    pub fn proven(note: impl Into<String>) -> Self {
        GateOutcome {
            status: GateStatus::Proven,
            note: note.into(),
        }
    }
    pub fn skipped(note: impl Into<String>) -> Self {
        GateOutcome {
            status: GateStatus::Skipped,
            note: note.into(),
        }
    }
    pub fn failed(note: impl Into<String>) -> Self {
        GateOutcome {
            status: GateStatus::Failed,
            note: note.into(),
        }
    }
}

/// 門禁帳本:計數 + 結論渲染 + 退出碼(結論行由實跑計數渲染,F-06)。
pub struct GateLedger {
    pub strict: bool,
    pub proven: usize,
    pub skipped: usize,
    pub failed: usize,
}

impl GateLedger {
    pub fn new(strict: bool) -> Self {
        GateLedger {
            strict,
            proven: 0,
            skipped: 0,
            failed: 0,
        }
    }

    /// 記錄一門結論並印出結論行(與舊 bin 輸出格式逐字對齊)。
    pub fn record(&mut self, o: GateOutcome) {
        match o.status {
            GateStatus::Proven => {
                println!("PASSED{}", wrap_note(&o.note));
                self.proven += 1;
            }
            GateStatus::Skipped => {
                println!("SKIPPED{}", wrap_note(&o.note));
                self.skipped += 1;
            }
            GateStatus::Failed => {
                println!("FAILED{}", wrap_note(&o.note));
                self.failed += 1;
            }
        }
    }

    pub fn total(&self) -> usize {
        self.proven + self.skipped + self.failed
    }

    /// 退出碼:任一 Failed ⇒ 1;strict 下任一 Skipped ⇒ 1;否則 0。
    pub fn exit_code(&self) -> i32 {
        if self.failed > 0 || (self.strict && self.skipped > 0) {
            1
        } else {
            0
        }
    }
}

fn wrap_note(note: &str) -> String {
    if note.is_empty() {
        String::new()
    } else {
        format!(" ({})", note)
    }
}

// ===========================================================================
// 共用見證(D-01:狀態宇宙引用 testkit 夾具單一真相)
// ===========================================================================

pub fn core_states() -> Vec<AState> {
    fixtures::sample_states()
}

pub fn core_sn_witness() -> SNWitness {
    SNWitness::LivenessScopeBounded {
        max_span_len: 4,
        storages: 1,
    }
}

// ===========================================================================
// verify_all 十門禁(檢查 + 判定)
// ===========================================================================

/// [Gate 1] L1 無損回環 / L2 決定論 / L5 laminar(gen_legal 樣本)。
pub fn gate_l1_l2_l5(samples: usize, seed: u64) -> GateOutcome {
    let mut rng = Rng::new(seed);
    for _ in 0..samples {
        let src = gen_legal(&mut rng);
        if let Ok(tree) = parse(&src) {
            if tree.unparse() != src || !tree.laminar_ok() {
                return GateOutcome::failed("存在樣本違反 L1/L2/L5");
            }
        }
    }
    GateOutcome::proven(format!("{} 樣本 100% 逐字節吻合", samples))
}

/// [Gate 2] L3/L4 增量重析等價性。
pub fn gate_reparse_equiv() -> GateOutcome {
    use crate::edit::Edit;
    let samples = vec![
        (
            "fn main() { let mut x = 1; }",
            Edit {
                start: 24,
                old_end: 25,
                text: "2".to_string(),
            },
        ),
        (
            "fn foo(a: i32) { if a > 0 { bar(a); } }",
            Edit {
                start: 20,
                old_end: 25,
                text: "b < 10".to_string(),
            },
        ),
    ];
    let rep = crate::reparse_verifier::verify_reparse_equivalence(&samples);
    if rep.failed_edits.is_empty() {
        GateOutcome::proven(format!(
            "{}/{} 全部等價",
            rep.passed_cases, rep.tested_cases
        ))
    } else {
        GateOutcome::failed(format!("{} 組增量重析不等價", rep.failed_edits.len()))
    }
}

/// [Gate 3] L8/L9 遞減圖局部峰值會合。
pub fn gate_dd_peaks(states: &[AState]) -> GateOutcome {
    let rep = check_confluence_with_mode(states, CheckerMode::DecreasingDiagrams, 6);
    if rep.certified {
        GateOutcome::proven(format!(
            "{} 狀態 / {} 峰值全部收斂於唯一正規形",
            rep.total_states, rep.total_peaks
        ))
    } else {
        GateOutcome::failed(format!("{} 峰值不可會合", rep.non_joinable_peaks.len()))
    }
}

/// [Gate 4] Newman 快速通道(SN ∧ WCR ⇒ CR)+ CPF-KB 短證。
/// 回傳 (結論, New­man 報告)——報告供 Gate 5 建 KB 證書複用。
pub fn gate_newman_fastpath(states: &[AState], sn: &SNWitness) -> (GateOutcome, DDReport) {
    let start = std::time::Instant::now();
    let rep = check_confluence_with_mode(
        states,
        CheckerMode::Newman {
            sn_witness: sn.clone(),
        },
        6,
    );
    let dur = start.elapsed();
    let o = if rep.certified && rep.cpf_kb_proof.is_some() {
        GateOutcome::proven(format!("WCR 可接合 · 出具 CPF-KB 短證 · 耗時 {:?}", dur))
    } else {
        GateOutcome::failed("Newman 快速通道未通過")
    };
    (o, rep)
}

/// [Gate 5] CPF 證書雙重核驗(DD 偏序 + KB 短證)。
pub fn gate_cpf_dual(newman_report: &DDReport, sn: &SNWitness) -> GateOutcome {
    use crate::rule_labeling::{dd_label_universe, dd_strict_order_pairs};
    let cert_dd = CPFCertificate::new_decreasing_diagrams(
        "CL0-DD",
        dd_label_universe(),
        dd_strict_order_pairs(),
    );
    let cert_kb = CPFCertificate::new_knuth_bendix(
        "CL0-KB",
        &sn.description(),
        newman_report.kb_critical_pair_witnesses.clone(),
    );
    if cert_dd.verify() == CertResult::Certified && cert_kb.verify() == CertResult::Certified {
        GateOutcome::proven(format!(
            "CERTIFIED · DD 偏序無環 ∧ KB 短證 {} 對見證全數合法",
            cert_kb.critical_pairs_count()
        ))
    } else {
        GateOutcome::failed(format!("CPF 證書被拒絕: {:?}", cert_kb.verify()))
    }
}

/// [Gate 6] 髒輸入 0 Panic + Tactic Scheduler 命中 NewmanFastPath。
pub fn gate_dirty_input_scheduler(
    samples: usize,
    seed: u64,
    states: &[AState],
    sn: &SNWitness,
) -> GateOutcome {
    let mut rng = Rng::new(seed);
    for _ in 0..samples {
        let garbage = gen_garbage(&mut rng, 40);
        if parse(&garbage).is_err() {
            return GateOutcome::failed("全化解析器對髒輸入返回 Err");
        }
    }
    let res = TacticScheduler::schedule_and_verify(states, Some(sn.clone()), 6);
    if res.selected_tactic == Tactic::NewmanFastPath {
        GateOutcome::proven(format!(
            "{} 隨機髒輸入 0 Panic ∧ 策略調度器命中 NewmanFastPath",
            samples
        ))
    } else {
        GateOutcome::failed("策略調度器未命中 NewmanFastPath")
    }
}

/// [Gate 7] 18 大形式化引理矩陣機械自証。
pub fn gate_lemma_matrix() -> GateOutcome {
    let results = crate::lemmas::LemmaRegistry::verify_all_lemmas();
    if results.iter().all(|r| r.is_certified()) {
        GateOutcome::proven(format!("{} 形式化引理全部獲得機器證明見證", results.len()))
    } else {
        GateOutcome::failed("存在未通過引理")
    }
}

/// [Gate 8] Rocq 形式化理論導出 + rocqchk 微內核核檢(三態如實)。
pub fn gate_rocq(module: &str) -> GateOutcome {
    let theory = RocqExporter::export_full_cl0_theory(module);
    match RocqExporter::get_rocq_binary_path() {
        Some(_) => match RocqExporter::compile_and_verify(&theory, module) {
            Ok(r) if r.success && r.kernel_check == KernelCheck::Checked => {
                GateOutcome::proven("Rocq 9.2 .v 導出 ∧ rocqchk 機械證明全量合規")
            }
            Ok(r) => match r.kernel_check {
                KernelCheck::NotRun(reason) => {
                    GateOutcome::skipped(format!("編譯通過但 rocqchk 未執行: {}", reason))
                }
                KernelCheck::Checked => unreachable!(),
            },
            Err(e) => GateOutcome::failed(e),
        },
        None => GateOutcome::skipped("rocq 不在 PATH/ROCQ_HOME — 本機未執行 .v 編譯與微內核核檢"),
    }
}

/// [Gate 9] Creusot / Why3 + Z3 SMT 演繹核驗(三態如實)。
pub fn gate_creusot(module: &str) -> GateOutcome {
    use crate::creusot_export::CreusotExporter;
    let theory = CreusotExporter::export_full_creusot_theory(module);
    match (
        CreusotExporter::check_why3_available(),
        CreusotExporter::check_z3_available(),
    ) {
        (true, true) => match CreusotExporter::verify_with_why3(&theory, module) {
            Ok(r) if r.success && r.valid_goals == r.total_goals => GateOutcome::proven(format!(
                "Creusot MLW 導出 ∧ Why3+Z3 SMT {}/{} Goals 100% Valid",
                r.valid_goals, r.total_goals
            )),
            Ok(r) => GateOutcome::failed(format!(
                "Why3 僅消解 {}/{} goals",
                r.valid_goals, r.total_goals
            )),
            Err(e) => GateOutcome::failed(e),
        },
        (why3, z3) => {
            let missing = match (why3, z3) {
                (false, false) => "why3 與 z3 均不在 PATH — 未執行 SMT 消解",
                (false, true) => "why3 不在 PATH — 未執行 SMT 消解",
                (true, false) => "z3 不在 PATH — 未執行 SMT 消解",
                _ => unreachable!(),
            };
            GateOutcome::skipped(missing)
        }
    }
}

/// [Gate 10] 巨集七原則 P1–P7 + 借用組合 B1–B6 + Θ(n²) 實測(純機內)。
pub fn gate_macro_seven() -> GateOutcome {
    let mut bad: Vec<String> = Vec::new();
    for r in crate::macro_lab::verify_seven_principles() {
        if !r.passed {
            bad.push(format!("{}:{}", r.id, r.evidence));
        }
    }
    for r in crate::borrow_model::verify_borrow_model() {
        if !r.passed {
            bad.push(format!("{}:{}", r.id, r.evidence));
        }
    }
    let (cx_ok, cx_ev) = crate::macro_lab::complexity_report();
    if !cx_ok {
        bad.push(format!("CX:{}", cx_ev));
    }
    if bad.is_empty() {
        GateOutcome::proven(format!("P1–P7 七門禁 + B1–B6 借用組合門禁 + {}", cx_ev))
    } else {
        GateOutcome::failed(format!("{} 項未過:{:?}", bad.len(), bad))
    }
}

// ===========================================================================
// ci_verify 七門禁(檢查 + 判定)
// ===========================================================================

/// [CI Gate 1] 18 大引理(與 verify_all Gate 7 同源,文案對齊 CI 括號)。
pub fn ci_gate_lemma_matrix() -> GateOutcome {
    let results = crate::lemmas::LemmaRegistry::verify_all_lemmas();
    if results.iter().all(|r| r.is_certified()) {
        GateOutcome::proven(format!(
            "{}/{} 形式化引理 100% 機器自証通過",
            results.len(),
            results.len()
        ))
    } else {
        GateOutcome::failed("18 大形式化引理存在未通過項")
    }
}

/// [CI Gate 2] 引理海量壓測(F-07:文案與真值同源)。
pub fn ci_gate_stress(seed: u64, level: usize) -> (usize, GateOutcome) {
    use crate::lemma_stress_generator::LemmaStressEvaluator;
    let rep = LemmaStressEvaluator::run_massive_evaluation(seed, level);
    let expected = LemmaStressEvaluator::expected_total(level);
    let o = if rep.total_passed == rep.total_tested {
        GateOutcome::proven(format!("{} 樣本 100.00% 通過 · 0 Panic", rep.total_passed))
    } else {
        GateOutcome::failed(format!(
            "{}/{} 樣本未通過",
            rep.total_tested - rep.total_passed,
            rep.total_tested
        ))
    };
    (expected, o)
}

/// [CI Gate 3] 五大組合端到端深度合成閉環。
pub fn ci_gate_synthesis() -> GateOutcome {
    use crate::pipeline_synthesis::EndToEndSynthesizer;
    match EndToEndSynthesizer::execute_synthesized_loop(
        "fn main() {\n    let mut x = 1;\n    let r = &mut x;\n}",
        6,
    ) {
        Ok(rep) if rep.pipeline_converged => GateOutcome::proven("全流程 100% CONVERGED"),
        Ok(_) => GateOutcome::failed("合成流水線未收斂"),
        Err(e) => GateOutcome::failed(e),
    }
}

/// [CI Gate 4] 結構化 JSON 錯誤報告 + 修復閉環(0 Defect Rate)。
pub fn ci_gate_json_repair() -> GateOutcome {
    use crate::json_report::JsonDiagnosticPipeline;
    match JsonDiagnosticPipeline::execute_json_repair_to_fixpoint(
        "fn main() {\n    let mut x = 1;\n    let r = &mut x;\n}",
    ) {
        Ok((patched, rep)) => {
            if rep.error_count == 0
                && rep.repair_applied
                && rep.polonius_converged
                && patched.contains("borrow region shortened")
            {
                GateOutcome::proven("修復後錯誤數嚴格歸零 · 0 Defect Rate")
            } else {
                GateOutcome::failed("JSON 修復閉環未達 0 缺陷")
            }
        }
        Err(e) => GateOutcome::failed(e),
    }
}

/// [CI Gate 5] DAG 指針共享 ∧ 辨別樹 ∧ 合一 ∧ Rocq 核檢。
pub fn ci_gate_dag_dt_unif_rocq() -> GateOutcome {
    let mut pool = crate::dag_term::DagPool::new();
    let a = pool.constant("a");
    let ga1 = pool.app("g", vec![a]);
    let ga2 = pool.app("g", vec![a]);
    let ptr_ok = ga1 == ga2;
    let mut dt = crate::discrimination_tree::DiscriminationTree::new();
    let var0 = pool.var(0);
    let pat = pool.app("g", vec![var0]);
    dt.insert(&pool, pat, "G_RULE");
    let dt_ok = !dt.query_generalizations(&pool, ga1).is_empty();
    let unif_ok = crate::unification::unify(&mut pool, ga1, pat).is_some();
    if !ptr_ok || !dt_ok || !unif_ok {
        return GateOutcome::failed("DAG/DT/合一 本機檢查未過");
    }
    match gate_rocq("CL0_CI_Rocq") {
        GateOutcome {
            status: GateStatus::Proven,
            ..
        } => GateOutcome::proven(
            "DAG Pointer Sharing ∧ Discrimination Tree ∧ Unification ∧ Rocq 9.2 ✓",
        ),
        GateOutcome {
            status: GateStatus::Skipped,
            note,
        } => GateOutcome::skipped(format!(
            "DAG ∧ DT ∧ Unification 本機已驗證;Rocq 9.2 核檢未執行: {}",
            note
        )),
        other => other,
    }
}

/// [CI Gate 6] MIR Move/Drop ∧ OOPSLA 2025 契約 ∧ Aeneas/Creusot ∧ Dropck/UCG。
pub fn ci_gate_mir_contracts() -> GateOutcome {
    use crate::mir::{
        BasicBlockData, BorrowKind, DropElaborator, Local, MirBody, MirType, MoveAnalysisSolver,
        MoveData, Operand, Place, RegionVid, Rvalue, Statement, StatementKind, Terminator,
        TerminatorKind,
    };
    use crate::modular_contracts::{ReborrowManager, ReborrowStatus};
    use crate::proof_resources::{AeneasTranslator, ProphecyEnvironment};
    use crate::span::Span;
    use crate::variance_dropck_ub::{
        DropckChecker, DropckGenericConstraint, UbDiagnosticOracle, Variance, VarianceEngine,
    };
    use std::collections::HashMap;

    // 1. MIR Move & Drop
    let mut mir_body = MirBody::new(1);
    let ret = mir_body.add_local(MirType::Int(32), true, Span::new(0, 5), Some("_0".into()));
    let arg1 = mir_body.add_local(MirType::Int(32), false, Span::new(5, 10), Some("_1".into()));
    let mut bb0 = BasicBlockData::new(Some(Terminator {
        kind: TerminatorKind::Return,
        span: Span::new(10, 15),
    }));
    bb0.statements.push(Statement {
        kind: StatementKind::Assign(
            Place::from_local(ret),
            Rvalue::Use(Operand::Copy(Place::from_local(arg1))),
        ),
        span: Span::new(6, 9),
    });
    mir_body.add_block(bb0);
    let move_data = MoveData::build(&mir_body);
    let init_states = MoveAnalysisSolver::compute_init_states(&mir_body, &move_data);
    let move_ok =
        MoveAnalysisSolver::check_use_validity(&mir_body, &move_data, &init_states).is_empty();
    let drop_ok = DropElaborator::elaborate_scope_drops(&[arg1], &mir_body)
        .drops
        .len()
        == 1;

    // 2. Reborrow & OOPSLA
    let mut rm = ReborrowManager::new();
    rm.loan_status.insert(1, ReborrowStatus::Active);
    rm.issue_reborrow(
        1,
        2,
        Place::from_local(Local(0)),
        BorrowKind::Mut {
            allow_two_phase_borrow: false,
        },
    );
    let reborrow_ok = rm.loan_status[&1] == ReborrowStatus::Suspended
        && rm.loan_status[&2] == ReborrowStatus::Active;
    rm.expire_loan(2);
    let reactivated_ok = rm.loan_status[&1] == ReborrowStatus::Active;

    // 3. Aeneas & Creusot(預言環境)
    let swap_trans = AeneasTranslator::translate_swap_example();
    let mut eval_env = HashMap::new();
    eval_env.insert("x".into(), 10);
    eval_env.insert("y".into(), 20);
    let aeneas_ok =
        AeneasTranslator::eval_expr(&swap_trans.backward_functions[0].1, &eval_env) == 20;
    let mut penv = ProphecyEnvironment::new();
    penv.register_borrow("cell", 1, 99);
    let creusot_reborrow_ok = penv.register_reborrow("cell", "child_cell").is_ok();
    let creusot_resolve_ok =
        penv.resolve_borrow("child_cell") == Some(99) && penv.cells["cell"].current_val == 99;

    // 4. Variance / Dropck / UB
    let ref_ty = MirType::Ref(
        RegionVid(0),
        Box::new(MirType::TypeParam("T".into())),
        BorrowKind::Mut {
            allow_two_phase_borrow: false,
        },
    );
    let var_ok = VarianceEngine::infer_variance_of_param(&ref_ty, "T") == Variance::Invariant;
    let dropck_ok = DropckChecker::verify_dropck_safety(
        "S",
        &[DropckGenericConstraint {
            type_param: "T".into(),
            has_may_dangle: true,
            used_in_destructor: false,
        }],
    )
    .is_ok();
    let ub_ok = UbDiagnosticOracle::check_bool_validity(2).is_some()
        && UbDiagnosticOracle::check_pointer_access(0, 4).is_some();

    let core_ok = move_ok
        && drop_ok
        && reborrow_ok
        && reactivated_ok
        && aeneas_ok
        && creusot_reborrow_ok
        && creusot_resolve_ok
        && var_ok
        && dropck_ok
        && ub_ok;
    if !core_ok {
        return GateOutcome::failed("MIR/契約/Aeneas/Dropck 本機檢查未過");
    }
    // Why3/Z3 三態(缺席 ⇒ 整項 SKIPPED)
    match gate_creusot("CL0_CI_Creusot") {
        GateOutcome {
            status: GateStatus::Proven,
            ..
        } => GateOutcome::proven(
            "MIR Move/Drop ∧ OOPSLA 2025 ∧ Aeneas/Creusot (Why3/Z3) ∧ Dropck/UCG Oracle ✓",
        ),
        GateOutcome {
            status: GateStatus::Skipped,
            ..
        } => GateOutcome::skipped("MIR/OOPSLA/Aeneas/Dropck 本機已驗證;Why3/Z3 SMT 核檢未執行"),
        other => other,
    }
}

/// [CI Gate 7] 差分審核矩陣與結構共享驗證(≥92%)。
pub fn ci_gate_differential_audit(samples: usize) -> GateOutcome {
    use crate::differential_checker::DifferentialAuditSuite;
    let rep = DifferentialAuditSuite::execute_full_differential_audit(samples);
    if rep.reaches_92_percent_target && rep.overall_score >= 0.92 {
        GateOutcome::proven(format!(
            "差分審核綜合評分: {:.2}% >= 92.00% · 全維度合規 ✓",
            rep.overall_score * 100.0
        ))
    } else {
        GateOutcome::failed(format!(
            "差分審核評分 {:.2}% 未達 92%",
            rep.overall_score * 100.0
        ))
    }
}

// ===========================================================================
// 測試(DL-004:決策層下沉後可單測)
// ===========================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ledger_counts_and_exit_codes() {
        let mut l = GateLedger::new(false);
        l.record(GateOutcome::proven("a"));
        l.record(GateOutcome::proven(""));
        assert_eq!((l.proven, l.skipped, l.failed), (2, 0, 0));
        assert_eq!(l.exit_code(), 0);

        let mut s = GateLedger::new(false);
        s.record(GateOutcome::proven(""));
        s.record(GateOutcome::skipped("工具缺席"));
        assert_eq!(s.exit_code(), 0, "非 strict:SKIPPED 不阻斷");

        let mut strict = GateLedger::new(true);
        strict.record(GateOutcome::skipped("工具缺席"));
        assert_eq!(strict.exit_code(), 1, "strict:SKIPPED 即阻斷");

        let mut f = GateLedger::new(false);
        f.record(GateOutcome::failed("x"));
        assert_eq!(f.exit_code(), 1);
        assert_eq!(f.total(), 1);
    }

    #[test]
    fn machine_gates_proven_on_core_witness() {
        assert_eq!(
            gate_l1_l2_l5(50, 0xC10_2024_0001).status,
            GateStatus::Proven
        );
        assert_eq!(gate_reparse_equiv().status, GateStatus::Proven);
        let states = core_states();
        let sn = core_sn_witness();
        assert_eq!(gate_dd_peaks(&states).status, GateStatus::Proven);
        let (o, nm) = gate_newman_fastpath(&states, &sn);
        assert_eq!(o.status, GateStatus::Proven);
        assert!(nm.certified);
        assert_eq!(gate_cpf_dual(&nm, &sn).status, GateStatus::Proven);
        assert_eq!(
            gate_dirty_input_scheduler(100, 0xD1_2024_0002, &states, &sn).status,
            GateStatus::Proven
        );
        assert_eq!(gate_lemma_matrix().status, GateStatus::Proven);
        assert_eq!(gate_macro_seven().status, GateStatus::Proven);
    }

    #[test]
    fn external_tool_gates_are_tri_state_honest() {
        // 本機無 rocq/why3 ⇒ Skipped;有 ⇒ Proven。绝不 Proven-by-absence。
        for o in [
            gate_rocq("CL0_SelfCheck_Rocq"),
            gate_creusot("CL0_SelfCheck_Creusot"),
        ] {
            assert!(matches!(o.status, GateStatus::Proven | GateStatus::Skipped));
            assert!(!o.note.is_empty(), "三態必附證據/原因");
        }
    }

    #[test]
    fn ci_machine_gates_proven() {
        assert_eq!(ci_gate_lemma_matrix().status, GateStatus::Proven);
        let (expected, o) = ci_gate_stress(0xC10_2024_0001, 1);
        assert!(expected > 0);
        assert_eq!(o.status, GateStatus::Proven);
        assert_eq!(ci_gate_synthesis().status, GateStatus::Proven);
        assert_eq!(ci_gate_json_repair().status, GateStatus::Proven);
        assert!(matches!(
            ci_gate_dag_dt_unif_rocq().status,
            GateStatus::Proven | GateStatus::Skipped
        ));
        assert!(matches!(
            ci_gate_mir_contracts().status,
            GateStatus::Proven | GateStatus::Skipped
        ));
        assert_eq!(ci_gate_differential_audit(10).status, GateStatus::Proven);
    }
}
