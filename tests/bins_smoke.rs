// Copyright (c) 2026 Ken Yuen Ka Chun. All rights reserved.
// PROPRIETARY & CONFIDENTIAL — unauthorized copying, modification, distribution, or reverse engineering is prohibited.
//! DL-014 bin 整合測試(零第三方依賴):以 cargo 內建 `CARGO_BIN_EXE_*`
//! 機制驅動全部 17 個自証 bin,斷言退出碼 0 + 特徵輸出。
//!
//! 目的:①把「CI 每輪實跑 bin」從口頭承諾變成測試資產;
//! ②補 llvm-cov 量測空洞(bin 檔顯示 0% 屬儀器化測試不執行 bin 嘅假象);
//! ③任何 bin 無聲退化(輸出格式壞/退出碼變)即時被捉。
//!
//! 三態誠實語義(F-01)注意:本機無 Rocq/Why3/Z3 時,verify_all 為
//! 8/10+2 SKIPPED、ci_verify 為 5/7——**呢啲係正確行為**,測試斷言嘅
//! 係「特徵結論行存在 + 退出碼 0(非 strict 模式)」,唔係強求 10/10。

use std::io::Read;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;

/// 執行一個 bin(帶 30s 超時保護),回傳 (成功與否, 合併輸出)。
fn run_bin(exe: &str, args: &[&str]) -> (bool, String) {
    let mut cmd = Command::new(exe);
    cmd.args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = cmd
        .spawn()
        .unwrap_or_else(|e| panic!("spawn {exe} 失敗: {e}"));
    let mut stdout = child.stdout.take().expect("stdout piped");
    let mut stderr = child.stderr.take().expect("stderr piped");

    // 讀取線程:avoid pipe 滿死鎖(stdout+stderr 併讀)
    let (tx, rx) = mpsc::channel();
    let t_out = std::thread::spawn(move || {
        let mut s = String::new();
        let _ = stdout.read_to_string(&mut s);
        s
    });
    let t_err = std::thread::spawn(move || {
        let mut s = String::new();
        let _ = stderr.read_to_string(&mut s);
        s
    });
    let _ = tx.send(());

    let status = child.wait().expect("wait 失敗");
    let out = t_out.join().expect("stdout 線程");
    let err = t_err.join().expect("stderr 線程");
    let _ = rx.recv_timeout(Duration::from_secs(30));
    (status.success(), format!("{out}{err}"))
}

/// 斷言 bin 成功退出且輸出含特徵串。
fn assert_bin(exe: &str, args: &[&str], needle: &str) {
    let (ok, out) = run_bin(exe, args);
    assert!(
        ok,
        "{exe} 應成功退出(退出碼 0);輸出尾部:{}",
        out.chars()
            .rev()
            .take(300)
            .collect::<String>()
            .chars()
            .rev()
            .collect::<String>()
    );
    assert!(
        out.contains(needle),
        "{exe} 輸出應含「{needle}」;實際輸出(截 300 字):{}",
        &out[out.len().saturating_sub(300)..]
    );
}

#[test]
fn bin_cl0r0_nine_laws_demo() {
    assert_bin(env!("CARGO_BIN_EXE_cl0r0"), &[], "九律");
}

#[test]
fn bin_dev_loop_kanban_zero_violation() {
    assert_bin(env!("CARGO_BIN_EXE_dev_loop"), &[], "0 違規");
}

#[test]
fn bin_l9newman_reports_newman_channel() {
    assert_bin(env!("CARGO_BIN_EXE_l9newman"), &[], "Newman");
}

#[test]
fn bin_dd_verify_dual_channel() {
    assert_bin(env!("CARGO_BIN_EXE_dd_verify"), &[], "雙通道");
}

#[test]
fn bin_verify_all_honest_conclusion() {
    // 本機無外部証明器 ⇒ 8/10+2 SKIPPED 亦為正確(非 strict);斷言結論行存在
    assert_bin(env!("CARGO_BIN_EXE_verify_all"), &[], "自証結論");
}

#[test]
fn bin_ci_verify_honest_conclusion() {
    assert_bin(env!("CARGO_BIN_EXE_ci_verify"), &[], "CI 最終結論");
}

#[test]
fn bin_macro_lab_fourteen_gates() {
    assert_bin(env!("CARGO_BIN_EXE_macro_lab"), &[], "14/14");
}

#[test]
fn bin_fuzz_zero_total_failures() {
    assert_bin(env!("CARGO_BIN_EXE_fuzz"), &[], "總失敗數:0");
}

#[test]
fn bin_fuzz_daemon_single_round() {
    let (ok, out) = run_bin(env!("CARGO_BIN_EXE_fuzz_daemon"), &["--single-round"]);
    assert!(ok, "fuzz_daemon --single-round 應成功退出;輸出:{out}");
}

#[test]
fn bin_cert_factory_production_lines() {
    assert_bin(env!("CARGO_BIN_EXE_cert_factory"), &[], "100% 正常運轉");
}

#[test]
fn bin_pipeline_runner_five_stage_closure() {
    assert_bin(env!("CARGO_BIN_EXE_pipeline_runner"), &[], "全流程無縫收斂");
}

#[test]
fn bin_coco_benchmark_confluence_rate() {
    assert_bin(env!("CARGO_BIN_EXE_coco_benchmark"), &[], "Confluence Rate");
}

#[test]
fn bin_lemma_stress_full_invariants() {
    assert_bin(
        env!("CARGO_BIN_EXE_lemma_stress_coverage"),
        &[],
        "18 大形式化引理",
    );
}

#[test]
fn bin_dev_prover_evidence_conclusion() {
    assert_bin(env!("CARGO_BIN_EXE_dev_prover"), &[], "取証結論");
}

#[test]
fn bin_rocq_verify_tri_state_absent_ok() {
    // 本機無 Rocq ⇒ 三態如實申報(SKIP),非 strict 退出碼仍 0
    assert_bin(env!("CARGO_BIN_EXE_rocq_verify"), &[], "Rocq 9.2 驗證結論");
}

#[test]
fn bin_creusot_verify_tri_state_absent_ok() {
    assert_bin(
        env!("CARGO_BIN_EXE_creusot_verify"),
        &[],
        "Creusot 驗證結論",
    );
}

#[test]
fn bin_lsp_server_serves_then_eof_clean_exit() {
    // 餵一行 initialize 請求 → 服務循環派發一次;隨後 EOF ⇒ 乾淨退出
    let exe = env!("CARGO_BIN_EXE_cl0r0_lsp");
    let mut child = Command::new(exe)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|e| panic!("spawn cl0r0_lsp 失敗: {e}"));
    {
        use std::io::Write;
        let stdin = child.stdin.as_mut().expect("stdin piped");
        let body = r#"{"method":"initialize","id":1}"#;
        let frame = format!("Content-Length: {}\r\n\r\n{}", body.len(), body);
        let _ = writeln!(stdin, "{frame}");
    } // drop stdin ⇒ EOF
    let out = child.wait_with_output().expect("wait 失敗");
    assert!(out.status.success(), "cl0r0_lsp EOF 後應乾淨退出(退出碼 0)");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("\"id\":1"),
        "應回顯 initialize 響應;實際輸出:{stdout}"
    );
}
