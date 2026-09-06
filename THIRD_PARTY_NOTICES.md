THIRD-PARTY NOTICES — 第三方軟件聲明清單
=========================================
(內部合規文件;隨產品分發時須按各授權條款附帶本清單)

1. Rust 工具鏈與標準庫 (runtime 依賴)
   ---------------------------------------------------------------
   本引擎以 Rust 編譯,二進位產物靜態鏈結 Rust 標準庫。
   • 授權:MIT OR Apache-2.0 (雙授權,任擇其一)
   • 版權:The Rust Project Contributors / The Rust Foundation
   • 條款要求:保留版權與授權聲明(本清單即履行此義務)
   • 官方條款全文:
     https://github.com/rust-lang/rust/blob/master/LICENSE-MIT
     https://github.com/rust-lang/rust/blob/master/LICENSE-APACHE

2. 運行時第三方依賴
   ---------------------------------------------------------------
   無。
   本引擎 Cargo.toml 不含任何 [dependencies](零第三方 crate),
   經 `cargo tree` 可複核。此為盡職調查要點。

3. 開發/構建/CI 工具(不分發於產品,僅作內部合規紀錄)
   ---------------------------------------------------------------
   • rustc / cargo / rustfmt / clippy — MIT OR Apache-2.0 (Rust Project)
   • cargo-llvm-cov — MIT OR Apache-2.0 (cargo-llvm-cov contributors)
   • LLVM llvm-tools-preview — Apache-2.0 WITH LLVM-exception (LLVM Project)
   • GitHub Actions — GitHub Terms of Service
   • Rocq (rocq-core / rocq-stdlib, CI 證明器) — LGPL-2.1 (INRIA)
   • Why3 — LGPL-2.1 (CNRS/INRIA/UJF)
   • Z3 — MIT OR Apache-2.0 (Microsoft Research)
   註:Rocq/Why3/Z3 僅於 CI 作外部證明器呼叫,不鏈結進產品二進位。

4. 聲明
   ---------------------------------------------------------------
   本清單由構建時點之依賴掃描生成;新增任何依賴時必須同步更新,
   並由 dev_loop 看板項目(DL-009 起)納入驗收。
