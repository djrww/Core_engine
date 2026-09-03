//! cl0r0_lsp —— 獨立 Language Server Protocol (LSP) 二進制服務進程。
//!
//! 通過 stdio JSON-RPC 2.0 協議與 VS Code、Rust-Rover、Neovim 等 IDE 通信，
//! 實時提供借用衝突檢測、Newman/DD 形式化數學證明解釋與一鍵合流修法 (CodeAction QuickFix)。
//!
//! 運行: `cargo run --bin cl0r0_lsp`

use cl0r0::lsp_bridge::LspEngine;
use std::io::{self, BufRead, Read, Write};

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let mut stdin_lock = stdin.lock();
    let mut stdout = io::stdout();

    let mut line = String::new();
    while stdin_lock.read_line(&mut line)? > 0 {
        if line.starts_with("Content-Length:") {
            let len_str = line.trim_start_matches("Content-Length:").trim();
            if let Ok(content_len) = len_str.parse::<usize>() {
                // 讀取接下來的空行 (\r\n)
                line.clear();
                stdin_lock.read_line(&mut line)?;

                // 讀取 JSON payload
                let mut body_buf = vec![0u8; content_len];
                stdin_lock.read_exact(&mut body_buf)?;

                if let Ok(req_str) = String::from_utf8(body_buf) {
                    if let Some(resp_str) = LspEngine::process_json_rpc(&req_str) {
                        let resp_bytes = resp_str.as_bytes();
                        write!(
                            stdout,
                            "Content-Length: {}\r\n\r\n{}",
                            resp_bytes.len(),
                            resp_str
                        )?;
                        stdout.flush()?;
                    }
                    if req_str.contains("\"method\":\"exit\"") {
                        break;
                    }
                }
            }
        }
        line.clear();
    }

    Ok(())
}
