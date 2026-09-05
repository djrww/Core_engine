//! Token 樹(TT)模型 —— 巨集規則樹的載體(借.md §0 形式框架)。
//!
//! macro_rules 的匹配對象是有序 token 森林;本模組提供最小可用的樹模型
//! 與微型詞法器,供 `macro_lab` 的樣式匹配器、展開器與互斥檢查器使用。
//!
//! 終止度量(借.md §3):`μ(forest) = |forest|`(token 樹數目)。

/// 定界符
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Delim {
    Paren,
    Bracket,
    Brace,
}

impl Delim {
    pub fn open(self) -> char {
        match self {
            Delim::Paren => '(',
            Delim::Bracket => '[',
            Delim::Brace => '{',
        }
    }
    pub fn close(self) -> char {
        match self {
            Delim::Paren => ')',
            Delim::Bracket => ']',
            Delim::Brace => '}',
        }
    }
}

/// Token 掟子
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Tok {
    /// 標識符(含關鍵字):`acc`、`mut`、`x`
    Ident(String),
    /// 字面量:`1`、`0usize`、`"s"`
    Lit(String),
    /// 單字標點:`@`、`,`、`+`
    Punct(char),
}

impl Tok {
    pub fn describe(&self) -> &'static str {
        match self {
            Tok::Ident(_) => "ident",
            Tok::Lit(_) => "literal",
            Tok::Punct(_) => "punct",
        }
    }
}

/// Token 樹:原子或定界群組
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TT {
    Atom(Tok),
    Group(Delim, Vec<TT>),
}

impl TT {
    /// 該樹是否為指定定界符的群組
    pub fn as_group(&self, d: Delim) -> Option<&[TT]> {
        match self {
            TT::Group(dd, inner) if *dd == d => Some(inner),
            _ => None,
        }
    }

    /// 原子種類描述(供 Frag 接受判定)
    pub fn kind(&self) -> &'static str {
        match self {
            TT::Atom(Tok::Ident(_)) => "ident",
            TT::Atom(Tok::Lit(_)) => "literal",
            TT::Atom(Tok::Punct(_)) => "punct",
            TT::Group(..) => "group",
        }
    }
}

/// 微型詞法器:把簡單 token 字串解析為 TT 森林。
///
/// 支持的子集(模型的誠實邊界,見 MACRO_SEVEN_PRINCIPLES.md §4):
/// 標識符 `[A-Za-z_][A-Za-z0-9_]*`、整數字面量 `[0-9][0-9A-Za-z_]*`、
/// 單字標點、三對定界符與 `::`(`:` 視為單字標點,兩個 `:` 亦同)。
pub fn parse_forest(src: &str) -> Result<Vec<TT>, String> {
    let mut chars: Vec<char> = src.chars().collect();
    let mut pos = 0usize;
    let out = parse_seq(&mut chars, &mut pos, None)?;
    if pos != chars.len() {
        return Err(format!("trailing input at char {}", pos));
    }
    Ok(out)
}

fn parse_seq(
    chars: &mut Vec<char>,
    pos: &mut usize,
    closer: Option<char>,
) -> Result<Vec<TT>, String> {
    let mut out = Vec::new();
    while *pos < chars.len() {
        let c = chars[*pos];
        if c.is_whitespace() {
            *pos += 1;
            continue;
        }
        if let Some(cl) = closer {
            if c == cl {
                return Ok(out);
            }
        }
        match c {
            '(' | '[' | '{' => {
                let d = match c {
                    '(' => Delim::Paren,
                    '[' => Delim::Bracket,
                    _ => Delim::Brace,
                };
                *pos += 1;
                let inner = parse_seq(chars, pos, Some(d.close()))?;
                let cl = chars.get(*pos).copied();
                if cl != Some(d.close()) {
                    return Err(format!("unclosed {:?} group", d));
                }
                *pos += 1;
                out.push(TT::Group(d, inner));
            }
            ')' | ']' | '}' => {
                return Err(format!("unmatched closer {} at {}", c, pos));
            }
            _ if c.is_ascii_alphabetic() || c == '_' || c == '@' => {
                let start = *pos;
                if c == '@' {
                    *pos += 1;
                }
                while *pos < chars.len() {
                    let ch = chars[*pos];
                    if ch.is_ascii_alphanumeric() || ch == '_' {
                        *pos += 1;
                    } else {
                        break;
                    }
                }
                let s: String = chars[start..*pos].iter().collect();
                if s == "@" {
                    out.push(TT::Atom(Tok::Punct('@')));
                } else {
                    out.push(TT::Atom(Tok::Ident(s)));
                }
            }
            _ if c.is_ascii_digit() => {
                let start = *pos;
                while *pos < chars.len() {
                    let ch = chars[*pos];
                    if ch.is_ascii_alphanumeric() || ch == '_' {
                        *pos += 1;
                    } else {
                        break;
                    }
                }
                let s: String = chars[start..*pos].iter().collect();
                out.push(TT::Atom(Tok::Lit(s)));
            }
            _ => {
                out.push(TT::Atom(Tok::Punct(c)));
                *pos += 1;
            }
        }
    }
    match closer {
        None => Ok(out),
        Some(cl) => Err(format!("expected closer {} before end", cl)),
    }
}

/// 渲染 TT 森林回字串(快照/顯示用;群組內以空白分隔)
pub fn render_forest(f: &[TT]) -> String {
    let mut s = String::new();
    for (i, t) in f.iter().enumerate() {
        if i > 0 {
            s.push(' ');
        }
        s.push_str(&render_tt(t));
    }
    s
}

fn render_tt(t: &TT) -> String {
    match t {
        TT::Atom(Tok::Ident(s)) | TT::Atom(Tok::Lit(s)) => s.clone(),
        TT::Atom(Tok::Punct(c)) => c.to_string(),
        TT::Group(d, inner) => format!("{}{}{}", d.open(), render_forest(inner), d.close()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_roundtrip_simple() {
        let f = parse_forest("( a b 1 )").unwrap();
        assert_eq!(
            f,
            vec![TT::Group(
                Delim::Paren,
                vec![
                    TT::Atom(Tok::Ident("a".into())),
                    TT::Atom(Tok::Ident("b".into())),
                    TT::Atom(Tok::Lit("1".into()))
                ]
            )]
        );
    }

    #[test]
    fn parse_nested_groups_and_discriminator() {
        // muncher 輸入形態:@ acc [ 1 1 ] x y(renderer 緊湊形式:括號內側不留空)
        let f = parse_forest("( @ acc [ 1 1 ] x y )").unwrap();
        assert_eq!(render_forest(&f), "(@ acc [1 1] x y)");
        // 首字面判別子:Punct('@') + Ident("acc")
        match &f[0] {
            TT::Group(Delim::Paren, inner) => {
                assert_eq!(inner[0], TT::Atom(Tok::Punct('@')));
                assert_eq!(inner[1], TT::Atom(Tok::Ident("acc".into())));
            }
            _ => panic!("expected group"),
        }
    }

    #[test]
    fn parse_rejects_malformed() {
        assert!(parse_forest("( unclosed").is_err());
        assert!(parse_forest(") stray").is_err());
        assert!(parse_forest("( a ) extra )").is_err());
    }

    #[test]
    fn mu_measure_is_tree_count() {
        let f = parse_forest("( a [ b ] c )").unwrap();
        // μ 定義在森林層級:頂層 = 1 棵(群組樹)
        assert_eq!(f.len(), 1);
        // 群組內 = 3 棵:a、[b]、c
        let inner = f[0].as_group(Delim::Paren).unwrap();
        assert_eq!(inner.len(), 3);
    }
}
