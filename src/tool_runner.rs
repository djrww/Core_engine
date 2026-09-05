//! 外部證明工具子進程封裝(審計 F-09 / F-12 / D-03 單一真相)。
//!
//! 過去 `rocq_export` 與 `creusot_export` 各自維護一套同構的
//! 「路徑探測 → 臨時寫檔 → Command::output → 計時 → 清理」骨架(約 140 行 ×2),
//! 且臨時檔寫入共享 TMPDIR 的**固定檔名**({module}.v / .mlw),並發進程
//! 互相踐踏、符號鏈接可劫持寫入目標。本模組統一提供:
//!
//! 1. `find_binary` —— 依 `{TOOL}_HOME` 環境變量 → `PATH` 逐項 → 顯式
//!    fallback 目錄的順序探測可執行檔(取代寫死的開發者機器路徑);
//! 2. `WorkDir` —— `cl0r0-{tag}-{pid}-{nonce}` 進程級隔離工作目錄,
//!    Drop 時 `remove_dir_all`,清理失敗**不再靜默**(警告入 log / stderr);
//! 3. 單調 nonce —— 同進程內兩次並發驗證同名模組亦不碰撞。

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static NONCE: AtomicU64 = AtomicU64::new(0);

fn next_nonce() -> u64 {
    NONCE.fetch_add(1, Ordering::Relaxed)
}

/// 在候選路徑上探測一個能成功回應 `--version` 的可執行檔。
fn probe(path: &str, args: &[&str]) -> bool {
    Command::new(path)
        .args(args)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// 依優先級探測外部工具:
/// 1. `{home_env}` 環境變量指向的目錄(及其 `bin/` 子目錄);
/// 2. `PATH` 中的每一項;
/// 3. 顯式 fallback(如 `$HOME/.opam/rocq/bin`,由調用方決定語義)。
pub fn find_binary(
    bin: &str,
    home_env: Option<&str>,
    fallback_subdirs: &[PathBuf],
) -> Option<String> {
    // 1. 環境變量指定的安裝根
    if let Some(env_key) = home_env {
        if let Ok(home) = std::env::var(env_key) {
            let home = std::path::Path::new(&home);
            for dir in [home.to_path_buf(), home.join("bin")] {
                let cand = dir.join(bin);
                if let Some(s) = cand.to_str() {
                    if probe(s, &["--version"]) {
                        return Some(s.to_string());
                    }
                }
            }
        }
    }

    // 2. PATH 逐項
    if let Ok(paths) = std::env::var("PATH") {
        for dir in std::env::split_paths(&paths) {
            let cand = dir.join(bin);
            if let Some(s) = cand.to_str() {
                if probe(s, &["--version"]) {
                    return Some(s.to_string());
                }
            }
        }
    }

    // 3. 顯式 fallback(如 $HOME/.opam/rocq/bin)
    for dir in fallback_subdirs {
        let cand = dir.join(bin);
        if let Some(s) = cand.to_str() {
            if probe(s, &["--version"]) {
                return Some(s.to_string());
            }
        }
    }
    None
}

/// opam 預設安裝根(從 `$HOME` 派生,不再寫死任何特定開發者路徑)。
pub fn opam_default_root(tool: &str) -> PathBuf {
    std::env::var("HOME")
        .map(|h| PathBuf::from(h).join(".opam").join(tool).join("bin"))
        .unwrap_or_else(|_| PathBuf::from("/nonexistent"))
}

/// 進程級隔離的臨時工作目錄(F-09:取代共享 TMPDIR 固定檔名)。
///
/// 目錄名:`cl0r0-{tag}-{pid}-{nonce}`,同名模組的並發驗證互不干擾;
/// `Drop` 時整目錄刪除,失敗時向 stderr 發警告(不靜默)。
pub struct WorkDir {
    path: PathBuf,
    pub warnings: Vec<String>,
}

impl WorkDir {
    pub fn new(tag: &str) -> std::io::Result<Self> {
        let path = std::env::temp_dir().join(format!(
            "cl0r0-{}-{}-{}",
            tag,
            std::process::id(),
            next_nonce()
        ));
        std::fs::create_dir_all(&path)?;
        Ok(WorkDir {
            path,
            warnings: Vec::new(),
        })
    }

    pub fn path(&self) -> &std::path::Path {
        &self.path
    }

    /// 在本工作目錄內執行命令(產物落在隔離目錄,不污染共享 TMPDIR)。
    pub fn run(&mut self, bin: &str, args: &[&str]) -> std::io::Result<std::process::Output> {
        Command::new(bin)
            .args(args)
            .current_dir(&self.path)
            .output()
    }

    /// 顯式清理(返回錯誤供調用方記錄;Drop 仍會兜底)。
    pub fn cleanup(&mut self) -> std::io::Result<()> {
        std::fs::remove_dir_all(&self.path)
    }
}

impl Drop for WorkDir {
    fn drop(&mut self) {
        if self.path.exists() {
            if let Err(e) = std::fs::remove_dir_all(&self.path) {
                // F-09:清理失敗不再 let _ 吞掉 —— 至少留下痕跡
                let msg = format!(
                    "[tool_runner] cleanup failed for {}: {} (stale artifacts may remain)",
                    self.path.display(),
                    e
                );
                self.warnings.push(msg.clone());
                eprintln!("{}", msg);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn work_dirs_are_unique_per_call() {
        let a = WorkDir::new("ut").unwrap();
        let b = WorkDir::new("ut").unwrap();
        assert_ne!(
            a.path(),
            b.path(),
            "concurrent same-name modules must not collide"
        );
        let name = a.path().file_name().unwrap().to_string_lossy().to_string();
        assert!(name.starts_with("cl0r0-ut-"), "dir name = {name}");
        assert!(name.contains(&format!("-{}", std::process::id())));
        assert!(a.path().is_dir());
    }

    #[test]
    fn work_dir_cleans_up_on_drop() {
        let p = {
            let w = WorkDir::new("ut-drop").unwrap();
            std::fs::write(w.path().join("x.v"), "x").unwrap();
            w.path().to_path_buf()
        };
        assert!(!p.exists(), "WorkDir must remove its directory on drop");
    }

    #[test]
    fn find_binary_returns_none_for_nonexistent_tool() {
        assert!(find_binary("definitely-not-a-real-tool-xyz", None, &[]).is_none());
    }
}
