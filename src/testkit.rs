//! 共享測試/自証見證夾具(審計 D-01:六處手抄巨塊的單一真相)。
//!
//! `Ev { id, storage, kind, it }` 構造塊過去在 `bin/coco_benchmark`、
//! `bin/verify_all`、`cert_generator_factory`、`lemma_stress_generator`、
//! `lemmas`、`tests/sota_verification` 六處逐字複製;任一處修補其餘五處
//! 不動,差分測試會以自證方式製造假綠。本模組將其收編為單一引用。
//!
//! 供 tests/ 與自証門禁 bin 共用;非生產邏輯,不參與任何執行期路徑。

#[doc(hidden)]
pub mod fixtures {
    use crate::ast::Interval;
    use crate::rep_dd::{AState, Ev, K};

    /// 兩事件重疊狀態:Ev0 = Mut `[s1, e1)`,Ev1 = Sh `[s2, e2)`(storage 0)。
    pub fn two_event_state(s1: u32, e1: u32, s2: u32, e2: u32) -> AState {
        AState::new(vec![
            Ev {
                id: 0,
                storage: 0,
                kind: K::Mut,
                it: Interval { start: s1, end: e1 },
            },
            Ev {
                id: 1,
                storage: 0,
                kind: K::Sh,
                it: Interval { start: s2, end: e2 },
            },
        ])
    }

    /// 標準「峰值豐富」狀態:Mut [0,3) × Sh [1,4) —— 產生紅邊與多條
    /// 可應用重寫規則,是 L9-Newman / L10-KB / 憑證工廠的標準輸入。
    pub fn overlapping_pair() -> AState {
        two_event_state(0, 3, 1, 4)
    }

    /// 標準抽樣狀態宇宙(sota `sample_states` / verify_all Gate 3 /
    /// coco_benchmark 同構):`s1 ∈ 0..max`,`e1 ∈ s1+1..=max`,
    /// `s2 ∈ 1..max`,`e2 ∈ s2+1..=max`。
    pub fn sample_state_universe(max_coord: u32) -> Vec<AState> {
        let mut states = Vec::new();
        for s1 in 0..max_coord {
            for e1 in (s1 + 1)..=max_coord {
                for s2 in 1..max_coord {
                    for e2 in (s2 + 1)..=max_coord {
                        states.push(two_event_state(s1, e1, s2, e2));
                    }
                }
            }
        }
        states
    }

    /// 預設宇宙(max_coord = 4,與歷史行為一致)
    pub fn sample_states() -> Vec<AState> {
        sample_state_universe(4)
    }
}

#[cfg(test)]
mod tests {
    use super::fixtures;

    #[test]
    fn fixtures_match_historical_shapes() {
        let st = fixtures::two_event_state(0, 3, 1, 4);
        assert_eq!(st.evs.len(), 2);
        assert_eq!(st.evs[0].it.start, 0);
        assert_eq!(st.evs[1].it.end, 4);

        let uni = fixtures::sample_states();
        assert!(!uni.is_empty());
        // max_coord=4 ⇒ s1∈0..4(4 值,10 組)× s2∈1..4(3 值,6 組)= 60 配置,
        // 與 coco_benchmark 的宇宙一致,且為舊 verify_all/sota 54 狀態宇宙
        // (s1∈0..3)的嚴格超集 —— 覆蓋只增不減。
        assert_eq!(uni.len(), 60);
    }
}
