# Core_engine (`cl0r0`) v0.2.0

CL0/R₀ 雙載體的機械自証代碼庫：表面語法樹、18 大形式化引理自証矩陣、Rocq 9.2 微內核核檢、Creusot / Why3 預言變量演算、van Oostrom 遞減圖（Decreasing Diagrams）、Newman 快速通道（SN ∧ WCR ⇒ CR）、持久化結構共享語法樹（Sharing $\ge 92\%$）、原生 CPF 證書核驗與五大天然組合深度合成閉環。

---

## 專案核心文件導航 (Project Documentation Navigation)

- 🏛️ **[系統全景架構圖與時序圖 (ARCHITECTURE.md)](ARCHITECTURE.md)**: 涵蓋接入點、第一層 (CLI & MCP)、第二層 (雙載體核心與 18 引理) 與第三層 (Rocq 9.2, Creusot/Why3, Isabelle/CeTA, Polonius, Maude) 之 Mermaid 全景架構圖與端到端驗證修復時序圖。
- 📜 **[架構決策記錄與開發事項流水簿 (ADR.md)](ADR.md)**: 記錄 ADR-001 至 ADR-008 之完整決策背景、日期時間、發起人、建議事項、狀態與數學見證。
- 🔬 **[開發階段引理取証工具指南 (PROOF_TOOLCHAIN_GUIDE.md)](PROOF_TOOLCHAIN_GUIDE.md)**: 闡述日常開發期之 CeTA/CPF、Creusot/Why3、Rocq 9.2、Kani 與 DDMin 取証流水線調用方式與証物打包規範。
- 🛡️ **[唯讀核心專案貢獻指南 (CONTRIBUTING.md)](CONTRIBUTING.md)**: 宣告不可變核心專案政策、7 大 CI 驗收門禁、Rocq 9.2 / Creusot 驗證標準與 DDMin 差分反例提交規範。
- 📊 **[18 大形式化引理與自証報告 (DEVELOPMENT_PLAN.md)](DEVELOPMENT_PLAN.md)**: 18 大強類型引理矩陣、MIR 降階、OOPSLA 2025 模組化契約與 CI 門禁證明。
- 📈 **[海量測試數據與差分審核矩陣 (COVERAGE_REPORT.md)](COVERAGE_REPORT.md)**: 79,000 組極限壓測樣本 100% 自証通過與 99.02% 差分審核評分明細。

---

## 核心特徵 (Key Features)

1. **雙載體體系 (Dual Carrier)**:
   - **CL0 定律載體**: 嚴格驗證 18 大形式化引理（L1–L18），保證無損回環與幾何 Laminarity。
   - **R₀ 實用載體**: 面向 Rust 實用子集（附錄 B EBNF），包含 LALR(1) 乾淨子集與 `unsupported` 邊界防火牆。

2. **形式化證明器與演繹驗證雙後端**:
   - **Rocq 9.2 (The Rocq Prover)**: 自動導出 `.v` 理論，由 `rocq compile` 生成 `.vo` 並由獨立微內核 `rocqchk` 進行雙重機械自証。
   - **Creusot (Pearlite & Why3 + Z3)**: 建立可變借用 $(\text{current}, \text{prophecy})$ 預言變量演算，Why3 + Z3 SMT 全自動消解 100% 驗證條件。

3. **合流性雙通道 (Dual Confluence Tracks)**:
   - **Decreasing Diagrams 軌道**: 基於 van Oostrom 規則標號良基偏序（$\text{Trim} \prec \text{Split} \prec \text{Runtime}$），完全擺脫全局強終止（SN）束縛。
   - **Newman 快速通道**: 當存在作用域/良基階數見證（`SNWitness`）時，僅需局部弱合流（WCR），出具緊湊的 **CPF-KB 短證**（速度提升 10 倍）。

4. **持久化結構共享 AST (Persistent Structural Sharing)**:
   - 基於 `Arc<DiffAstNode>` 的不可變節點共享，文本突變時僅重構變更脊椎，非相交子樹 0 拷貝重用，實測共享率 **95.08%**（超越 $\ge 92.0\%$ 指標）。

5. **五大天然組合深度合成系統 (5-Cluster Unified Pipeline)**:
   - **組合 1【語義幾何與重寫規範化】**: `ast` + `rep_dd` + `tree` + `r0_lower`
   - **組合 2【合流證明與形式化證書工廠】**: `dd_checker` + `tactic_scheduler` + `cpf_cert` + `ari_export` + `rocq_export` + `creusot_export`
   - **組合 3【全化語法解析與雙向編譯器事實橋】**: `parse` + `lex` + `r0` + `polonius_bridge`
   - **組合 4【無損單子逆映射與補丁合成閉環】**: `span` + `span_monad` + `edit` + `patch_engine`
   - **組合 5【污料/變異生成宇宙與自動機驅動】**: `gen` + `shrink` + `rustc_json` + `fuzz_daemon`

---

## 快速運行指令

```sh
# 1. 格式化与 Clippy 零警告检查
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings

# 2. 全量测试套件 (76 项测试 100% 通过)
cargo test --all-targets

# 3. 运行 CI 全量 7 大机械自证门禁
cargo run --bin ci_verify

# 4. 运行 Rocq 9.2 (The Rocq Prover) 形式化理论微内核核检
cargo run --bin rocq_verify

# 5. 运行 Creusot / Why3 + Z3 预言变量与演绎目标 SMT 消解
cargo run --bin creusot_verify

# 6. 运行 79,000+ 组引理海量数据极限压测 (0 Panic)
cargo run --release --bin lemma_stress_coverage

# 7. 运行六大端到端自证门禁
cargo run --bin verify_all

# 8. 运转认证与污料宇宙生成机
cargo run --bin cert_factory

# 9. 运转五大组合端到端深度合成驱动流水线
cargo run --bin pipeline_runner

# 10. 国际 CoCo 压力基准测试
cargo run --release --bin coco_benchmark
```

---

## 發布說明 (Release v0.2.0)

- `[profile.release]`: `opt-level = 3`、`lto = true` (fat LTO)、`codegen-units = 1`、`debug = true`。
- CI 門禁包含格式化、Clippy 0 警告、單元測試、7 大 CI 門禁、Rocq 9.2 微內核核檢與 Creusot 演繹消解。
