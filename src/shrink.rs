//! §8.2 不变量驱动的 AST 语法树剪枝缩小算法 (Delta Shrinker)。
//!
//! 在 Fuzzing 发现反例时，在毫秒级将庞大测试用例精简为 3-5 行最小可重现代码。

use crate::parse::parse;

pub fn shrink_source<F>(mut src: String, predicate: F) -> String
where
    F: Fn(&str) -> bool,
{
    let mut chunk_size = src.len() / 2;
    while chunk_size > 0 {
        let mut offset = 0;
        while offset + chunk_size <= src.len() {
            let mut candidate = src.clone();
            candidate.drain(offset..offset + chunk_size);

            if parse(&candidate).is_ok() && predicate(&candidate) {
                src = candidate;
            } else {
                offset += chunk_size;
            }
        }
        chunk_size /= 2;
    }
    src
}
