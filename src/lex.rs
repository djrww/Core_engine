//! CL0 詞法層(§5.1 詞法層:DFA——CL0 詞法完全正則)。
//!
//! # Input model
//!
//! * [`lex`] takes a Rust `&str` — **valid UTF-8**. Token spans always fall on
//!   Unicode scalar-value (char) boundaries; a multibyte character is never
//!   split across tokens.
//! * [`lex_bytes`] accepts arbitrary bytes. Valid UTF-8 scalar values that are
//!   not part of the CL0 alphabet become a single `Bad` token covering the
//!   whole sequence; ill-formed UTF-8 produces one `Bad` token per maximal
//!   ill-formed subsequence (as defined by `str::from_utf8` error lengths),
//!   never splitting a well-formed multibyte character.
//!
//! 不變量(詞法不變量,由測試 `lex_lexical_invariants` 機械驗證):
//!   * **平鋪(tiling)**:`lex(s)` 的 token 序列逐字節覆蓋 `s`,無縫隙、無重疊、
//!     無遺漏:∀i: tᵢ.end == tᵢ₊₁.start,且 t₀.start == 0、t_last.end == s.len()。
//!     這是 L1(無損回環)的字節級前提。
//!   * 空白與 `//` 註釋一律是 trivia token,參與樹的序列化與回環(L1),
//!     因此「重寫不重排用戶代碼」在 CL0 上可機械驗證。
//!   * 任何非法字字節也會得到一個 `Bad` token(長度為該字元的 utf8 長度),
//!     所以詞法器是**全函數**:對任何輸入都返回完整平鋪。
//!   * 關鍵字是保留字:fn / let / mut / if / else / while / true / false。

use crate::span::Span;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum TokKind {
    Ident,
    Number,
    True,
    False,
    Fn,
    Let,
    Mut,
    If,
    Else,
    While,
    Amp,  // &
    Star, // *
    Plus,
    Minus,
    EqEq,
    Lt,
    Eq, // =
    LParen,
    RParen,
    LBrace,
    RBrace,
    Semi,
    Colon,
    Comma,
    Trivia,
    Bad, // 詞法錯誤字元(仍佔一個平鋪 token)
}

impl TokKind {
    pub fn is_struct(self) -> bool {
        self != TokKind::Trivia
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Token {
    pub kind: TokKind,
    pub span: Span,
}

const TRANSLIT: &str = "fnletmutifelsewhiletruefalse";

fn keyword_of(word: &str) -> Option<TokKind> {
    match word {
        "fn" => Some(TokKind::Fn),
        "let" => Some(TokKind::Let),
        "mut" => Some(TokKind::Mut),
        "if" => Some(TokKind::If),
        "else" => Some(TokKind::Else),
        "while" => Some(TokKind::While),
        "true" => Some(TokKind::True),
        "false" => Some(TokKind::False),
        _ => None,
    }
}

/// Length of the next UTF-8 scalar value starting at `i`, or the length of the
/// maximal ill-formed subsequence if the bytes are not valid UTF-8.
fn utf8_token_len(bytes: &[u8], i: usize) -> usize {
    debug_assert!(i < bytes.len());
    match std::str::from_utf8(&bytes[i..]) {
        Ok(s) => s.chars().next().map(|c| c.len_utf8()).unwrap_or(1),
        Err(e) => {
            if e.valid_up_to() > 0 {
                // Should not happen when `i` is already at an error boundary,
                // but stay defensive: consume the valid prefix char.
                let valid = std::str::from_utf8(&bytes[i..i + e.valid_up_to()]).unwrap();
                valid.chars().next().map(|c| c.len_utf8()).unwrap_or(1)
            } else {
                e.error_len().unwrap_or(bytes.len() - i)
            }
        }
    }
}

fn push_tok(toks: &mut Vec<Token>, kind: TokKind, start: usize, end: usize) {
    toks.push(Token {
        kind,
        span: Span::new(start as u32, end as u32),
    });
}

/// DFA 詞法 over valid UTF-8 (`&str`). Never splits a Unicode scalar value.
pub fn lex(src: &str) -> Vec<Token> {
    lex_bytes(src.as_bytes())
}

/// DFA 詞法 over arbitrary bytes. Tiling invariant holds at the byte level;
/// well-formed multibyte UTF-8 characters are never split across tokens.
pub fn lex_bytes(bytes: &[u8]) -> Vec<Token> {
    let mut toks = Vec::new();
    let mut i = 0usize;
    while i < bytes.len() {
        let start = i;
        let b = bytes[i];
        match b {
            // trivia:空白
            b' ' | b'\t' | b'\r' | b'\n' => {
                while i < bytes.len() && matches!(bytes[i], b' ' | b'\t' | b'\r' | b'\n') {
                    i += 1;
                }
                push_tok(&mut toks, TokKind::Trivia, start, i);
            }
            // trivia:// 註釋(至行尾,含換行前的內容;不含換行本身時仍平鋪)
            b'/' if i + 1 < bytes.len() && bytes[i + 1] == b'/' => {
                // Advance by UTF-8 scalar / ill-formed chunks so comment text
                // never leaves `i` mid-character before the newline.
                i += 2;
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += utf8_token_len(bytes, i);
                }
                push_tok(&mut toks, TokKind::Trivia, start, i);
            }
            b'0'..=b'9' => {
                while i < bytes.len() && bytes[i].is_ascii_digit() {
                    i += 1;
                }
                push_tok(&mut toks, TokKind::Number, start, i);
            }
            b'A'..=b'Z' | b'a'..=b'z' | b'_' => {
                while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                    i += 1;
                }
                let word = std::str::from_utf8(&bytes[start..i]).unwrap_or("");
                push_tok(
                    &mut toks,
                    keyword_of(word).unwrap_or(TokKind::Ident),
                    start,
                    i,
                );
            }
            b'&' => {
                i += 1;
                push_tok(&mut toks, TokKind::Amp, start, i);
            }
            b'*' => {
                i += 1;
                push_tok(&mut toks, TokKind::Star, start, i);
            }
            b'+' => {
                i += 1;
                push_tok(&mut toks, TokKind::Plus, start, i);
            }
            b'-' => {
                i += 1;
                push_tok(&mut toks, TokKind::Minus, start, i);
            }
            b'=' => {
                i += 1;
                let kind = if i < bytes.len() && bytes[i] == b'=' {
                    i += 1;
                    TokKind::EqEq
                } else {
                    TokKind::Eq
                };
                push_tok(&mut toks, kind, start, i);
            }
            b'<' => {
                i += 1;
                push_tok(&mut toks, TokKind::Lt, start, i);
            }
            b'(' => {
                i += 1;
                push_tok(&mut toks, TokKind::LParen, start, i);
            }
            b')' => {
                i += 1;
                push_tok(&mut toks, TokKind::RParen, start, i);
            }
            b'{' => {
                i += 1;
                push_tok(&mut toks, TokKind::LBrace, start, i);
            }
            b'}' => {
                i += 1;
                push_tok(&mut toks, TokKind::RBrace, start, i);
            }
            b';' => {
                i += 1;
                push_tok(&mut toks, TokKind::Semi, start, i);
            }
            b':' => {
                i += 1;
                push_tok(&mut toks, TokKind::Colon, start, i);
            }
            b',' => {
                i += 1;
                push_tok(&mut toks, TokKind::Comma, start, i);
            }
            _ => {
                // Non-alphabet: one Bad token covering the next UTF-8 scalar
                // or maximal ill-formed subsequence (never a UTF-8 split).
                i += utf8_token_len(bytes, i);
                push_tok(&mut toks, TokKind::Bad, start, i);
            }
        }
    }
    let _ = TRANSLIT; // (保留:關鍵字表來源標記)
    toks
}

/// 詞法不變量檢驗(「平鋪」公理,L1 的字節級地基)。
pub fn lexical_invariants(src: &str) -> Result<(), String> {
    lexical_invariants_bytes(src.as_bytes())
}

/// Byte-level tiling invariant for arbitrary inputs.
pub fn lexical_invariants_bytes(bytes: &[u8]) -> Result<(), String> {
    let toks = lex_bytes(bytes);
    let mut expected = 0u32;
    for t in &toks {
        if t.span.start != expected {
            return Err(format!(
                "gap/overlap at byte {}: token {:?} starts at {}",
                expected, t.kind, t.span.start
            ));
        }
        // Spans produced from &str / validated UTF-8 chunks must be char-aligned
        // when the covered region is valid UTF-8.
        if let Ok(s) = std::str::from_utf8(bytes) {
            if !s.is_char_boundary(t.span.start as usize)
                || !s.is_char_boundary(t.span.end as usize)
            {
                return Err(format!(
                    "token {:?} span {} is not on a UTF-8 char boundary",
                    t.kind, t.span
                ));
            }
        }
        expected = t.span.end;
    }
    if expected != bytes.len() as u32 {
        return Err(format!(
            "lexer did not cover source: covered {} of {} bytes",
            expected,
            bytes.len()
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lex_lexical_invariants() {
        // 對任意(含髒)輸入,詞法器必須平鋪全源碼。用一小段窮舉驗證。
        let alphabet = [
            "x", "1", " ", "(", ")", "{", "}", ";", "&", "=", "//", "\n", "@", "fn", "let", "mut",
            "if", "else", "while", "true", "false", "*", "+", "-", "<", ":", ",",
        ];
        fn gen(alphabet: &[&str], len: usize, cur: &mut String, n: usize, out: &mut Vec<String>) {
            if n >= len {
                out.push(cur.clone());
                return;
            }
            for a in alphabet {
                cur.push_str(a);
                gen(alphabet, len, cur, n + 1, out);
                let k = a.len();
                cur.truncate(cur.len() - k);
            }
        }
        let mut samples = Vec::new();
        for l in 1..=3 {
            gen(&alphabet, l, &mut String::new(), 0, &mut samples);
        }
        for s in samples {
            match lexical_invariants(&s) {
                Ok(()) => {}
                Err(e) => panic!("lexical tiling violated for {:?}: {}", s, e),
            }
        }
    }

    #[test]
    fn lex_cjk_is_single_bad_token() {
        let src = "中文";
        let toks = lex(src);
        assert_eq!(toks.len(), 2);
        assert_eq!(toks[0].kind, TokKind::Bad);
        assert_eq!(toks[0].span, Span::new(0, "中".len() as u32));
        assert_eq!(toks[1].kind, TokKind::Bad);
        assert_eq!(toks[1].span.len() as usize, "文".len());
        lexical_invariants(src).unwrap();
        // Slicing by token span must not panic.
        assert_eq!(
            &src[toks[0].span.start as usize..toks[0].span.end as usize],
            "中"
        );
        assert_eq!(
            &src[toks[1].span.start as usize..toks[1].span.end as usize],
            "文"
        );
    }

    #[test]
    fn lex_emoji_is_single_bad_token() {
        let src = "a😀b";
        let toks = lex(src);
        assert_eq!(toks.len(), 3);
        assert_eq!(toks[0].kind, TokKind::Ident);
        assert_eq!(toks[1].kind, TokKind::Bad);
        assert_eq!(toks[1].span.len() as usize, "😀".len());
        assert_eq!(toks[2].kind, TokKind::Ident);
        lexical_invariants(src).unwrap();
        let mid = &src[toks[1].span.start as usize..toks[1].span.end as usize];
        assert_eq!(mid, "😀");
    }

    #[test]
    fn lex_combining_char_not_split() {
        // "e" + combining acute accent (U+0301)
        let src = "e\u{0301}";
        let toks = lex(src);
        assert_eq!(toks.len(), 2);
        assert_eq!(toks[0].kind, TokKind::Ident);
        assert_eq!(toks[1].kind, TokKind::Bad);
        assert_eq!(toks[1].span.len(), "\u{0301}".len() as u32);
        lexical_invariants(src).unwrap();
    }

    #[test]
    fn lex_bytes_invalid_utf8_tiling() {
        // Lone continuation byte, truncated 3-byte sequence, overlong-ish junk.
        let samples: &[&[u8]] = &[
            &[0x80],
            &[0xE4, 0xB8],       // truncated 中
            &[0xF0, 0x9F, 0x98], // truncated emoji
            b"ok\x80end",
            &[0xC0, 0x80], // invalid overlong encoding of NUL (ill-formed)
        ];
        for s in samples {
            lexical_invariants_bytes(s).unwrap_or_else(|e| panic!("{:?}: {}", s, e));
            let toks = lex_bytes(s);
            assert!(!toks.is_empty() || s.is_empty());
            // No token may claim a span that cuts through a *valid* UTF-8 char
            // when the whole buffer is valid; for invalid buffers, spans still tile.
            let covered: u32 = toks.last().map(|t| t.span.end).unwrap_or(0);
            assert_eq!(covered as usize, s.len());
        }
    }

    #[test]
    fn lex_comment_with_multibyte_stays_char_aligned() {
        let src = "// 註解😀\nx";
        lexical_invariants(src).unwrap();
        let toks = lex(src);
        // After comment trivia we should still be able to slice every token.
        for t in &toks {
            let _ = &src[t.span.start as usize..t.span.end as usize];
        }
        assert_eq!(toks.last().unwrap().kind, TokKind::Ident);
    }
}
