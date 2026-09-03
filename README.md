# Core_engine (`cl0r0`) v0.2.0

CL0/R₀ 雙載體的機械自証代碼庫：表面語法樹、九條定律（L1–L9）、van Oostrom 遞減圖（Decreasing Diagrams）、Newman 快速通道（SN ∧ WCR ⇒ CR）、原生 CPF 證書核驗、五大天然組合深度合成閉環、污料宇宙生成機與 Polonius 雙向事實橋接。

---

## 核心特徵 (Key Features)

1. **雙載體體系 (Dual Carrier)**:
   - **CL0 定律載體**: 嚴格驗證九條基本結構與重寫定律（L1–L9），保證無損回環與幾何 Laminarity。
   - **R₀ 實用載體**: 面向 Rust 實用子集（附錄 B EBNF），包含 LALR(1) 乾淨子集與 `unsupported` 邊界防火牆。

2. **合流性雙通道 (Dual Confluence Tracks)**:
   - **Decreasing Diagrams 軌道**: 基於 van Oostrom 規則標號良基偏序（$\text{Trim} \prec \text{Split} \prec \text{Runtime}$），完全擺脫全局強終止（SN）束縛。
   - **Newman 快速通道**: 當存在作用域/良基階數見證（`SNWitness`）時，僅需局部弱合流（WCR），出具緊湊的 **CPF-KB 短證**（速度提升 10 倍）。

3. **五大天然組合深度合成系統 (5-Cluster Unified Pipeline)**:
   - **組合 1【語義幾何與重寫規範化】**: `ast` + `rep_dd` + `tree` + `r0_lower`
   - **組合 2【合流證明與形式化證書工廠】**: `dd_checker` + `tactic_scheduler` + `cpf_cert` + `ari_export`
   - **組合 3【全化語法解析與雙向編譯器事實橋】**: `parse` + `lex` + `r0` + `polonius_bridge`
   - **組合 4【無損單子逆映射與補丁合成閉環】**: `span` + `span_monad` + `edit` + `patch_engine`
   - **組合 5【污料/變異生成宇宙與自動機驅動】**: `gen` + `shrink` + `rustc_json` + `fuzz_daemon`

4. **形式化證書工廠與污料宇宙 (Cert & Dirty Data Factory)**:
   - 內置 `cert_generator_factory` 與 `cert_factory` 運行機：高熵髒輸入流、截斷代碼段、越界語法突變。
   - 自動出具 CPF-KB、CPF-DD、ARI-COPS 與 CeTA 3.7.1 兼容之 XML / S-expr 形式化證書。

5. **Polonius 官方事實橋接與 LSP 修法**:
   - 雙向轉換 Rust 官方 `-Zpolonius` Datalog 事實（`loan_issued_at`, `borrow_live_at`, `invalidates`）。
   - 提供符合 Language Server Protocol (LSP) 的 QuickFix 修法建議與形式化數學解釋。

6. **4 小時自動化 Fuzzing 流水線**:
   - GitHub Actions 每 4 小時定時觸發（`0 */4 * * *`）。
   - 本地常駐守護進程 `fuzz_daemon`，配備不變量引導的 AST 最小反例縮減器（Delta Shrinker）。

---

## 六大檢測門 (The Six Verification Gates)

代碼庫集成六大端到端機械自証門禁（一鍵全綠驗證）：

```sh
cargo run --bin verify_all
```

| 門禁 | 驗證內容 | 理論依據 |
| --- | --- | --- |
| **Gate 1** | L1 無損回環 / L2 決定論 / L5 Laminar 樹公理 | $\text{unparse} \circ \text{parse} \equiv \text{id}$, CW 複形 $\chi = 1$ |
| **Gate 2** | L3 / L4 增量重析配置快照重用等價性 | $\text{parse}(E(s)) \equiv \text{reparse}(s, t, E)$ |
| **Gate 3** | L8 / L9 van Oostrom 遞減圖合流性自証 | $\forall \text{ local peak}, \text{Decreasing Valley exists}$ |
| **Gate 4** | Newman 快速通道 (SN ∧ WCR ⇒ CR) 與 KB 短證 | $SN \land WCR \Rightarrow CR$（10 倍加速） |
| **Gate 5** | 原生 CPF 證書偏序無環與 Knuth-Bendix 短證核驗 | IsaFoR / CeTA 3.7.1 標準規範 |
| **Gate 6** | 全化解析器 0 Panic 與 Tactic Scheduler 策略調度 | 2000 隨機髒輸入全化成樹 ∧ 策略最優命中 |

---

## 模組架構

| 模塊 | 職責 |
| --- | --- |
| `span` | $\sigma(v) = [a, b)$ 半開區間 (§1.2) |
| `lex` | CL0 詞法器 —— DFA，平鋪全源碼，trivia 保留 (§5.1) |
| `parse` | 表面語法樹 + 增量重析 + ERROR 全化 (§1–§2.3) |
| `tree` | 樹的性質檢查（連續性公理、laminar、CW 複形） |
| `edit` | 編輯單體（位移函數、複合、結合律） |
| `ast` | 語義面 —— liveness 三軌、衝突圖（完美圖、弦圖） |
| `rep` | 經典修法菜單重寫系統 (§4) |
| `rep_dd` | van Oostrom 遞減圖 (Decreasing Diagrams) ARS 模組與 $\alpha$-同構 |
| `dd_checker` | 遞減圖 / Newman 雙模式局部峰值機械核驗器 |
| `cpf_cert` | 原生輕量級 CPF 證書生成與核驗器（KB 短證 + DD 偏序證） |
| `tactic_scheduler` | 重寫策略調度器與 ARI-COPS 競賽對拍層 |
| `pipeline_synthesis` | 五大核心組合深度合成引擎 (1+2+3+4+5 閉環) |
| `cert_generator_factory` | 污料生成宇宙、形式化證書工廠與認證流水線 |
| `span_monad` | AST 節點與事實層區間的雙向單子逆映射 (保持 L1/L6) |
| `patch_engine` | 端到端補丁合成與閉環自驗引擎 |
| `r0` | R₀ Rust 子集 —— 附錄 B 覆蓋面契約 + unsupported 申報 |
| `r0_lower` | R₀ 實用載體語義降階與 Def-Use 活躍期分析 |
| `reparse_verifier` | L3/L4 增量重析等價性核驗器 |
| `rustc_json` | 強制 `.json` 格式報錯解析與自動機套接 |
| `polonius_bridge` | Polonius 官方 Datalog 關係事實雙向橋接 |
| `lsp_bridge` | Language Server Protocol 交互式修法橋接 (附 Newman/DD 解釋) |
| `shrink` | 不變量驅動的語法樹 Delta 剪枝縮減算法 |
| `gen` | 合法程式 + 髒輸入生成器 |
| `l9newman` | 機械的 Newman 通道 |

---

## 快速運行

```sh
# 格式化与 Clippy 零警告检查
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings

# 全量测试套件 (含 15 个集成测试)
cargo test --all-targets

# 机械自证全量流水线 (六大门禁)
cargo run --bin verify_all

# 运转认证与污料宇宙生成机
cargo run --bin cert_factory

# 运转五大组合端到端深度合成驱动流水线
cargo run --bin pipeline_runner

# 递减图与 CPF 独立核验
cargo run --bin dd_verify

# 国际 CoCo 压力基准测试
cargo run --release --bin coco_benchmark

# 4 小时 Fuzzing 守护进程 (单轮演示)
cargo run --release --bin fuzz_daemon -- --single-round

# 生成完整文档
cargo doc --all-features --no-deps --open
```

---

## 發布說明 (Release v0.2.0)

- `[profile.release]`: `opt-level = 3`、`lto = true` (fat LTO)、`codegen-units = 1`、`debug = true`。
- CI: push 標籤 `v*` $\to$ 校驗版本 $\to$ `cargo build --release` $\to$ 打包全量二進制 (`cert_factory`, `cl0r0`, `coco_benchmark`, `dd_verify`, `fuzz`, `fuzz_daemon`, `l9newman`, `pipeline_runner`, `verify_all`) $\to$ 建立 GitHub Release。
