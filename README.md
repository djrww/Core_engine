# Core_engine (`cl0r0`)

CL0/R0 雙載體的機械自証代碼庫:表面語法樹、九條定律(L1–L9)、機械自証。

## 工具鏈(固定)

* rust **1.98.0** + **clippy** + **rustfmt** + **rust-analyzer**
  (由 `rust-toolchain.toml` 固定,rustup 自動切換;CI 用 `dtolnay/rust-toolchain@1.98.0`)

## 檢測門(與 CI 一致)

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
```

機械自証(三個二進制;fuzz / l9newman 失敗時以非零退出碼結束):

```sh
cargo run --bin cl0r0      # 九律演示(全綠 = 自証通過)
cargo run --bin l9newman   # Newman 通道:SN ∧ WCR ⇒ 唯一正規形
cargo run --bin fuzz       # 屬性測試(確定性種子 0xC1020240001,可重現)
```

## 發布(release v0.1.1)

* `[profile.release]`:`opt-level = 3`、`lto = true`(fat LTO)、`codegen-units = 1`、`debug = true`(保留調試符號)
* CI:push 標籤 `v*` → 校驗「標籤 = Cargo.toml 版本」→ `cargo build --release` → 打包三個二進制 → 建立 GitHub Release

## 結構

| 模塊 | 職責 |
| --- | --- |
| `span` | σ(v) = [a,b) 半開區間(§1.2) |
| `lex` | CL0 詞法器 —— DFA,平鋪全源碼,trivia 保留(§5.1) |
| `parse` | 表面語法樹 + 增量重析 + ERROR 全化(§1–§2.3) |
| `tree` | 樹的性質檢查(連續性公理、laminar、CW 復形) |
| `edit` | 編輯單體(§2.1:位移函數、複合、結合律) |
| `ast` | 語義面 —— liveness 三軌、衝突圖(§3.2–§3.3) |
| `rep` | 修法菜單重寫系統(§4:L8 遞減測度、L9 合流) |
| `gen` | 合法程式 + 髒輸入生成器(屬性測試的輸入宇宙) |
| `r0` | R₀ Rust 子集 —— 附錄 B 覆蓋面契約 + unsupported 申報 |
| `l9newman` | 機械的 Newman 通道(終止 + 局部合流 ⇒ 合流 ⇒ 唯一正規形) |
