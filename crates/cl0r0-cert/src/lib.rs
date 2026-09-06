//! `cl0r0-cert` —— 證書與外部證明層(C4)
//!
//! CPF 證書類型消費、Rocq/Isabelle/Ari/Creusot 導出、外部工具子進程封裝。
//! 依賴方向:向下依賴 C3(cl0r0-ars)與 C2(cl0r0-mir)。

pub use cl0r0_ars::cpf_cert; // 令 crate::cpf_cert 於本 crate 內照常解析
pub use cl0r0_mir::mir; // 令 crate::mir 於本 crate 內照常解析

pub mod ari_export;
pub mod creusot_export;
pub mod isabelle_export;
pub mod proof_resources;
pub mod rocq_export;
pub mod tool_runner;
