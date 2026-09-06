//! `cl0r0-mir` —— 中層 IR(C2)
//!
//! MIR 類型層 + 模組化契約 + 變異性/dropck/UB 預言機。
//! 依賴方向:僅向下依賴 C1(cl0r0-syntax)。

pub use cl0r0_syntax::span; // 令 crate::span 於本 crate 內照常解析

pub mod mir;
pub mod modular_contracts;
pub mod variance_dropck_ub;
