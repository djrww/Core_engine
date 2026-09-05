# 巨集七原則實作佈署計畫（借.md → cl0r0 機核實作)

> 依據《借.md》(TRS 形式框架 + 七條巨集原則 + 3·o²·n² 借用組合模型)之完整實作藍圖。
> 立場:另一個 AI 寫了 9000+ 行「每棵都成樹」;本計畫以 **先規劃樹、後寫碼** 的紀律出發,
> 目標是每一條規則樹、每一個驗證主張都能對應到可機械重放的檢查點。

---

## 0. 依賴決策(已檢查)

| 項目 | 狀態 | 決策 |
|---|---|---|
| runtime deps | **0**(Cargo.lock 僅自身) | 維持零依賴(審計 S-02 護城河) |
| dev-deps(proptest/insta/trybuild/cargo-expand) | 未安裝 | **不引入**;以廠內等價物取代(見 §1) |
| 工具鏈 | rust 1.98.0 + clippy + rustfmt | 與 rust-toolchain.toml 一致 |
| 既有自証基礎 | 96/96 測試 · clippy 0 警告 · verify_all 9 門禁 | 巨集工作疊加於其上,不得造成退化 |

### 借.md 驗證階梯 → 廠內(零依賴)等價物

| 借.md 工具 | 保證 | 廠內等價物 |
|---|---|---|
| `cargo expand` + `insta` | 展開結果 syntactically 穩定 | `macro_lab` 模型展開器:`ExpansionTrace`(逐步規則、μ 記錄、比較計數)+ 手寫快照斷言 |
| `proptest` | 統計性等價 | `gen::Rng` 確定性種子性質測試(bounded,如實申報非 ∀) |
| `trybuild` | compile-fail / shadowing | shadowing 正例以真測試編譯執行;反例(相對路徑壞掉)以文檔+模型層斷言 |
| Lean/Coq 結構遞歸 | 終止 | 模型展開器 μ 嚴格遞減記錄 + fuel(誠實區分「有界」≠「終止」) |
| Polonius Datalog | error=∅ 判定 | `borrow_model::MiniDatalog`(附件 §2 五謂詞規則原樣實作,定點求解) |
| Miri / -Z polonius | 動態/另核對拍 | 與既有 `rep_dd::red_edges`、naive/sweep/laminar 三算法差分對拍 |

---

## 1. 模組佈署圖

```
core_engine/
├── src/token_tree.rs        [新] TT 樹模型 + 微型詞法器(§2.1)
├── src/macro_lab.rs         [新] Pat/Tpl 規則樹 + 展開器(telemetry)
│                                 + 互斥檢查器 + 線性檢查器
│                                 + 七原則註冊表(規則樹=單一真相)
│                                 + 真 macro_rules! 巨集(與模型同構)
│                                 + verify_seven_principles() 七門禁
├── src/borrow_model.rs      [新] 借用 (π,κ,o) 模型:3 格衝突矩陣、
│                                 naive O(n²)/sweep O(n log n)/laminar depth×n、
│                                 MiniDatalog(附件 §2)、搜尋空間會計(3·o²·n²)
├── src/bin/macro_lab.rs     [新] 七原則自証執行器(廠風格輸出)
├── src/bin/verify_all.rs    [改] 增 Gate 10/10(九項→十項)
├── src/lib.rs               [改] 註冊三模組
├── .github/workflows/ci.yml [改] 機核清單加 macro_lab
└── README/CONTRIBUTING/ARCHITECTURE [改] 門禁數/測試數對齊
```

---

## 2. 每棵樹結構(實作前定案)

### 2.1 Token 樹(TT)——一切規則的載體

```
TT     ::= Ident(name) | Lit(text) | Punct(ch) | Group(Delim, Forest)
Delim  ::= Paren '(' | Bracket '[' | Brace '{'
Forest = Vec<TT>                    // 有序森林
μ(forest) = |forest|                // 終止度量:token 樹數目(借.md §3)
```

### 2.2 規則樹:LHS 樣式 / RHS 模板

```
Pat ::= Tok(TT)                 // 字面 token(判別子)
      | Meta(name, Frag)        // $x:frag —— sorted 變數(原則 7)
      | Group(Delim, [Pat])     // 定界群
      | Rep([Pat], sep?)        // $(...)* —— 僅現於序列內
Frag ::= Tt|Ident|Lit|Punct|Expr|Ty|Path    // 片段種類(模型為保守子集,見 §5)

Tpl ::= Tok(TT) | Sub(name)     // 代換點:Tree 代換(非 tt 片段=原子子樹)
      | Group(Delim,[Tpl]) | Rep([Tpl])
      | Recurse([Tpl])          // 遞迴調用點(模型層追蹤 μ)
Rule = { name, lhs:[Pat], rhs: Tpl }
```

### 2.3 巨集規則樹逐棵定案(七原則註冊表)

**(A) `cl0_count_tts` / `cl0_count_tts_inner` —— 原則 1+3(+6)**

```
E1(入口,單規則宏 ⇒ 平凡互斥):
  LHS: ( Rep([Meta(tts,Tt)]) )
  RHS: Recurse( @ acc [ ] Rep(Sub tts) )          -- 委派 inner,僅執行一次

I1(終止):
  LHS: @ acc [ Rep([Meta(acc,Tt)]) ]
  RHS: 0usize Rep([ '+', Sub(acc) ])              -- 無 Recurse ⇒ 終態
I2(步進,μ 嚴格遞減):
  LHS: @ acc [ Rep([Meta(a,Tt)]) ] Meta(h,Tt) Rep([Meta(r,Tt)])
  RHS: Recurse( @ acc [ Rep(Sub a) 1 ] Rep(Sub r) )
  μ: 1+|r| < 1+1+|r| ✓  (head 消耗一棵)

互斥性(原則 1):I1 要求 ']' 後輸入結束;I2 要求 ']' 後 ≥1 棵 ⇒ L(I1)∩L(I2)=∅
  (前綴 '@acc[...]' 長度固定 ⇒ 結構檢查器給出精確 Disjoint 判定)
設計決策:入口拆成單規則宏 —— 經典三規則 count_tts!(入口 catch-all 會與
  @acc 規則相交,靠次序兜底)違反互斥紀律,故不採用。
```

**(B) `cl0_double`(expr,正例)vs `cl0_double_tt`(反例)—— 原則 7**

```
D1: LHS: ( Meta(e,Expr) )          RHS: ( ( Sub e ) * 2 )     -- double!(1+1)=4
D2: LHS: ( Rep([Meta(t,Tt)]) )     RHS: ( Rep(Sub t) * 2 )    -- 1+1*2=3 反例
兩者為兩個單規則宏(同宏雙規則會相交 ⇒ 違反原則 1,故拆開;反例僅供對照教學)
```

**(C) `cl0_borrow_kind` —— 原則 1(首字面判別)**

```
B1: LHS: ( mut  Meta(p,Ident) )    RHS: …   -- 首token 'mut'
B2: LHS: ( shr  Meta(p,Ident) )    RHS: …   -- 首token 'shr'  ⇒ 首 token 即分岔,交集 trivially 空
```

**(D) `cl0_produce` / `cl0_consume` —— 原則 4(CPS)**

```
PR1: LHS: ( Meta(k,Ident) )  RHS: Recurse→ k ( 11 , 31 )   -- transcriber 直出
CO1: LHS: ( Meta(a,Expr) , Meta(b,Expr) )  RHS: ( (Sub a)+(Sub b) )
定理實測:exp(produce!(consume)) = consume!(11,31) = 42(outermost ≡ innermost)
```

**(E) `cl0_safe_vec` —— 原則 5(絕對路徑)+ 6(重複內線性)**

```
V1: LHS: ( )                       RHS: ::std::vec::Vec::new()
V2: LHS: ( Meta(e0,Expr) Rep([',',Meta(e,Expr)],sep=',' ... )
    RHS: {{ let mut v = ::std::vec::Vec::new(); v.push(e0); Rep(v.push(e);) v }}
    -- 全部非 hygienic 名稱=絕對路徑;每個 $e 在每次重複內恰出現一次
```

**(F) `cl0_with_val` —— 原則 6(let 強制 sharing)**

```
W1: LHS: ( Meta(val,Expr) , Meta(f,Expr) )
    RHS: {{ let a = Sub val ; ( Sub f ) ( &a , &a ) }}
    |rhs|_$val = 1 ⇒ 線性 ✓;a 為 hygiene 局部變數;副作用恰一次(Cell 計數實測)
```

**(G) `cl0_laminar_scope` —— `{{ }}` 區間有界 + 層狀族(借.md 第二部分核心)**

```
S1: LHS: ( Meta(id,Lit) { Rep([Meta(body,Tt)]) } )
    RHS: {{ let _g = $crate::macro_lab::ScopeGuard::enter(id) ; { Rep(Sub body) } }}
    -- $crate = 原則 5 在 crate 內項目的正確形態(hygiene)
    -- Drop 釘死區間右端點 ⇒ o 有界;嵌套塊 ⇒ laminar family(執行期實測)
```

### 2.4 借用模型樹(3·o²·n² 的解剖)

```
Borrow b = ( π_b , κ_b , o_b )
  π = Vec<u32>(投影路徑)   π1 ∩̄ π2 ⟺ prefix(π1,π2) ∨ prefix(π2,π1)
  κ ∈ {Shr, Mut}
  o = [s,e) 區間(直線碼;端點比較 s1≤e2 ∧ s2≤e1)

κ-矩陣(4 格,恰 3 格衝突 —— 公式中的常數 3):
              Shr      Mut
        Shr   ✓ 相容   ✗ 衝突
        Mut   ✗ 衝突   ✗ 衝突

conflict(b1,b2) = π重疊 ∧ o重疊 ∧ κ衝突
naive 檢查空間 = 3 × C(n,2) × O(1)(每區間 2 端點 —— 公式中的常數 2)
```

**層狀族樹(laminar)—— `{{ }}` 雙括號引理**:

```
scope A [0..9)
├─ scope B [1..4)        A ⊇ B
│   └─ scope C [2..3)    B ⊇ C
└─ scope D [5..8)        A ⊇ D, C ∩ D = ∅
∀ 區間對:不交 或 包含(無部分重疊)
⇒ 衝突只發生在祖先—後代邊 ⇒ 檢查空間 C(n,2) → depth × n
```

### 2.5 Datalog 定點樹(附件 §2 原樣)

```
EDB:  borrow(L,Place,Kind) · region_live_at(R,P) · borrow_region(R,L,P)
      access(P,Place,Kind) · conflict_kind ×3 · overlaps(P1,P2)
IDB:  borrow_live_at(L,P) :- borrow_region(R,L,_), region_live_at(R,P)
      invalidates(P,L)    :- access(P,P2,K2), borrow(L,P1,K1,_),
                             overlaps(P1,P2), conflict_kind(K1,K2)
      error(P)            :- invalidates(P,L), borrow_live_at(L,P)
單調 ∧ 有限域 ⇒ 定點存在唯一(判定終止);error=∅ ⟺ ownership 正確
```

### 2.6 複雜度會計樹(tt-muncher O(n²) 的實證)

```
每步成本 = 掃描 I1 失敗(|acc|+c) + 掃描 I2 命中(|acc|+1+|tail|)
Σ_{i=0..n-1} (2i + (n-i) + c) = 2·n(n-1)/2 + n(n+1)/2 + cn = Θ(n²)
  ↑ 累加器同被 re-match(借.md:「accumulator 唔會改善」的實測驗證)
實證法:比較計數 comps(2n)/comps(n) → 4(Θ(n²));naive 借用對檢恰 = C(n,2);
sweep 端點事件恰 = 2n;laminar 檢查比較數 < C(n,2)(深嵌套實例)。
```

---

## 3. 驗證矩陣(原則 → 機制 → 位置)

| # | 原則 | 機械驗證 | 位置 |
|---|---|---|---|
| 1 | 規則互斥 | 樣式相容性檢查器:首 token 判別集 + 結構判定;註冊表全部 Disjoint;負控(相交樣式)必須被標記 | macro_lab::check_exclusive |
| 2 | 語義等價 | 真巨集 vs 參考函數,確定性種子 bounded 性質測試 | macro_lab tests + P2 gate |
| 3 | 結構遞減 | 模型展開器 μ 記錄:n=0..64 全部終止、I2 步嚴格遞減、fuel 不耗盡 | expand() telemetry |
| 4 | CPS | produce!(consume) ≡ consume!(11,31) ≡ 42 | P4 gate |
| 5 | 絕對路徑 | 區域 `struct Vec;` shadowing 下巨集仍編譯且正確 | P5 gate(真編譯) |
| 6 | 線性 | (a)註冊表模板 |rhs|_x ≤ 1 全 pass;(b)Cell 副作用計數=1 | template_linearity + P6 gate |
| 7 | sorted var | double!(1+1)=4 vs double_tt 反例=3;代換原子性 | P7 gate |
| 加 | O(n²)/3·o²·n² | 比較計數比例→4;C(n,2) 精確;2n 端點;naive=sweep=Datalog=rep_red_edges 四方差分 | borrow_model tests |

---

## 4. 誠實邊界(如實申報)

1. **Frag 模型為保守子集**:`Expr` 片段以「消費至頂層 `,`/結尾」近似;真 rustc 片文法更複雜。模型用於 TRS 性質驗證,語義以真巨集為準。
2. **相容檢查器**:無 Rep 或固定前綴+尾分岔 → 精確判定;含自由 Rep → 保守回報可能相交(完備判定需 tree automaton product;與借.md「實務上用 literal discriminator」一致)。
3. **終止=fuel 內有界**:模型在 fuel 內 100% 終止且 μ 嚴格遞減(良基 descent 證據),非 Lean 級 ∀ 證明。
4. **等價=bounded**:確定性種子 0..64,非全稱命題。
5. 模型規則樹與真 macro_rules! 同構(單一真相登記),但模型展開字串 ≠ rustc 展開位元組(無 cargo-expand,零依賴決策)。

---

## 5. 整合點

- `verify_all`:Gate 10/10「巨集七原則機核檢」(GATES 常量 → 十項,文檔同步)
- 新 bin `macro_lab`:七門禁輸出(廠風格 PASSED/SKIPPED/FAILED —— 本門禁純機內,無外部工具 ⇒ 不會 SKIPPED)
- CI:機核清單 + `cargo run --bin macro_lab`;README 快速指令 #12
- 測試數與門禁數在 README/CONTRIBUTING/ARCHITECTURE 以實測值更新
