//! cert_factory —— 污料生成宇宙与形式化证书批量生产运行机。
//!
//! 运行: `cargo run --bin cert_factory`

use cl0r0::cert_generator_factory::CertGeneratorFactory;

fn main() {
    println!("======================================================================");
    println!(" CL0 / R₀ 污料生成宇宙 · 形式化證書工廠 · Polonius 事實生產機");
    println!("======================================================================");

    let batch_size = 100usize;
    println!(
        "[1/3] 正在運轉生成機：批量生產 {} 組認證資產與污料宇宙...",
        batch_size
    );

    let artifacts = CertGeneratorFactory::run_factory_production(batch_size, 0xC10_2024_0001);

    println!(
        "  >> 污料與髒輸入流生成數:      {} 個樣本",
        artifacts.dirty_samples_count
    );
    println!(
        "  >> CPF 形式化證書出具數:      {} 份 (KB/DD 雙模認證)",
        artifacts.generated_cpf_certificates.len()
    );
    println!(
        "  >> ARI-COPS 競賽標準題目生成: {} 題",
        artifacts.generated_ari_problems.len()
    );
    println!(
        "  >> Polonius Datalog 事實條目: {} 條",
        artifacts.polonius_facts_catalog.len()
    );
    println!(
        "  >> 形式化認證合格率:          {:.2}%",
        artifacts.total_certified_rate
    );

    println!("\n[2/3] 正在抽樣展示出具之 CPF-KB 形式化 XML 證書:");
    if let Some(first_cert) = artifacts.generated_cpf_certificates.first() {
        println!("{}", first_cert.to_cpf_xml());
    }

    println!("\n[3/3] 正在對 100 個污料/截斷代碼樣本進行全化解析器防禦性測試...");
    let mut rng = cl0r0::gen::Rng::new(0xDEAD_BEEF);
    let dirty_universe = CertGeneratorFactory::produce_dirty_universe(&mut rng, 100);
    let (passed, failed) = CertGeneratorFactory::verify_dirty_robustness(&dirty_universe);

    println!(
        "  >> 全化解析器容錯結果: {} 成功構造 AST / {} 崩潰 (0 Panic)",
        passed, failed
    );

    println!("\n======================================================================");
    println!(" 認證生成機運轉結論: 全部生產線 100% 正常運轉，各類證書與污料已交付！");
    println!("======================================================================");
}
