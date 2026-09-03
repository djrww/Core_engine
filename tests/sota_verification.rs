//! tests/sota_verification.rs
//!
//! 综合集成测试套件：覆盖 Decreasing Diagrams、Newman 快速通道 (SN ∧ WCR ⇒ CR)、
//! 五大组合深度合成流水线 (1+2+3+4+5)、CPF-KB 短证、Tactic Scheduler、
//! 证书与污料生产机 (CertGeneratorFactory) 与 Polonius 桥接。

use cl0r0::ari_export::export_menu_to_ari;
use cl0r0::ast::Interval;
use cl0r0::cert_generator_factory::CertGeneratorFactory;
use cl0r0::cpf_cert::{CPFCertificate, CertResult};
use cl0r0::dd_checker::{check_confluence_with_mode, CheckerMode, SNWitness};
use cl0r0::edit::Edit;
use cl0r0::lsp_bridge::LspEngine;
use cl0r0::parse::parse;
use cl0r0::patch_engine::PatchEngine;
use cl0r0::pipeline_synthesis::EndToEndSynthesizer;
use cl0r0::polonius_bridge::PoloniusBridge;
use cl0r0::r0_lower::R0Lowerer;
use cl0r0::rep_dd::{AState, Ev, K};
use cl0r0::reparse_verifier::verify_reparse_equivalence;
use cl0r0::rustc_json::RustcJsonAutomaton;
use cl0r0::shrink::shrink_source;
use cl0r0::tactic_scheduler::{Tactic, TacticScheduler};

fn sample_states() -> Vec<AState> {
    let mut states = Vec::new();
    for s1 in 0..3 {
        for e1 in (s1 + 1)..=4 {
            for s2 in 1..4 {
                for e2 in (s2 + 1)..=4 {
                    states.push(AState::new(vec![
                        Ev {
                            id: 0,
                            storage: 0,
                            kind: K::Mut,
                            it: Interval { start: s1, end: e1 },
                        },
                        Ev {
                            id: 1,
                            storage: 0,
                            kind: K::Sh,
                            it: Interval { start: s2, end: e2 },
                        },
                    ]));
                }
            }
        }
    }
    states
}

#[test]
fn test_cert_and_dirty_generator_factory() {
    let artifacts = CertGeneratorFactory::run_factory_production(20, 0xC10_2024_0001);
    assert_eq!(artifacts.dirty_samples_count, 20);
    assert!(!artifacts.generated_cpf_certificates.is_empty());
    assert_eq!(artifacts.generated_ari_problems.len(), 20);
    assert_eq!(artifacts.polonius_facts_catalog.len(), 20);
    assert!(artifacts.total_certified_rate > 90.0);

    let mut rng = cl0r0::gen::Rng::new(0xCAFE);
    let dirty_universe = CertGeneratorFactory::produce_dirty_universe(&mut rng, 30);
    let (passed, failed) = CertGeneratorFactory::verify_dirty_robustness(&dirty_universe);
    assert_eq!(failed, 0, "全化解析器对所有污料样本应 0 Panic 构造树");
    assert_eq!(passed, 30);
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
    let cert_kb = CPFCertificate::new_knuth_bendix("CL0-KB", "LivenessBounded", 24);
    assert_eq!(cert_kb.verify(), CertResult::Certified);
    assert!(cert_kb.to_cpf_xml().contains("<crKnuthBendix>"));

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
    let state = AState::new(vec![
        Ev {
            id: 0,
            storage: 0,
            kind: K::Mut,
            it: Interval { start: 1, end: 3 },
        },
        Ev {
            id: 1,
            storage: 0,
            kind: K::Sh,
            it: Interval { start: 2, end: 4 },
        },
    ]);

    let d_facts = PoloniusBridge::export_to_polonius_facts(&state);
    assert!(d_facts.contains("loan_issued_at('orig_0, 0, 1)."));
    assert!(d_facts.contains("invalidates(1, 0)."));

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
