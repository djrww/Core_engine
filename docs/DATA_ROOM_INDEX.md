# DATA ROOM INDEX — 內部規格文書包總目(DL-012)

> **機密等級**:機密(Proprietary & Confidential)——僅供獲授權第三方(投資者盡職調查、
> 潛在被授權方、審計方)閱覽,不得複製或外傳。閱畢請交還或銷毀。
> **版本**: v1.0 · 2026-09-06 · 對應代碼版本 v0.2.0(Atlas v0.3.0)
> **版權**: © 2026 Ken Yuen Ka Chun. All rights reserved.

## 快速導讀(按審閱目的選檔)

| 你想知… | 睇邊份 | 一句概括 |
|---|---|---|
| 呢個項目係咩、有幾可靠 | `QUALITY_EVIDENCE.md` | 200 測試、84.62% 覆蓋、10 門禁、0 SKIP |
| 系統點砌出嚟 | `SPEC_ARCHITECTURE.md` | 五 crate 分層,依賴單向,51 模組 |
| 同 IDE/CLI 點接 | `SPEC_INTERFACE.md` | LSP 標準協議+17 個自証 bin |
> **系統點証明嘢** | `SPEC_CERTIFICATES.md` | 三層證據分級(T1/T2/T3),如實申報 |
| **開發軌跡同行為紀錄** | `VERSION_RECORD.md` | DL-000..012 全程可審計 |
| **法律狀態** | 根目錄 `LICENSE`/`THIRD_PARTY_NOTICES.md` | 專有 EULA;運行時零第三方依賴 |

## 文書包結構

```
docs/
├── DATA_ROOM_INDEX.md      ← 本檔(總目+導讀)
├── SPEC_ARCHITECTURE.md    ← 架構規格(五 crate/依賴方向/分層)
├── SPEC_INTERFACE.md       ← 介面規格(LSP/CLI/lib API)
├── SPEC_CERTIFICATES.md    ← 證書格式對接(CPF/Rocq/Isabelle/Creusot)+證據分級
├── VERSION_RECORD.md       ← 版本紀錄(commit/CI/覆蓋率演進)
└── QUALITY_EVIDENCE.md     ← 質量証據匯編(測試/門禁/捉 bug 紀錄)
```

## 深度參考(文書包引導向根目錄既有文檔,不重複)

| 主題 | 檔案 |
|---|---|
| 全景架構圖(mermaid)與設計規範 | `ARCHITECTURE.md` |
| 功能圖鑑(67 項功能/語義/覆蓋率) | `FEATURE_ATLAS.md` |
| 架構決策流水簿 | `ADR.md` |
| 開發閉環章程 | `DEV_LOOP.md` + `BACKLOG.md` |
| 覆蓋率與差分審核詳報 | `COVERAGE_REPORT.md` |
| 證明工具鏈指南 | `PROOF_TOOLCHAIN_GUIDE.md` |
| 巨集七原則 | `MACRO_SEVEN_PRINCIPLES.md` |

## 本項目一句話

**CL0/R0 合流引擎**:以抽象重寫系統(ARS)嘅合流性定理(DD/Newman/KB)為數學基礎,
對 Rust 借用衝突做「可修補 ⇔ 可會合」判定,並出具分級機器證書嘅靜態分析內核。
