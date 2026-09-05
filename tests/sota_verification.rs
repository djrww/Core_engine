//! tests/sota_verification.rs
//!
//! 综合集成测试套件：覆盖 Decreasing Diagrams、Newman 快速通道 (SN ∧ WCR ⇒ CR)、
//! 五大组合深度合成流水线 (1+2+3+4+5)、CPF-KB 短证、Tactic Scheduler、
//! 证书与污料生产机 (CertGeneratorFactory)、Polonius 桥接、
//! MIR 控制流圖與 Move/Drop 分析、OOPSLA 2025 模組化契約、Aeneas/Creusot 證明資源與 UCG/Dropck 預言機、
//! 持久化結構共享 AST 與全量差分審核矩陣 (>= 92% 指標)。

use cl0r0::ari_export::export_menu_to_ari;
use cl0r0::cert_generator_factory::CertGeneratorFactory;
use cl0r0::cpf_cert::{CPFCertificate, CertResult};
use cl0r0::dd_checker::{check_confluence_with_mode, CheckerMode, SNWitness};
use cl0r0::diff_tree::{DiffAstNode, DiffAstVersionChain, DiffNodeType};
use cl0r0::differential_checker::DifferentialAuditSuite;
use cl0r0::edit::Edit;
use cl0r0::lsp_bridge::LspEngine;
use cl0r0::mir::{
    BasicBlockData, BorrowKind, DropElaborator, Local, MirBody, MirType, MoveAnalysisSolver,
    MoveData, Operand, Place, RegionVid, Rvalue, Statement, StatementKind, Terminator,
    TerminatorKind,
};
use cl0r0::modular_contracts::{
    IdiomaticPatternLibrary, LoopFixpointSolver, ReborrowManager, ReborrowStatus,
};
use cl0r0::parse::parse;
use cl0r0::patch_engine::PatchEngine;
use cl0r0::pipeline_synthesis::EndToEndSynthesizer;
use cl0r0::polonius_bridge::PoloniusBridge;
use cl0r0::proof_resources::{AeneasTranslator, Permission, ProphecyEnvironment};
use cl0r0::r0_lower::R0Lowerer;
use cl0r0::rep_dd::AState;
use cl0r0::reparse_verifier::verify_reparse_equivalence;
use cl0r0::rustc_json::RustcJsonAutomaton;
use cl0r0::shrink::shrink_source;
use cl0r0::span::Span;
use cl0r0::tactic_scheduler::{Tactic, TacticScheduler};
use cl0r0::variance_dropck_ub::{
    DropckChecker, DropckGenericConstraint, UbDiagnosticOracle, Variance, VarianceEngine,
};
use std::collections::HashMap;
use std::sync::Arc;

fn sample_states() -> Vec<AState> {
    // D-01:委派 cl0r0::testkit 單一真相夾具(與 verify_all / coco_benchmark 同源)
    cl0r0::testkit::fixtures::sample_state_universe(4)
}

#[test]
fn test_cert_and_dirty_generator_factory() {
    let artifacts = CertGeneratorFactory::run_factory_production(20, 0xC10_2024_0001);
    assert_eq!(artifacts.dirty_samples_count, 20);
    assert!(!artifacts.generated_cpf_certificates.is_empty());
    assert_eq!(artifacts.generated_ari_problems.len(), 20);
    assert!(!artifacts.generated_rocq_theories.is_empty());
    assert_eq!(artifacts.polonius_facts_catalog.len(), 20);
    assert!(artifacts.total_certified_rate > 90.0);

    let mut rng = cl0r0::gen::Rng::new(0xCAFE);
    let dirty_universe = CertGeneratorFactory::produce_dirty_universe(&mut rng, 30);
    let (passed, failed) = CertGeneratorFactory::verify_dirty_robustness(&dirty_universe);
    assert_eq!(failed, 0, "全化解析器對所有污料樣本應 0 Panic 構造樹");
    assert_eq!(passed, 30);
}

#[test]
fn test_rocq_9_2_formal_export_and_mechanical_check() {
    let full_theory =
        cl0r0::rocq_export::RocqExporter::export_full_cl0_theory("CL0_Integration_Rocq");
    assert!(full_theory.contains("Inductive sort"));
    assert!(full_theory.contains("Theorem lemma5_euler_characteristic_tree"));
    assert!(full_theory.contains("Theorem star_trans"));

    if cl0r0::rocq_export::RocqExporter::check_rocq_available() {
        let res = cl0r0::rocq_export::RocqExporter::compile_and_verify(
            &full_theory,
            "CL0_Integration_Rocq",
        );
        assert!(
            res.is_ok(),
            "Rocq 9.2 should mechanically verify full theory: {:?}",
            res
        );
        let rep = res.unwrap();
        assert!(rep.success);
        assert!(rep.vo_bytes > 0);
        assert_eq!(
            rep.kernel_check,
            cl0r0::rocq_export::KernelCheck::Checked,
            "rocqchk microkernel verification must succeed"
        );
    }
}

#[test]
fn test_five_stage_end_to_end_synthesis() {
    let sample = "fn main() {\n    let mut x = 1;\n    let r = &mut x;\n}";
    let report = EndToEndSynthesizer::execute_synthesized_loop(sample, 6)
        .expect("五大组合深度合成流水线应执行成功");
    assert!(report.stage1_ast_tree_ok);
    assert!(report.stage2_tactic_result.report.certified);
    assert!(!report.stage3_polonius_facts.is_empty());
    assert!(report.stage4_reparse_l3_l4_ok);
    assert!(report.stage5_json_diagnostic_resolved);
    assert!(report.pipeline_converged);

    let batch_ok = EndToEndSynthesizer::fuzz_and_synthesize_batch(50, 0xC10_2024_0001);
    assert!(batch_ok > 0);
}

#[test]
fn test_decreasing_diagrams_confluence() {
    let states = sample_states();
    let report = check_confluence_with_mode(&states, CheckerMode::DecreasingDiagrams, 6);
    assert!(
        report.certified,
        "Decreasing Diagrams 验证应 100% 通过无不可会合峰值"
    );
    assert_eq!(report.non_joinable_peaks.len(), 0);
    assert!(report.total_peaks > 0);
}

#[test]
fn test_newman_fast_path_and_cpf_kb() {
    let states = sample_states();
    let witness = SNWitness::LivenessScopeBounded {
        max_span_len: 4,
        storages: 1,
    };
    let report = check_confluence_with_mode(
        &states,
        CheckerMode::Newman {
            sn_witness: witness,
        },
        6,
    );
    assert!(report.certified, "Newman 快速通道应 100% 验证 WCR 可接合性");
    assert!(report.cpf_kb_proof.is_some());
    let proof_xml = report.cpf_kb_proof.unwrap();
    assert!(proof_xml.contains("knuth-bendix"));
}

#[test]
fn test_tactic_scheduler_portfolio() {
    let states = sample_states();
    let witness = SNWitness::PolynomialOrder {
        degree: 1,
        coeffs: vec![1, 0],
    };

    let res = TacticScheduler::schedule_and_verify(&states, Some(witness), 6);
    assert_eq!(res.selected_tactic, Tactic::NewmanFastPath);
    assert!(res.report.certified);
    assert_eq!(res.certificate.verify(), CertResult::Certified);

    let cops_problem = TacticScheduler::export_cops_problem(42, states.len());
    assert!(cops_problem.contains("COPS Problem #0042"));
}

#[test]
fn test_cpf_certificate_valid_and_invalid() {
    // F-03:KB 證書必須攜帶臨界對會合見證實體(由 Newman 通道機械實錄)
    let states = sample_states();
    let kb_report = check_confluence_with_mode(
        &states,
        CheckerMode::Newman {
            sn_witness: SNWitness::LivenessScopeBounded {
                max_span_len: 4,
                storages: 1,
            },
        },
        6,
    );
    assert!(kb_report.certified && !kb_report.kb_critical_pair_witnesses.is_empty());
    let cert_kb = CPFCertificate::new_knuth_bendix(
        "CL0-KB",
        "LivenessBounded",
        kb_report.kb_critical_pair_witnesses.clone(),
    );
    assert_eq!(cert_kb.verify(), CertResult::Certified);
    assert!(cert_kb.to_cpf_xml().contains("<crKnuthBendix>"));

    // F-03 迴歸:只有計數、沒有見證實體的空殼證書必須被拒
    let fabricated = CPFCertificate::new_knuth_bendix("CL0-KB-Fake", "whatever", vec![]);
    assert!(matches!(fabricated.verify(), CertResult::Rejected(_)));

    // F-02 迴歸:2-環與 4-環偏序必須被拒(舊代碼只查 3-鏈,全部漏放)
    let two_cycle = CPFCertificate::new_decreasing_diagrams(
        "evil-2cycle",
        vec!["a".to_string(), "b".to_string()],
        vec![
            ("a".to_string(), "b".to_string()),
            ("b".to_string(), "a".to_string()),
        ],
    );
    assert!(matches!(two_cycle.verify(), CertResult::Rejected(_)));
    let four_cycle = CPFCertificate::new_decreasing_diagrams(
        "evil-4cycle",
        vec![
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        ],
        vec![
            ("a".to_string(), "b".to_string()),
            ("b".to_string(), "c".to_string()),
            ("c".to_string(), "d".to_string()),
            ("d".to_string(), "a".to_string()),
        ],
    );
    assert!(matches!(four_cycle.verify(), CertResult::Rejected(_)));

    let cert_valid = CPFCertificate::new_decreasing_diagrams(
        "CL0-DD-Certified",
        vec![
            "Trim".to_string(),
            "Split".to_string(),
            "Runtime".to_string(),
        ],
        vec![
            ("Split".to_string(), "Trim".to_string()),
            ("Runtime".to_string(), "Split".to_string()),
            ("Runtime".to_string(), "Trim".to_string()),
        ],
    );
    assert_eq!(cert_valid.verify(), CertResult::Certified);

    let cert_self_loop = CPFCertificate::new_decreasing_diagrams(
        "Invalid-SelfLoop",
        vec!["Trim".to_string()],
        vec![("Trim".to_string(), "Trim".to_string())],
    );
    assert!(matches!(cert_self_loop.verify(), CertResult::Rejected(_)));

    // F-05 迴歸:敵意字串必須全部轉義,不得破壞 XML 結構
    let hostile = CPFCertificate::new_decreasing_diagrams(
        "<script>alert(1)</script>",
        vec!["a".to_string()],
        vec![("x&y".to_string(), "a".to_string())],
    );
    let hx = hostile.to_cpf_xml();
    assert!(!hx.contains("<script>"));
    assert!(hx.contains("&lt;script&gt;"));
    assert!(hx.contains("x&amp;y"));
}

#[test]
fn test_lsp_bridge_with_newman_dd_explanation() {
    let src = "fn main() { let mut x = 1; let r = &mut x; }";
    let (diags, actions) = LspEngine::analyze_and_suggest_actions(src);
    assert!(!diags.is_empty());
    assert!(!actions.is_empty());
    assert!(diags[0].proof_explanation.contains("Newman Fast Path"));
}

#[test]
fn test_reparse_equivalence_l3_l4() {
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
    let report = verify_reparse_equivalence(&samples);
    assert_eq!(report.failed_edits.len(), 0);
    assert_eq!(report.passed_cases, 2);
}

#[test]
fn test_r0_lowering_and_unsupported_boundary() {
    let valid_src = "fn main() { let mut x = 1; let r = &x; }";
    let tree = parse(valid_src).unwrap();
    let facts = R0Lowerer::lower(valid_src, &tree).unwrap();
    assert!(facts.total_storages >= 1);
    assert!(!facts.events.is_empty());

    let invalid_src = "fn main() { match x { 0 => {} } }";
    let inv_tree = parse(invalid_src).unwrap();
    let err = R0Lowerer::lower(invalid_src, &inv_tree);
    assert!(err.is_err(), "越界构造必须被如实拦截");
}

#[test]
fn test_ari_export_format() {
    let ari = export_menu_to_ari();
    assert!(ari.contains("(format trs)"));
    assert!(ari.contains("(fun conf 3)"));
    assert!(ari.contains("(rule (pair (conf S A B)"));
}

#[test]
fn test_polonius_bridge_roundtrip() {
    let state = cl0r0::testkit::fixtures::two_event_state(1, 3, 2, 4);

    let d_facts = PoloniusBridge::export_to_polonius_facts(&state);
    assert!(d_facts.contains("loan_issued_at('orig_0, 0, 1)."));
    assert!(d_facts.contains("invalidates(2, 0)."));

    let imported = PoloniusBridge::import_from_polonius_facts(&d_facts);
    assert_eq!(imported.evs.len(), 2);
}

#[test]
fn test_span_monad_and_patch_engine() {
    let src = "fn main() { let mut x = 1; let r = &mut x; }";
    let tree = parse(src).unwrap();
    let (patched_src, patched_tree) = PatchEngine::apply_shorten_repair(src, &tree, 27).unwrap();
    assert!(patched_src.contains("// [cl0r0 auto-drop]: borrow region shortened"));
    assert!(patched_tree.unparse() == patched_src);
}

#[test]
fn test_rustc_json_parser() {
    let mock_json = r#"{"message":"cannot borrow `x` as mutable more than once at a time","code":{"code":"E0499","explanation":null},"level":"error","spans":[{"file_name":"src/main.rs","byte_start":14,"byte_end":25,"line_start":2,"line_end":2,"column_start":13,"column_end":24,"is_primary":true,"text":[],"label":"first mutable borrow occurs here","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error[E0499]: ..."}"#;
    let diag = RustcJsonAutomaton::parse_single_json_line(mock_json).unwrap();
    assert_eq!(diag.code.as_deref(), Some("E0499"));
    assert_eq!(diag.level, "error");
    assert_eq!(diag.spans.len(), 1);
    assert_eq!(diag.spans[0].byte_start, 14);
    assert_eq!(diag.spans[0].byte_end, 25);
}

#[test]
fn test_delta_shrinker() {
    let large_src = "fn main() { let x = 1; /* noise noise noise */ ERROR_PATTERN; let y = 2; }";
    let shrunk = shrink_source(large_src.to_string(), |s| s.contains("ERROR_PATTERN"));
    assert!(shrunk.contains("ERROR_PATTERN"));
    assert!(shrunk.len() < large_src.len(), "Shrinker 应该精简无用噪声");
}

#[test]
fn test_formal_lemmas_mechanical_registry() {
    let results = cl0r0::lemmas::LemmaRegistry::verify_all_lemmas();
    assert_eq!(results.len(), 18, "Must verify all 18 formal lemmas");
    for res in results {
        assert!(res.is_certified(), "Lemma must be certified: {:?}", res);
    }
}

#[test]
fn test_polonius_datalog_fixpoint_solver() {
    let state = cl0r0::testkit::fixtures::two_event_state(1, 4, 2, 5);

    let db = PoloniusBridge::extract_database(&state);
    assert!(!db.loan_issued_at.is_empty());
    assert!(!db.cfg_edges.is_empty());

    let analysis = PoloniusBridge::solve_datalog_fixpoint(&db);
    assert!(analysis.fixpoint_iterations >= 1);
    assert!(!analysis.loan_live_points.is_empty());
}

#[test]
fn test_isabelle_theory_exporter() {
    // F-04:證書攜帶真實見證;.thy 為草稿格式,不得含非法 [[...]] 語法
    let cert = CPFCertificate::new_knuth_bendix(
        "CL0_Confluence_Cert",
        "LivenessBounded",
        vec![cl0r0::cpf_cert::CriticalPairWitness::new("L", "R", "J")],
    );
    let thy = cl0r0::isabelle_export::IsabelleExporter::export_theory("CL0_Theory", &cert);
    assert!(thy.contains("theory CL0_Theory"));
    assert!(thy.contains("IsaFoR.Decreasing_Diagrams"));
    assert!(thy.contains("datatype cl0_fun"));
    // 定理陳述以註釋記錄並標明機械核驗權威(Isabelle 完整證明待形式化)
    assert!(thy.contains("theorem cl0_confluence"));
    assert!(thy.contains("cpf_cert::verify"));
    assert!(
        !thy.contains("[["),
        "illegal Isabelle attribute syntax must be gone"
    );
}

#[test]
fn test_lsp_json_rpc_processor() {
    let init_req = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
    let init_resp = LspEngine::process_json_rpc(init_req).unwrap();
    assert!(init_resp.contains("\"capabilities\""));

    let hover_req = r#"{"jsonrpc":"2.0","id":2,"method":"textDocument/hover","params":{}}"#;
    let hover_resp = LspEngine::process_json_rpc(hover_req).unwrap();
    assert!(hover_resp.contains("Decreasing Diagrams"));

    let inlay_req = r#"{"jsonrpc":"2.0","id":3,"method":"textDocument/inlayHint","params":{}}"#;
    let inlay_resp = LspEngine::process_json_rpc(inlay_req).unwrap();
    assert!(inlay_resp.contains("[✓ DD Confluent]"));
}

#[test]
fn test_rule_labeling_heuristic_solver() {
    use cl0r0::rep_dd::Label;
    use cl0r0::rule_labeling::{PrecedenceConstraint, RuleLabelingSolver};

    let labels = vec![Label::Trim(0), Label::Split(0), Label::Runtime(0, 1)];
    let constraints = vec![
        PrecedenceConstraint {
            greater: Label::Runtime(0, 1),
            lesser: Label::Split(0),
        },
        PrecedenceConstraint {
            greater: Label::Split(0),
            lesser: Label::Trim(0),
        },
    ];

    let result = RuleLabelingSolver::solve_rule_labeling(&labels, &constraints);
    assert!(result.is_solvable, "Rule labeling DAG should be solvable");
    assert_eq!(result.topological_order.len(), 3);
    assert!(result.certificate.is_some());
    let cert = result.certificate.unwrap();
    assert_eq!(cert.verify(), CertResult::Certified);
}

#[test]
fn test_polonius_repair_loop() {
    use cl0r0::polonius_bridge::PoloniusRepairLoop;

    let sample_src = "fn main() {\n    let mut x = 1;\n    let r = &mut x;\n}";
    let report = PoloniusRepairLoop::analyze_and_repair(sample_src)
        .expect("Polonius repair loop should succeed");
    assert!(report.converged);
    assert_eq!(report.final_errors_count, 0);
}

#[test]
fn test_dag_terms_and_discrimination_tree_pipeline() {
    use cl0r0::dag_term::DagPool;
    use cl0r0::discrimination_tree::DiscriminationTree;

    let mut pool = DagPool::new();
    let a = pool.constant("a");
    let x = pool.var(0);

    // 構造共享項: f(g(a), g(a))
    let ga1 = pool.app("g", vec![a]);
    let ga2 = pool.app("g", vec![a]);
    assert_eq!(ga1, ga2); // Pointer sharing via Hash-Consing
    let f_term = pool.app("f", vec![ga1, ga2]);

    let mut dt = DiscriminationTree::new();
    let pat = pool.app("f", vec![x, ga1]);
    dt.insert(&pool, pat, 42usize);

    let query_res = dt.query_generalizations(&pool, f_term);
    assert_eq!(query_res, vec![42usize]);
}

#[test]
fn test_unification_and_coq_tactics() {
    use cl0r0::dag_term::DagPool;
    use cl0r0::tactics::{CongruenceClosure, LiaSolver, LinearInequality};
    use cl0r0::unification::unify;

    let mut pool = DagPool::new();
    let x = pool.var(0);
    let y = pool.var(1);
    let a = pool.constant("a");
    let b = pool.constant("b");

    // Unify f(x, b) with f(a, y) -> x=a, y=b
    let t1 = pool.app("f", vec![x, b]);
    let t2 = pool.app("f", vec![a, y]);
    let mgu = unify(&mut pool, t1, t2).unwrap();
    assert_eq!(mgu.get(0), Some(a));
    assert_eq!(mgu.get(1), Some(b));

    // Congruence Closure: a = b => h(a) = h(b)
    let ha = pool.app("h", vec![a]);
    let hb = pool.app("h", vec![b]);
    let cc_proof = CongruenceClosure::prove_goal(&pool, &[(a, b)], (ha, hb));
    assert!(cc_proof.proven);

    // LIA Tactic: bounds checking
    let ineqs = vec![
        LinearInequality::diff(0, 1, 10),
        LinearInequality::diff(1, 2, 5),
    ];
    let lia_proof = LiaSolver::check_satisfiability(&ineqs);
    assert!(lia_proof.is_satisfiable);
}

#[test]
fn test_maude_rewriting_logic_engine_model_checking() {
    use cl0r0::dag_term::DagPool;
    use cl0r0::maude_engine::MaudeModule;

    let mut pool = DagPool::new();
    let mut maude = MaudeModule::new("CL0-SYSTEM", vec!["State", "Token"]);

    let s0 = pool.constant("s0");
    let s1 = pool.constant("s1");
    let s2 = pool.constant("s2");

    // rl [t1] : s0 => s1 .
    // rl [t2] : s1 => s2 .
    maude.add_rule(&pool, "t1", s0, s1);
    maude.add_rule(&pool, "t2", s1, s2);

    // Search reachability of s2 from s0
    let reachable = maude.search(&mut pool, s0, s2, 5);
    assert_eq!(reachable, vec![s2]);

    let export = maude.export_maude(&pool);
    assert!(export.contains("mod CL0-SYSTEM is"));
    assert!(export.contains("rl [t1] : s0 => s1 ."));
}

#[test]
fn test_ast_tree_facts_and_edit_monoid() {
    use cl0r0::ast::{conflicts, extract, EvKind};
    use cl0r0::edit::{apply_all, compose, compose_seq, is_pairwise_disjoint, Edit};
    use cl0r0::parse::parse;
    use cl0r0::span::Span;
    use cl0r0::span_monad::{SpanAnchor, SpanMonad};

    let src = "fn main() { let mut x = 1; let r = &mut x; let y = x + 1; }";
    let tree = parse(src).unwrap();
    let facts = extract(&tree);
    assert!(!facts.bindings.is_empty());

    let (named, anon, err, trivia) = tree.stats();
    assert!(named > 0);
    assert!(anon > 0);
    assert_eq!(err, 0);
    assert!(trivia > 0);
    assert!(!tree.named_node_ids().is_empty());

    assert!(conflicts(EvKind::BorrowMut, EvKind::BorrowMut));
    assert!(conflicts(EvKind::BorrowMut, EvKind::Read));

    // Edit Monoid tests
    let e1 = Edit::new(10, 11, "y");
    let e2 = Edit::new(20, 21, "z");
    assert!(is_pairwise_disjoint(&[e1.clone(), e2.clone()]));
    let applied = apply_all("abcdefghijklmnopqrstuvwxyz", &[e1.clone(), e2.clone()]);
    assert!(!applied.is_empty());

    let seq = compose_seq(&[e1.clone(), e2.clone()]);
    assert!(seq.is_some());

    let e_comp = compose(&e1, &Edit::new(25, 26, "w"));
    assert!(e_comp.is_some());

    // SpanMonad patch test
    let anchor = SpanAnchor {
        event_id: 0,
        ast_node_id: 0,
        source_span: Span::new(10, 30),
        fact_interval: cl0r0::ast::Interval { start: 1, end: 10 },
    };
    let patch = SpanMonad::synthesize_patch(&tree, &anchor, 5, src);
    assert!(patch.is_some());
}

#[test]
fn test_rep_menu_and_l9newman_channel() {
    use cl0r0::l9newman::newman_check;
    use cl0r0::rep::{Menu, Policy};

    let rep_res = newman_check(Menu::CommutativeTrim, Policy::Guarded, 2, 4, 4);
    assert_eq!(rep_res.l8_violations.len(), 0);
    assert_eq!(rep_res.non_joinable.len(), 0);
    assert!(rep_res.unique_nf_states > 0);
}

#[test]
fn test_full_json_error_report_and_repair_fixpoint() {
    use cl0r0::json_report::JsonDiagnosticPipeline;

    let sample_src = "fn main() {\n    let mut x = 1;\n    let r = &mut x;\n}";
    let (patched, report) = JsonDiagnosticPipeline::execute_json_repair_to_fixpoint(sample_src)
        .expect("JSON repair pipeline should succeed");

    assert_eq!(report.error_count, 0);
    assert!(report.repair_applied);
    assert!(report.reparse_verified);
    assert!(report.polonius_converged);
    assert!(patched.contains("// [cl0r0 auto-drop]: borrow region shortened"));

    let json_text = report.to_json_string();
    assert!(json_text.contains("\"status\": \"repaired_and_certified\""));
    assert!(json_text.contains("\"error_count\": 0"));
    assert!(json_text.contains("\"reparse_verified\": true"));
}

#[test]
fn test_mir_cf_graph_and_move_analysis_dropck() {
    let mut body = MirBody::new(2);
    let ret = body.add_local(MirType::Int(32), true, Span::new(0, 5), Some("_0".into()));
    let arg1 = body.add_local(MirType::Int(32), false, Span::new(5, 10), Some("_1".into()));
    let arg2 = body.add_local(
        MirType::Int(32),
        false,
        Span::new(10, 15),
        Some("_2".into()),
    );

    let mut bb0 = BasicBlockData::new(Some(Terminator {
        kind: TerminatorKind::Return,
        span: Span::new(20, 25),
    }));

    bb0.statements.push(Statement {
        kind: StatementKind::Assign(
            Place::from_local(ret),
            Rvalue::BinaryOp(
                cl0r0::mir::MirBinOp::Add,
                Operand::Copy(Place::from_local(arg1)),
                Operand::Copy(Place::from_local(arg2)),
            ),
        ),
        span: Span::new(15, 20),
    });

    body.add_block(bb0);
    assert_eq!(body.num_blocks(), 1);
    assert_eq!(body.num_locals(), 3);

    let move_data = MoveData::build(&body);
    let init_states = MoveAnalysisSolver::compute_init_states(&body, &move_data);
    let errors = MoveAnalysisSolver::check_use_validity(&body, &move_data, &init_states);
    assert_eq!(
        errors.len(),
        0,
        "合法的引數加法不應有未初始化或 Move 後使用錯誤"
    );

    let drop_seq = DropElaborator::elaborate_scope_drops(&[arg1, arg2], &body);
    assert_eq!(drop_seq.drops.len(), 2);
    assert_eq!(drop_seq.drops[0].0, Place::from_local(arg2)); // 宣告反序
    assert_eq!(drop_seq.drops[1].0, Place::from_local(arg1));
}

#[test]
fn test_oopsla_2025_modular_contracts_and_reborrow_chains() {
    let mut reborrow_mgr = ReborrowManager::new();
    reborrow_mgr.loan_status.insert(100, ReborrowStatus::Active);

    // Reborrow 借用鏈: 100 (父) -> 101 (子)
    reborrow_mgr.issue_reborrow(
        100,
        101,
        Place::from_local(Local(1)).deref(),
        BorrowKind::Mut {
            allow_two_phase_borrow: false,
        },
    );
    assert_eq!(reborrow_mgr.loan_status[&100], ReborrowStatus::Suspended);
    assert_eq!(reborrow_mgr.loan_status[&101], ReborrowStatus::Active);

    // 結束子借用
    reborrow_mgr.expire_loan(101);
    assert_eq!(reborrow_mgr.loan_status[&100], ReborrowStatus::Active);

    // 循環不動點測試
    let fix = LoopFixpointSolver::solve_loop_fixpoint(&[1, 2], |s| {
        let mut n = s.clone();
        n.insert(3);
        n
    });
    assert!(fix.is_fixpoint);
    assert_eq!(fix.back_edge_loans.len(), 3);

    // OOPSLA ~97% 覆蓋率基準評測
    let swap_c = IdiomaticPatternLibrary::contract_swap();
    let subslice_c = IdiomaticPatternLibrary::contract_subslice();
    let iter_c = IdiomaticPatternLibrary::contract_iter_mut_next();
    let (verified, total, rate) =
        IdiomaticPatternLibrary::benchmark_oopsla_coverage(&[swap_c, subslice_c, iter_c]);
    assert_eq!(verified, total);
    assert!((rate - 1.0).abs() < 1e-6);
}

#[test]
fn test_aeneas_creusot_prusti_proof_resources() {
    // 1. Aeneas 反向函數求值
    let swap_trans = AeneasTranslator::translate_swap_example();
    let mut env = HashMap::new();
    env.insert("x".into(), 100);
    env.insert("y".into(), 200);
    assert_eq!(
        AeneasTranslator::eval_expr(&swap_trans.backward_functions[0].1, &env),
        200
    );
    assert_eq!(
        AeneasTranslator::eval_expr(&swap_trans.backward_functions[1].1, &env),
        100
    );

    // 2. Creusot 預言單元 Resolve 與 Reborrow 鏈
    let mut penv = ProphecyEnvironment::new();
    penv.register_borrow("acc", 50, 75);
    assert!(penv.register_reborrow("acc", "acc_child").is_ok());
    assert_eq!(penv.resolve_borrow("acc_child"), Some(75));
    assert_eq!(penv.cells["acc"].current_val, 75);

    // 3. Prusti 分數權限拆分與合併守恆
    let p_full = Permission::Exclusive;
    let (p1, p2) = p_full.split().unwrap();
    let p_rejoined = p1.join(&p2).unwrap();
    assert_eq!(p_rejoined, Permission::Exclusive);
}

#[test]
fn test_creusot_why3_theory_and_pearlite_contracts() {
    let theory = cl0r0::creusot_export::CreusotExporter::export_full_creusot_theory(
        "CL0_Creusot_Integration",
    );
    assert!(theory.contains("type mut_borrow"));
    assert!(theory.contains("predicate reborrow_valid"));
    assert!(theory.contains("lemma_l18_diff_structural_sharing"));

    if cl0r0::creusot_export::CreusotExporter::check_why3_available()
        && cl0r0::creusot_export::CreusotExporter::check_z3_available()
    {
        let res = cl0r0::creusot_export::CreusotExporter::verify_with_why3(
            &theory,
            "CL0_Creusot_Integration",
        );
        assert!(res.is_ok(), "Why3 + Z3 verification failed: {:?}", res);
        let rep = res.unwrap();
        assert!(rep.success);
        assert_eq!(rep.valid_goals, rep.total_goals);
    }
}

#[test]
fn test_variance_inference_dropck_eye_of_needle_and_ub_oracle() {
    // 1. Variance 推導
    let t_param = MirType::TypeParam("T".into());
    let mut_ref = MirType::Ref(
        RegionVid(0),
        Box::new(t_param.clone()),
        BorrowKind::Mut {
            allow_two_phase_borrow: false,
        },
    );
    assert_eq!(
        VarianceEngine::infer_variance_of_param(&mut_ref, "T"),
        Variance::Invariant
    );

    let fn_ptr = MirType::FnPtr {
        params: vec![t_param],
        ret: Box::new(MirType::Tuple(vec![])),
    };
    assert_eq!(
        VarianceEngine::infer_variance_of_param(&fn_ptr, "T"),
        Variance::Contravariant
    );

    // 2. Dropck 針眼法則
    let safe_constraint = vec![DropckGenericConstraint {
        type_param: "T".into(),
        has_may_dangle: true,
        used_in_destructor: false,
    }];
    assert!(DropckChecker::verify_dropck_safety("MyVec", &safe_constraint).is_ok());

    let unsafe_constraint = vec![DropckGenericConstraint {
        type_param: "T".into(),
        has_may_dangle: true,
        used_in_destructor: true,
    }];
    assert!(DropckChecker::verify_dropck_safety("BadVec", &unsafe_constraint).is_err());

    // 3. UCG / Rust Reference 未定義行為預言機
    assert!(UbDiagnosticOracle::check_bool_validity(255).is_some());
    assert!(UbDiagnosticOracle::check_pointer_access(0, 8).is_some());
    assert!(UbDiagnosticOracle::check_stacked_borrows_access(true, false, 0).is_some());
}

#[test]
fn test_differential_checking_and_audit_matrix_92_percent() {
    let report = DifferentialAuditSuite::execute_full_differential_audit(50);
    assert!(
        report.reaches_92_percent_target,
        "差分審核綜合評分 {} 必須達到並超越 92% (0.92) 目標！",
        report.overall_score
    );
    assert!(report.overall_score >= 0.92);
    for dim in &report.dimensions {
        assert!(
            dim.is_compliant,
            "維度 `{}` 評分 {} 未達 92% 門檻！",
            dim.name, dim.pass_rate
        );
    }
}

#[test]
fn test_persistent_diff_tree_version_chain_and_sharing() {
    let mut stmts = Vec::new();
    let mut offset = 0u32;
    for i in 0..40 {
        let s = format!("let z_{} = {};", i, i);
        let len = s.len() as u32;
        let span = Span::new(offset, offset + len);
        let leaf = DiffAstNode::leaf(i as u64, &s, span);
        let stmt = Arc::new(DiffAstNode::new(
            500 + (i as u64),
            DiffNodeType::Stmt("let".into(), vec![leaf]),
            span,
        ));
        stmts.push(stmt);
        offset += len + 1;
    }
    let base_root = DiffAstNode::root(1, stmts, Span::new(0, offset));
    let mut chain = DiffAstVersionChain::new(base_root);

    // Commit edit 1
    let (_v1, stats1) = chain.commit_patch(25, 1, "42");
    assert!(stats1.sharing_ratio >= 0.92);

    // Commit edit 2
    let (_v2, stats2) = chain.commit_patch(60, 1, "99");
    assert!(stats2.sharing_ratio >= 0.92);
    assert_eq!(chain.snapshots.len(), 3);
}
