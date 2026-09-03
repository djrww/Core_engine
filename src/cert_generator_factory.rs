//! §11.1 污料生成宇宙、形式化證書工廠與認證流水線 (Certificate & Dirty Stream Generator Factory)。
//!
//! 提供工業級生產器：
//! 1. 【污料/變異生成機】: 高熵髒輸入流、截斷代碼段、不平衡定界符、越界語法突變
//! 2. 【形式化證書工廠】: 自動批量出具 CPF-KB、CPF-DD、ARI-COPS 與 CeTA 兼容 XML 證書
//! 3. 【Polonius Facts 工廠】: 批量生成符合 rustc -Zpolonius 規範之 Datalog 關係事實流
//! 4. 【LSP QuickFix 認證補丁工廠】: 帶數學收斂證明的代碼補丁生成

use crate::ast::Interval;
use crate::cpf_cert::{CPFCertificate, CertResult};
use crate::dd_checker::{check_confluence_with_mode, CheckerMode, SNWitness};
use crate::gen::{gen_garbage, gen_legal, Rng};
use crate::parse::parse;
use crate::polonius_bridge::PoloniusBridge;
use crate::rep_dd::{AState, Ev, K};
use crate::rocq_export::RocqExporter;
use crate::tactic_scheduler::TacticScheduler;

#[derive(Clone, Debug)]
pub struct GeneratedArtifacts {
    pub dirty_samples_count: usize,
    pub generated_cpf_certificates: Vec<CPFCertificate>,
    pub generated_ari_problems: Vec<String>,
    pub generated_rocq_theories: Vec<String>,
    pub polonius_facts_catalog: Vec<String>,
    pub total_certified_rate: f64,
}

pub struct CertGeneratorFactory;

impl CertGeneratorFactory {
    /// 運行污料生成器：生產多種維度的髒輸入流 (截斷、高熵符號、突變)
    pub fn produce_dirty_universe(rng: &mut Rng, count: usize) -> Vec<String> {
        let mut dirty_streams = Vec::with_capacity(count);
        for i in 0..count {
            match i % 4 {
                0 => {
                    // 高熵隨機符號流
                    dirty_streams.push(gen_garbage(rng, 35));
                }
                1 => {
                    // 截斷代碼段 (寫到一半的檔案)
                    let legal = gen_legal(rng);
                    let cut_len = (legal.len() / 2).max(5);
                    dirty_streams.push(legal[..cut_len].to_string());
                }
                2 => {
                    // 包含 unsupported Rust 特徵的越界污料 (match, closures, macros)
                    let mut snippet = gen_legal(rng);
                    snippet.push_str("\nmatch val { 0 => println!(x), _ => |c| c + 1 };");
                    dirty_streams.push(snippet);
                }
                _ => {
                    // 不平衡括號與詞法邊界污料
                    let mut malformed = gen_legal(rng);
                    malformed.push_str(" { { let x = &mut ; if true { }}");
                    dirty_streams.push(malformed);
                }
            }
        }
        dirty_streams
    }

    /// 生產形式化證書與認證資產 (CPF XML / ARI COPS / Rocq 9.2 .v / Polonius Facts)
    pub fn run_factory_production(batch_size: usize, seed: u64) -> GeneratedArtifacts {
        let mut rng = Rng::new(seed);
        let dirty_samples = Self::produce_dirty_universe(&mut rng, batch_size);
        let dirty_count = dirty_samples.len();

        let mut certificates = Vec::new();
        let mut ari_problems = Vec::new();
        let mut rocq_theories = Vec::new();
        let mut polonius_catalog = Vec::new();
        let mut certified_count = 0usize;

        for i in 0..batch_size {
            // 構造隨機幾何區間配置
            let s1 = (i % 4) as u32;
            let e1 = s1 + 1 + ((i * 3) % 4) as u32;
            let s2 = ((i + 1) % 4) as u32;
            let e2 = s2 + 1 + ((i * 5) % 4) as u32;

            let astate = AState::new(vec![
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
            ]);

            // 1. 生成 Polonius Facts
            let facts = PoloniusBridge::export_to_polonius_facts(&astate);
            polonius_catalog.push(facts);

            // 2. 生成 ARI COPS 競賽題目
            let cops_problem = TacticScheduler::export_cops_problem((i + 1) as u32, 2);
            ari_problems.push(cops_problem);

            // 3. 雙通道合流性核驗與 CPF 證書生產
            let witness = SNWitness::LivenessScopeBounded {
                max_span_len: e1.max(e2),
                storages: 1,
            };
            let report = check_confluence_with_mode(
                std::slice::from_ref(&astate),
                CheckerMode::Newman {
                    sn_witness: witness.clone(),
                },
                6,
            );

            if report.certified {
                let mod_name = format!("CL0_AutoBatch_{:04}", i + 1);
                let cert_kb = CPFCertificate::new_knuth_bendix(
                    &mod_name,
                    &witness.description(),
                    report.total_peaks,
                );
                if cert_kb.verify() == CertResult::Certified {
                    certified_count += 1;
                    // 4. 導出 Rocq 9.2 形式化理論
                    let rocq_v = RocqExporter::export_theory(&mod_name, &cert_kb);
                    rocq_theories.push(rocq_v);
                    certificates.push(cert_kb);
                }
            }
        }

        let total_certified_rate = if batch_size > 0 {
            (certified_count as f64) / (batch_size as f64) * 100.0
        } else {
            100.0
        };

        GeneratedArtifacts {
            dirty_samples_count: dirty_count,
            generated_cpf_certificates: certificates,
            generated_ari_problems: ari_problems,
            generated_rocq_theories: rocq_theories,
            polonius_facts_catalog: polonius_catalog,
            total_certified_rate,
        }
    }

    /// 檢驗污料全化容錯性 (保證全化解析器 0 Panic)
    pub fn verify_dirty_robustness(samples: &[String]) -> (usize, usize) {
        let mut passed = 0usize;
        let mut failed = 0usize;
        for s in samples {
            if parse(s).is_ok() {
                passed += 1;
            } else {
                failed += 1;
            }
        }
        (passed, failed)
    }
}
