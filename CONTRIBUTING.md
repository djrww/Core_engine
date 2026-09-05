# Core_engine (`cl0r0`) 貢獻指南與唯讀核心專案規範 (Contributing Guidelines)

---

## 1. 唯讀核心專案宣告 (Read-Only Core Engine Policy)

> **【重要宣告 / Notice】**  
> `Core_engine` (`cl0r0`) 是經過高階數學自証與機械形式化認證的**不可變核心專案 (Verified Read-Only Core Engine)**。  
> 本代碼庫承載 CL0/R₀ 雙載體基礎規範、18 大形式化引理自証矩陣、van Oostrom 遞減圖合流定理、Rocq 9.2 微內核核檢與 Creusot 演繹驗證規範。任何對核心引理、公理系統或重寫規則的變更均受到嚴格的數學不變量約束。

---

## 2. 專案維護邊界與貢獻範圍 (Scope & Contribution Boundaries)

### 2.1 允許的貢獻範疇 (Permitted Contributions)
1. **下遊適配器與工具接入**:
   - 新增或優化 MCP (Model Context Protocol) 服務工具 (`mcp_server`)；
   - 擴展 LSP (Language Server Protocol) 前端客戶端插件或 QuickFix 交互界面；
   - 新增定理證明器/求解器導出適配層（如 Lean 4、Coq 8.x 向後兼容層、F* 導出器）。
2. **差分反例與測試增強**:
   - 提交符合 DDMin 1-極小化規範的語法/借用反例用例；
   - 擴展海量污料生成宇宙 (`lemma_stress_generator`) 與 Fuzzing 變異策略。
3. **性能與微架構優化**:
   - 在保持 100% 形式化不變量的前提下，優化 AST 結構共享率（$\ge 92.0\%$）、辨別樹查找或 SMT 消解效率。

### 2.2 禁止的變更行為 (Prohibited Modifications)
- ❌ 弱化或刪除 18 大形式化引理中的任何前置條件（Premises）或目標不變量（Postconditions）；
- ❌ 在全化解析器中引入任何可能導致 Panic 的非受控 `unwrap()` 或 `expect()`；
- ❌ 降低 7 大 CI 門禁、Rocq 9.2 微內核驗證或 Creusot SMT 消解的通過門檻；
- ❌ 引入未經形式化審核的外部 Crate 依賴（本核心引擎堅持 100% 純原生 Rust 零外部依賴標準）。

---

## 3. Pull Request 嚴格驗收門禁 (PR Acceptance Criteria)

所有提交至本倉庫的 Pull Request 必須在本地與 GitHub Actions CI 上達到 **100% 全綠通過**，具體包含以下七大硬性指標：

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                          PR 七大嚴格驗收門禁 (7-Gate CI Checklist)                     │
├───────┬──────────────────────────────┬────────────────────────┬────────────────────────┤
│ 門禁  │ 檢驗指令                     │ 驗收標準               │ 說明                   │
├───────┼──────────────────────────────┼────────────────────────┼────────────────────────┤
│ **1** │ `cargo fmt --all -- --check` │ 0 格式偏差             │ 嚴格對齊 Rustfmt 規範  │
├───────┼──────────────────────────────┼────────────────────────┼────────────────────────┤
│ **2** │ `cargo clippy --all-targets` │ 0 警告 (`-D warnings`) │ 零潛在代碼缺陷         │
├───────┼──────────────────────────────┼────────────────────────┼────────────────────────┤
│ **3** │ `cargo test --all-targets`   │ 96/96 項測試 100% 通過 │ 單元與集成測試全量綠標 │
├───────┼──────────────────────────────┼────────────────────────┼────────────────────────┤
│ **4** │ `cargo run --bin ci_verify`  │ 7 大 CI 門禁 100% 通過 │ 端到端全量自証閉環     │
├───────┼──────────────────────────────┼────────────────────────┼────────────────────────┤
│ **5** │ `cargo run --bin rocq_verify`│ `rocqchk` 雙重核驗通過 │ Rocq 9.2 微內核自証    │
├───────┼──────────────────────────────┼────────────────────────┼────────────────────────┤
│ **6** │ `cargo run --bin creusot_verify`│ 11/11 VCs 100% Valid│ Why3 + Z3 演繹消解     │
├───────┼──────────────────────────────┼────────────────────────┼────────────────────────┤
│ **7** │ `cargo run --release --bin lemma_stress_coverage` │ 79,000 樣本 100% 通過 | 0-Panic 極限壓測       │
└───────┴──────────────────────────────┴────────────────────────┴────────────────────────┘
```

---

## 4. 本地開發與自証檢驗工作流 (Local Verification Workflow)

在提交變更前，請在本地執行完整的自檢管線：

```sh
# 1. 代碼風格與靜態分析
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings

# 2. 全量單元與集成測試套件
cargo test --all-targets

# 3. 執行 7 大 CI 專項門禁
cargo run --bin ci_verify

# 4. 執行 Rocq 9.2 形式化微內核自証
cargo run --bin rocq_verify

# 5. 執行 Creusot / Why3 + Z3 演繹驗證
cargo run --bin creusot_verify

# 6. 執行 79,000 組引理極限壓測
cargo run --release --bin lemma_stress_coverage

# 6b. 執行巨集七原則與借用組合模型證據鏈(14 門禁,純機內)
cargo run --bin macro_lab

# 7. 執行十項端到端機核檢門禁(外部證明器缺席時如實報 SKIPPED)
cargo run --bin verify_all
```

---

## 5. 差分反例提交規範 (Differential Bug Report Guidelines)

若在實踐中發現形式化定理或解析器行為與預期不符，請按以下規範提交 Issue：

1. **反例最小化 (DDMin)**:
   - 提交前請先通過 `src/shrink.rs` 提供的 Delta Debugging 算法將反例裁剪至 1-極小反例；
2. **附帶數學見證**:
   - 標明觸發異常的引理標識（如 L1, L5, L8, L16 等）；
   - 提供對應的 AST 節點、MIR 控制流片段或 Polonius 事實關係；
3. **不可變回歸保護**:
   - 每個修復的反例均會被收錄至 `tests/sota_verification.rs` 作為永久回歸測試用例。
