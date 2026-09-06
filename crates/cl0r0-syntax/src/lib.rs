//! `cl0r0-syntax` —— 語法層載體(C1)
//!
//! 覆蓋 L1/L2(無損回環·決定論)、L3/L4(增量重析)、L7(錯誤全量化)的
//! 物質基礎:字面化 → 解析 → CST/TT → 編輯 → 差分樹。
//! 依賴方向:本 crate 為最底層,零內部依賴(僅 std)。

pub mod ast;
pub mod diff_tree;
pub mod edit;
pub mod gen;
pub mod lex;
pub mod parse;
pub mod span;
pub mod token_tree;
pub mod tree;
