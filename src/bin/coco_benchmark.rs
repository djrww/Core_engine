//! coco_benchmark —— 国际合流基准压力测试 (CoCo Benchmark Suite)。
//!
//! 运行: `cargo run --release --bin coco_benchmark`

use cl0r0::ast::Interval;
use cl0r0::dd_checker::check_decreasing_diagrams;
use cl0r0::rep_dd::{AState, Ev, K};
use std::time::Instant;

fn main() {
    println!("======================================================================");
    println!(" CoCo 2025/2026 國際合流基準壓力測試 (CL0 Benchmark Matrix)");
    println!("======================================================================");

    let max_coords = 4;
    println!("[配置]: 空間坐標界限 = 0..={}", max_coords);

    let start_gen = Instant::now();
    let mut benchmark_states = Vec::new();

    for s1 in 0..max_coords {
        for e1 in (s1 + 1)..=max_coords {
            for s2 in 1..max_coords {
                for e2 in (s2 + 1)..=max_coords {
                    benchmark_states.push(AState::new(vec![
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
    let gen_duration = start_gen.elapsed();
    println!(
        "  >> 狀態空間生成完畢: {} 個狀態配置 (耗時: {:?})",
        benchmark_states.len(),
        gen_duration
    );

    println!("\n[執行]: 正在運行 van Oostrom 遞減圖局部峰值會合性驗證...");
    let start_verify = Instant::now();
    let report = check_decreasing_diagrams(&benchmark_states, 6);
    let verify_duration = start_verify.elapsed();

    println!("======================================================================");
    println!(" 基准测试结果报告:");
    println!(
        "   - 状态总数 (Total States):           {}",
        report.total_states
    );
    println!(
        "   - 局部峰值总数 (Total Local Peaks):    {}",
        report.total_peaks
    );
    println!(
        "   - 递减山谷会合数 (Decreasing Valleys): {}",
        report.decreasing_valleys_found
    );
    println!(
        "   - 不可会合峰值数 (Non-joinable):       {}",
        report.non_joinable_peaks.len()
    );
    println!(
        "   - 总核验耗时 (Total Duration):         {:?}",
        verify_duration
    );
    if report.total_states > 0 {
        println!(
            "   - 单状态平均处理延迟:                  {:.2} μs/state",
            (verify_duration.as_micros() as f64) / (report.total_states as f64)
        );
    }
    println!(
        "   - 合流性达标率 (Confluence Rate):      {:.2}%",
        if report.certified { 100.0 } else { 0.0 }
    );
    println!("======================================================================");
}
