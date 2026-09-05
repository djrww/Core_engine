# CL0/R₀ 開發階段形式化引理取証工具指南 (Proof & Certification Toolchain Guide)

本指南面向 `cl0r0` 核心驗證引擎的開發者，闡述如何在日常開發階段（Development Phase）高效調用與集成**形式化引理取証工具鏈**，以機器可讀、可復現的證物（Witness Artifacts）為 18 大核心引理出具權威自証。

---

## 一、 開發階段取証工具架構與職能分工

```
+-----------------------------------------------------------------------------------------+
|                               開發階段引理取証工具體系                                   |
+-----------------------------------------------------------------------------------------+
| 1. 快速性質自証 (Proptest + In-Memory Witness)                                          |
|    - 覆蓋引理: L1 (無損回環), L2 (決定論), L5 (Laminarity & χ=1), L18 (結構共享 >=92%)   |
|    - 產出資產: 內存結構指紋、SHA-256 吻合驗證、歐拉示性數斷言                          |
+-----------------------------------------------------------------------------------------+
| 2. 專用項重寫合流取証器 (CeTA 3.7.1 + CPF 格式 XML)                                    |
|    - 覆蓋引理: L8 (良基遞減), L9 (遞減圖合流), L10 (Knuth-Bendix 關鍵對)               |
|    - 產出資產: `proofs/cl0_dd_confluence.cpf`, `proofs/cl0_kb_confluence.cpf`            |
+-----------------------------------------------------------------------------------------+
| 3. 演繹式驗證與 SMT 放電 (Creusot / Pearlite + Why3 / Z3)                               |
|    - 覆蓋引理: L11 (弦圖完美圖), L12 (Span Monad 雙射), L16 (Reborrow 懸掛), L17 (Aeneas) |
|    - 產出資產: `theories/CL0_Creusot.mlw` (包含 Pre/Post 條件與 Prophecies 預言合約)    |
+-----------------------------------------------------------------------------------------+
| 4. 符號執行與極小反例見證 (Kani / CBMC + DDMin 演算法)                                  |
|    - 覆蓋引理: L7 (0-Panic 全函數性), L13 (Zeller DDMin 1-極小性收斂)                  |
|    - 產出資產: 1500+ 極端污料 0-Panic 健全性見證、最小錯誤代碼片段                      |
+-----------------------------------------------------------------------------------------+
| 5. 交互式定理證明與微內核核檢 (Rocq 9.2 / Coq + Polonius Datalog)                       |
|    - 覆蓋引理: L14 (Polonius Datalog 不動點等價), L15 (MIR Def-Use 活躍期)              |
|    - 產出資產: `theories/CL0_Theories.v`, `proofs/polonius_facts.dl`                    |
+-----------------------------------------------------------------------------------------+
```

---

## 二、 開發者一鍵取証命令 (Developer One-Shot Workflow)

在開發或重構任何編譯器/重寫模組時，可隨時執行以下命令驗證並導出最新形式化証物：

### 1. 執行端到端引理取証與証物打包 (推薦日常使用)
```bash
cargo run --bin dev_prover
```
*   **執行耗時**：$\approx 15\sim 30\text{ ms}$
*   **產出結果**：
    1. 在控制台實時輸出 18 大引理自証矩陣與 Witness 摘要；
    2. 自動導出 CeTA 格式 XML 證書至 `proofs/*.cpf`；
    3. 自動生成 Creusot 演繹理論至 `theories/CL0_Creusot.mlw`；
    4. 自動生成 Rocq 9.2 定理證明理論至 `theories/CL0_Theories.v`；
    5. 自動導出 Polonius Datalog 事實庫至 `proofs/polonius_facts.dl`；
    6. 將全量証物與校驗結果匯總至 `proofs/lemma_evidence_bundle.json`。

### 2. 六大 CI 門禁機械自証檢驗
```bash
cargo run --bin verify_all
```
*   嚴格檢驗六大門禁（語法回環、增量重析、遞減圖合流、Newman 快速通道、原生 CPF 證書、全化 0-Panic）。

### 3. 批量證書生產與污料宇宙容錯測試
```bash
cargo run --bin cert_factory
```
*   批量生成 100+ 份隨機 AST 結構、CPF 證書與極端破壞性污料樣本。

---

## 三、 生成証物規範與外部工具驗證

### 1. 使用 CeTA (IsaFoR) 獨立核驗 CPF XML 證書
若本地安裝有 `ceta` 可執行檔（基於 Isabelle/HOL 導出）：
```bash
ceta proofs/cl0_dd_confluence.cpf
# 預期輸出: CERTIFIED
ceta proofs/cl0_kb_confluence.cpf
# 預期輸出: CERTIFIED
```

### 2. 使用 Why3 + Z3 演繹驗證 Creusot 理論
```bash
why3 prove -P z3 theories/CL0_Creusot.mlw
# 預期輸出: 100% Valid (All goals discharged)
```

### 3. 使用 Rocq 9.2 (Coq) 微內核獨立驗證
```bash
rocq compile theories/CL0_Theories.v
rocqchk -silent CL0_Theories
# 預期輸出: 0 Axiom Violations, Kernel Accepted
```

---

## 四、 引理証物包結構 (`proofs/lemma_evidence_bundle.json`)

取証流水線生成的 JSON 証物包符合以下機器可讀結構：

```json
{
  "project": "cl0r0",
  "version": "0.2.0",
  "timestamp_utc": "2026-09-05T01:40:00Z",
  "status": "ALL_CERTIFIED",
  "certified_witnesses_count": 25,
  "lemmas": [
    {
      "id": "L1",
      "title": "無損回環引理 (Lossless Roundtrip Lemma)",
      "certified": true,
      "summary": "Verified on 4 representative CST cases: 100% byte-for-byte lossless roundtrip."
    },
    ...
  ],
  "artifacts": [
    "proofs/cl0_dd_confluence.cpf",
    "proofs/cl0_kb_confluence.cpf",
    "theories/CL0_Creusot.mlw",
    "theories/CL0_Theories.v",
    "proofs/polonius_facts.dl"
  ]
}
```

---

## 五、 開發階段常見反例除錯指南

1. **若 L1 (無損回環) 失敗**：
   - 檢查 `lex.rs` 是否遺漏了某些新型 Unicode 空白字符或註解結尾的換行符。
2. **若 L5 (Laminarity) 失敗**：
   - 檢查 `parse.rs` 中是否有節點的 Span 計算跨越了其父節點邊界，導致區間重疊。
3. **若 L9 (遞減圖合流) 失敗**：
   - 檢查 `rep_dd.rs` 中新加入的重寫規則是否打破了標籤偏序無環性（Cyclic Label Dependency）。
4. **若 L18 (結構共享率) 低於 92%**：
   - 檢查 `diff_tree.rs` 中是否在修訂局部節點時過度深拷貝了未受影響的兄弟子樹。
