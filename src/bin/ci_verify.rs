//! ci_verify —— 端到端 CI 門禁與全量形式化自証執行器。
//!
//! 審計 F-01/F-07 修復:
//! * Gate 2 進度文案由 `LemmaStressEvaluator::expected_total` 派生,
//!   與實測數字同源,手寫魔術數字(60,500)已消滅;
//! * Gate 5/6 內嵌的 Rocq / Creusot 核檢三態化 —— 工具缺席 ⇒ 整項門禁
//!   如實 SKIPPED(嚴格模式非零退出),絕不偽稱 PASSED;
//! * 結論行由實跑計數渲染,嚴格模式 `CL0R0_STRICT=1` 下 SKIPPED 即阻斷。
//!
//! 運行: `cargo run --bin ci_verify`

use cl0r0::differential_checker::DifferentialAuditSuite;
use cl0r0::json_report::JsonDiagnosticPipeline;
use cl0r0::lemma_stress_generator::LemmaStressEvaluator;
use cl0r0::lemmas::LemmaRegistry;
use cl0r0::mir::{
    BasicBlockData, BorrowKind, DropElaborator, Local, MirBody, MirType, MoveAnalysisSolver,
    MoveData, Operand, Place, RegionVid, Rvalue, Statement, StatementKind, Terminator,
    TerminatorKind,
};
use cl0r0::modular_contracts::{ReborrowManager, ReborrowStatus};
use cl0r0::pipeline_synthesis::EndToEndSynthesizer;
use cl0r0::proof_resources::{AeneasTranslator, ProphecyEnvironment};
use cl0r0::rocq_export::{KernelCheck, RocqExporter};
use cl0r0::span::Span;
use cl0r0::variance_dropck_ub::{
    DropckChecker, DropckGenericConstraint, UbDiagnosticOracle, Variance, VarianceEngine,
};
use std::collections::HashMap;
use std::time::Instant;

/// 門禁結論三態(F-01:Skipped ≠ Passed)
#[derive(Clone, Copy, PartialEq, Eq)]
enum GateStatus {
    Proven,
    Skipped,
    Failed,
}

fn strict_mode() -> bool {
    std::env::args().any(|a| a == "--strict")
        || std::env::var("CL0R0_STRICT")
            .map(|v| v != "0")
            .unwrap_or(false)
}

fn main() {
    let strict = strict_mode();
    let mut skipped_gates = 0usize;
    let mut proven_gates = 0usize;
    println!("======================================================================");
    println!(" CL0 / R₀ 雙載體 · CI 全量自動化機械自証與門禁流水線 (CI Matrix Gate)");
    if strict {
        println!(" 【嚴格發布模式】:外部證明器缺席 (SKIPPED) 即視為發布阻斷");
    }
    println!("======================================================================");

    let start_all = Instant::now();
    let mut all_passed = true;

    // ------------------------------------------------------------------
    // [CI Gate 1]: 18 大形式化引理自証
    // ------------------------------------------------------------------
    print!("[CI Gate 1/7] 正在驗證 18 大形式化核心引理機器證明見證... ");
    let lemma_res = LemmaRegistry::verify_all_lemmas();
    let lemmas_ok = lemma_res.iter().all(|r| r.is_certified());
    if lemmas_ok {
        println!(
            "PASSED ({}/{} 形式化引理 100% 機器自証通過)",
            lemma_res.len(),
            lemma_res.len()
        );
        proven_gates += 1;
    } else {
        println!("FAILED");
        all_passed = false;
    }

    // ------------------------------------------------------------------
    // [CI Gate 2]: 引理海量數據極限壓力測試(F-07:文案與真值同源)
    // ------------------------------------------------------------------
    let expected_samples = LemmaStressEvaluator::expected_total(5);
    print!(
        "[CI Gate 2/7] 正在執行 {} 組引理海量數據與高熵污料壓測... ",
        expected_samples
    );
    let stress_rep = LemmaStressEvaluator::run_massive_evaluation(0xC10_2024_0001, 5);
    if stress_rep.total_passed == stress_rep.total_tested {
        println!(
            "PASSED ({}/{} 樣本 100.00% 通過 · 0 Panic)",
            stress_rep.total_passed, stress_rep.total_tested
        );
        proven_gates += 1;
    } else {
        println!("FAILED");
        all_passed = false;
    }

    // ------------------------------------------------------------------
    // [CI Gate 3]: 五大組合端到端深度合成閉環
    // ------------------------------------------------------------------
    print!("[CI Gate 3/7] 正在驗證五大組合深度合成閉環 (1+2+3+4+5)... ");
    let synth_res = EndToEndSynthesizer::execute_synthesized_loop(
        "fn main() {\n    let mut x = 1;\n    let r = &mut x;\n}",
        6,
    );
    if let Ok(rep) = synth_res {
        if rep.pipeline_converged {
            println!("PASSED (全流程 100% CONVERGED)");
            proven_gates += 1;
        } else {
            println!("FAILED");
            all_passed = false;
        }
    } else {
        println!("FAILED");
        all_passed = false;
    }

    // ------------------------------------------------------------------
    // [CI Gate 4]: 完整結構化 JSON 錯誤報告生成與修復閉環 (0 缺失率)
    // ------------------------------------------------------------------
    print!("[CI Gate 4/7] 正在校驗結構化 JSON 錯誤報告與自動修復消解... ");
    let json_repair = JsonDiagnosticPipeline::execute_json_repair_to_fixpoint(
        "fn main() {\n    let mut x = 1;\n    let r = &mut x;\n}",
    );
    if let Ok((patched_src, json_rep)) = json_repair {
        if json_rep.error_count == 0
            && json_rep.repair_applied
            && json_rep.polonius_converged
            && patched_src.contains("borrow region shortened")
        {
            println!("PASSED (修復後錯誤數嚴格歸零 · 0 Defect Rate)");
            proven_gates += 1;
        } else {
            println!("FAILED");
            all_passed = false;
        }
    } else {
        println!("FAILED");
        all_passed = false;
    }

    // ------------------------------------------------------------------
    // [CI Gate 5]: DAG 項共享、Discrimination Tree、Unification 與 Maude
    // ------------------------------------------------------------------
    print!("[CI Gate 5/7] 正在驗證 DAG 項指針共享、辨別樹索引與 Maude 引擎... ");
    let mut pool = cl0r0::dag_term::DagPool::new();
    let a = pool.constant("a");
    let ga1 = pool.app("g", vec![a]);
    let ga2 = pool.app("g", vec![a]);
    let ptr_ok = ga1 == ga2;

    let mut dt = cl0r0::discrimination_tree::DiscriminationTree::new();
    let var0 = pool.var(0);
    let pat = pool.app("g", vec![var0]);
    dt.insert(&pool, pat, "G_RULE");
    let dt_ok = !dt.query_generalizations(&pool, ga1).is_empty();

    let unif_ok = cl0r0::unification::unify(&mut pool, ga1, pat).is_some();

    let rocq_theory = RocqExporter::export_full_cl0_theory("CL0_CI_Rocq");
    // 4. Rocq 9.2 形式化理論與機械核驗(F-01:三態如實回報,缺席 ⇒ SKIPPED)
    let rocq_gate = match RocqExporter::get_rocq_binary_path() {
        Some(_) => match RocqExporter::compile_and_verify(&rocq_theory, "CL0_CI_Rocq") {
            Ok(r) if r.success && r.kernel_check == KernelCheck::Checked => GateStatus::Proven,
            Ok(r) => match r.kernel_check {
                KernelCheck::NotRun(_) => GateStatus::Skipped,
                KernelCheck::Checked => unreachable!(),
            },
            Err(_) => GateStatus::Failed,
        },
        None => GateStatus::Skipped,
    };

    let gate5_status = if !ptr_ok || !dt_ok || !unif_ok {
        GateStatus::Failed
    } else {
        rocq_gate
    };
    match gate5_status {
        GateStatus::Proven => {
            println!(
                "PASSED (DAG Pointer Sharing ∧ Discrimination Tree ∧ Unification ∧ Rocq 9.2 ✓)"
            );
            proven_gates += 1;
        }
        GateStatus::Skipped => {
            let reason = if RocqExporter::get_rocq_binary_path().is_none() {
                "rocq 不在 PATH/ROCQ_HOME"
            } else {
                "rocqchk 未執行(詳見 rocq_verify 日誌)"
            };
            println!(
                "SKIPPED (DAG ∧ DT ∧ Unification 本機已驗證;Rocq 9.2 核檢未執行: {})",
                reason
            );
            skipped_gates += 1;
        }
        GateStatus::Failed => {
            println!("FAILED");
            all_passed = false;
        }
    }

    // ------------------------------------------------------------------
    // [CI Gate 6]: MIR 靜態分析、OOPSLA 2025 模組化契約、Aeneas/Creusot & UCG/Dropck
    // ------------------------------------------------------------------
    print!("[CI Gate 6/7] 正在驗證 MIR 控制流、OOPSLA 2025 契約、Aeneas/Creusot & Dropck/UCG... ");

    // 1. MIR Move & Drop Analysis
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

    // 2. Reborrow & OOPSLA 契約
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

    // 3. Aeneas & Creusot
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

    let creusot_why3_theory =
        cl0r0::creusot_export::CreusotExporter::export_full_creusot_theory("CL0_CI_Creusot");
    // F-01:三態如實回報 —— why3/z3 缺席 ⇒ SKIPPED,絕不以「導出非空」偽稱 PASSED
    let creusot_gate = match (
        cl0r0::creusot_export::CreusotExporter::check_why3_available(),
        cl0r0::creusot_export::CreusotExporter::check_z3_available(),
    ) {
        (true, true) => {
            match cl0r0::creusot_export::CreusotExporter::verify_with_why3(
                &creusot_why3_theory,
                "CL0_CI_Creusot",
            ) {
                Ok(r) if r.success && r.valid_goals == r.total_goals => GateStatus::Proven,
                Ok(_) => GateStatus::Failed,
                Err(_) => GateStatus::Failed,
            }
        }
        (why3, z3) => {
            let missing = match (why3, z3) {
                (false, false) => "why3 與 z3 均不在 PATH",
                (false, true) => "why3 不在 PATH",
                (true, false) => "z3 不在 PATH",
                _ => unreachable!(),
            };
            println!("\n         [Why3/Z3]: SKIPPED ({})", missing);
            GateStatus::Skipped
        }
    };

    // 4. Variance & Dropck & UB
    let t_param = MirType::TypeParam("T".into());
    let ref_ty = MirType::Ref(
        RegionVid(0),
        Box::new(t_param),
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

    let gate6_core_ok = move_ok
        && drop_ok
        && reborrow_ok
        && reactivated_ok
        && aeneas_ok
        && creusot_reborrow_ok
        && creusot_resolve_ok
        && var_ok
        && dropck_ok
        && ub_ok;
    let gate6_status = if !gate6_core_ok {
        GateStatus::Failed
    } else {
        creusot_gate
    };
    match gate6_status {
        GateStatus::Proven => {
            println!(
                "PASSED (MIR Move/Drop ∧ OOPSLA 2025 ∧ Aeneas/Creusot (Why3/Z3) ∧ Dropck/UCG Oracle ✓)"
            );
            proven_gates += 1;
        }
        GateStatus::Skipped => {
            println!("SKIPPED (MIR/OOPSLA/Aeneas/Dropck 本機已驗證;Why3/Z3 SMT 核檢未執行)");
            skipped_gates += 1;
        }
        GateStatus::Failed => {
            println!("FAILED");
            all_passed = false;
        }
    }

    // ------------------------------------------------------------------
    // [CI Gate 7]: 差分審核矩陣與結構共享驗證 (Differential Audit Matrix >= 92%)
    // ------------------------------------------------------------------
    print!("[CI Gate 7/7] 正在執行全維度差分審核與持久化結構共享驗證 (>= 92%)... ");
    let diff_report = DifferentialAuditSuite::execute_full_differential_audit(50);
    if diff_report.reaches_92_percent_target && diff_report.overall_score >= 0.92 {
        println!(
            "PASSED (差分審核綜合評分: {:.2}% >= 92.00% · 全維度合規 ✓)",
            diff_report.overall_score * 100.0
        );
        proven_gates += 1;
    } else {
        println!("FAILED");
        all_passed = false;
    }

    let elapsed = start_all.elapsed();
    println!("======================================================================");
    let failed_gates = 7usize - proven_gates - skipped_gates;
    if all_passed && skipped_gates == 0 {
        println!(
            " [CI 最終結論]: 7/7 CI 門禁 Proven · 0 Skipped · 0 Failed — 100% 全部通過!總耗時: {:?}",
            elapsed
        );
        std::process::exit(0);
    } else if all_passed {
        if strict {
            eprintln!(
                " [CI 最終結論]: {}/7 Proven · {} SKIPPED · 0 FAILED — 嚴格發布模式下 SKIPPED 即發布阻斷!請在配備 Rocq/Why3/Z3 的環境復跑。總耗時: {:?}",
                proven_gates, skipped_gates, elapsed
            );
            std::process::exit(1);
        }
        println!(
            " [CI 最終結論]: {}/7 CI 門禁 Proven · {} 項 SKIPPED(外部證明器缺席,如實申報未執行)· 0 FAILED。機內門禁全部通過;發布前請以 CL0R0_STRICT=1 復跑。總耗時: {:?}",
            proven_gates, skipped_gates, elapsed
        );
        std::process::exit(0);
    } else {
        eprintln!(
            " [CI 最終結論]: {}/7 Proven · {} SKIPPED · {} FAILED — 存在失敗門禁,請檢查上述輸出!",
            proven_gates, skipped_gates, failed_gates
        );
        std::process::exit(1);
    }
}
