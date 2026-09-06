// Copyright (c) 2026 Ken Yuen Ka Chun. All rights reserved.
// PROPRIETARY & CONFIDENTIAL — unauthorized copying, modification, distribution, or reverse engineering is prohibited.
//! cpf_check —— 第三方 CPF 証書獨立複核入口(DL-016)。
//!
//! 用法:
//! * `cpf_check`(無參數):以內嵌第三方風格樣本自演示;
//! * `cpf_check <path>`:讀取並獨立複核指定 CPF(XML)檔。
//!
//! 退出碼(四態如實申報):
//! * 0 = VERIFIED(獨立複核通過)
//! * 3 = UNSUPPORTED(証明類型不在消費子集)
//! * 4 = REJECTED(獨立複核否決)
//! * 5 = MALFORMED(結構不良)
//! * 6 = 檔案讀取失敗

use cl0r0::cpf_import::{import_and_verify, CpfImportVerdict};

/// 內嵌演示樣本:第三方(例如競賽參賽者)提交嘅 DD 合流証書。
const THIRD_PARTY_SAMPLE: &str = r#"<certificationProblem>
  <systemId>cofesco_2026_entry_17</systemId>
  <input><!-- 第三方 TRS;消費方不信任內容,只重放見證 --></input>
  <cpf:proof>
    <crProof>
      <decreasingDiagrams>
        <labels><name>Trim</name><name>Split</name><name>Runtime</name></labels>
        <strict>
          <pair><arg>Split</arg><arg>Trim</arg></pair>
        </strict>
      </decreasingDiagrams>
    </crProof>
  </cpf:proof>
</certificationProblem>"#;

fn main() {
    let arg = std::env::args().nth(1);
    let (xml, origin) = match &arg {
        None => (THIRD_PARTY_SAMPLE.to_string(), "內嵌演示樣本"),
        Some(path) => match std::fs::read_to_string(path) {
            Ok(s) => (s, path.as_str()),
            Err(e) => {
                println!("[cpf_check] 讀取 {path} 失敗:{e}");
                std::process::exit(6);
            }
        },
    };

    println!("======================================================================");
    println!("[cpf_check] 第三方 CPF 獨立複核(來源:{origin})");
    println!("======================================================================");
    let verdict = import_and_verify(&xml);
    println!("[複核判定]:{verdict}");
    match verdict {
        CpfImportVerdict::Verified { .. } => {
            println!("[結論]:見證重放通過,接受該合流宣稱(獨立複核)。");
            std::process::exit(0);
        }
        CpfImportVerdict::Unsupported { .. } => {
            println!("[結論]:類型不在消費子集,如實申報不處理。");
            std::process::exit(3);
        }
        CpfImportVerdict::Rejected { .. } => {
            println!("[結論]:獨立複核否決(見證重放失敗)。");
            std::process::exit(4);
        }
        CpfImportVerdict::Malformed { .. } => {
            println!("[結論]:結構不良,拒收。");
            std::process::exit(5);
        }
    }
}
