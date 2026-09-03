//! fuzz_daemon —— 本地 4 小时一轮自动化 Fuzzing 守护进程。
//!
//! 运行: `cargo run --release --bin fuzz_daemon`

use cl0r0::gen::{gen_garbage, gen_legal, Rng};
use cl0r0::parse::parse;
use cl0r0::shrink::shrink_source;
use std::thread::sleep;
use std::time::Duration;

fn main() {
    println!("======================================================================");
    println!(" CL0 / R₀ 4 小時定時 Fuzzing 守護進程已啟動 (每 14,400 秒輪詢一次)");
    println!("======================================================================");

    let round_interval = Duration::from_secs(4 * 3600);
    let mut round_count = 1usize;

    // 单次运行演示后进入周期循环 (支持 --single-round 参数)
    let single_round = std::env::args().any(|arg| arg == "--single-round");

    loop {
        println!("\n>>> [第 {} 輪 Fuzzing 開始]", round_count);

        let seed = 0xC10_2024_0001 ^ (round_count as u64);
        let mut rng = Rng::new(seed);
        let iterations = 20_000;
        let mut violations = 0usize;

        for i in 0..iterations {
            let src = if i % 2 == 0 {
                gen_legal(&mut rng)
            } else {
                gen_garbage(&mut rng, 35)
            };

            match parse(&src) {
                Ok(tree) => {
                    if tree.unparse() != src {
                        violations += 1;
                        let min_repro = shrink_source(src.clone(), |s| {
                            parse(s).is_ok_and(|t| t.unparse() != s)
                        });
                        eprintln!("[FAIL] L1 违反 (Iteration {})! 最小反例:\n{}", i, min_repro);
                    }
                }
                Err(err) => {
                    violations += 1;
                    eprintln!("[FAIL] 解析器报错: {:?}", err);
                }
            }
        }

        if violations == 0 {
            println!(
                "<<< [第 {} 輪 Fuzzing 完成]: {}/{} 通過 (0 違規 · 100% 綠色)",
                round_count, iterations, iterations
            );
        } else {
            eprintln!(
                "<<< [第 {} 輪 Fuzzing 警告]: 發現 {} 處違規！",
                round_count, violations
            );
        }

        if single_round {
            println!(">>> --single-round 已完成，正常退出。");
            break;
        }

        round_count += 1;
        println!(">>> 正在休眠 4 小時後進入下一輪...");
        sleep(round_interval);
    }
}
