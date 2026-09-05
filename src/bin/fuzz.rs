//! fuzz —— 屬性測試主程序(確定性種子,可重現)。
//!
//! DL-008:套件引擎已下沉 `cl0r0::fuzz_engine`(可單測);本 bin 只剩
//! 配置讀取(env 驅動,F-08)、報告渲染與退出碼。
//!
//! 運行:`cargo run --bin fuzz`(失敗時以非零退出碼結束)。
//! 覆蓋(全部為「律」級檢查;L3/L4 增量等價依約定排除,見 README):
//!   L1 無損回環、L2 決定論、L5 嵌套、L6 投影一致、L7a 不假報、L7b 良構極大、
//!   L8 遞減、L9 合流/唯一正規形(抽查)+ 編輯單體定律 + 連續性/樹公理 + T2。

use cl0r0::fuzz_engine::{run_property_suite, FuzzConfig};

fn main() {
    let cfg = FuzzConfig::from_env();
    println!(
        "fuzz 迭代配置: main={} legal={} half={} edits={} reparse={} l8={} (env-driven)",
        cfg.main_iter, cfg.legal_iter, cfg.half_iter, cfg.edit_iter, cfg.reparse_iter, cfg.l8_iter,
    );

    let report = run_property_suite(0xC10_2024_0001, &cfg);

    // 失敗明細(引擎只記錄,渲染在此)
    for f in &report.failures {
        eprintln!("LAW FAIL {}", f);
    }

    let mut total_fail = 0usize;
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║ 機械自証報告(種子 0xC1020240001,可重現)                 ║");
    println!("╠══════════════════════════════════════════════════════════╣");
    for s in &report.stats {
        total_fail += s.failed;
        println!(
            "║ {:<6} 檢查 {} 次,失敗 {} 次                                    ║",
            s.law, s.checked, s.failed
        );
    }
    println!("╠══════════════════════════════════════════════════════════╣");
    println!("║ 總失敗數:{}", total_fail);
    println!("╚══════════════════════════════════════════════════════════╝");
    if total_fail > 0 {
        std::process::exit(1);
    }
}
