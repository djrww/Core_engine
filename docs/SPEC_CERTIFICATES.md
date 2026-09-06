# SPEC_CERTIFICATES — 證書格式對接與證據分級(DL-012)

> 機密 · © 2026 Ken Yuen Ka Chun. 對應版本:v0.2.0 / 2026-09-06。

## 1 · 三層證據模型(本引擎嘅核心賣點)

所有宣稱按証據強度分級,**如實申報,永不上調**:

| 層 | 名稱 | 証據性質 | 現況 |
|---|---|---|---|
| **T1** | 機內自証 | 引擎對自身執行機械檢查:18 引理、10 門禁、種子確定性屬性套件 | ✅ 全 Proven(CI 每輪) |
| **T2** | 外部核檢 | 第三方證明器獨立核驗:Rocq(rocqchk 微內核)、Creusot/Why3/Z3(VC 消解) | ✅ CI 真裝真跑,0 SKIP |
| **T3** | 草案導出 | Isabelle 理論導出(定理以註釋陳述,完整 Isabelle 證明待形式化) | ⚠️ 誠實標注 DRAFT |

> 投資者/審計要點:本引擎唔宣稱「全部形式化證明」,而係**每一個宣稱均標明
> 証據層級**——呢種分級誠實正係高端市場准入貨幣(seL4/CoCo 同款文化)。

## 2 · CPF 證書(Certified Proof Format)

內部証據載體 `CPFCertificate`(`cl0r0-ars::cpf_cert`),三種證明類型:

| ProofType | 數學依據 | 攜帶見證 |
|---|---|---|
| `DecreasingDiagrams` | van Oostrom 遞減圖 | 標籤清單 `labels` + 嚴格序對 `strict_order_pairs` |
| `KnuthBendixCriticalPairs` | KB + Newman 引理 | SN 見證 `sn_witness` + 臨界對見證清單(個數為**派生量**,不可申報) |
| `OrthogonalLeftLinear` | Rosen 正交性 | 零臨界對系統如實申報途徑 |

- `verify()`:三色 DFS 環檢測(偏序良基性機械檢查)→ `CertResult`;
- `to_cpf_xml()`:出具 CPF XML 交換格式;
- 設計原則(F-03):**一切計數派生、見證實錄、拒絕可捏造申報**。

## 3 · 外部證明器對接(T2)

| 工具 | 用途 | CI 實況 | 對接模組 |
|---|---|---|---|
| **Rocq 9.2**(rocq-core+stdlib) | 理論導出+`rocqchk` 微內核複核 | Proven(「Modules were successfully checked」) | `rocq_export`+`tool_runner` |
| **Why3 + Z3** | Creusot Pearlite 契約 VC 消解 | Proven | `creusot_export`+`proof_resources` |
| Maude(可選) | 重寫引擎差分對照 | 差分通道 | `maude_engine` |
| Ari | Ari 輸出交換 | 導出器 | `ari_export` |

外部工具一律經 `tool_runner` 三態封裝(逾時/缺席/失敗如實申報),
**缺席 ⇒ Skipped,絕不阻塞亦絕不冒充**。

## 4 · 語義邊界(誠實申報)

- 引擎語義模型為 **R₀ 刻意收窄子集**(線性借用/基本控制流/無閉包捕獲等);
- 子集內:合流性宣稱由 T1/T2 全額支撐;
- 子集外(如 two-phase borrows、closures):**未宣稱**——屬路線圖
  (見 VERSION_RECORD §4),唔係現有宣稱一部分;
- UB 預言機為簡化模型(Stacked Borrows 子集),與 Miri 完整模型嘅
  系統化差分對齊屬後續工作。

## 5 · 與國際基準對接

- **CoCo(Confluence Competition)**:`coco_benchmark` bin 已留對接位;
  消費第三方 CPF/TPFA 輸入做獨立複核係下一步(現階段自產自銷屬已知弱點,
  如實申報);
- **TPFA**: Termination and Complexity 系證書格式,對接屬路線圖。
