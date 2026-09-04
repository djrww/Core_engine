//! §1.2 — 位址映象 σ : V → ℤ×ℤ ,σ(v) = [a, b) 為源碼字節區間(半開)。
//! 半開區間是全部定律的幾何基石:連續性公理、L5 嵌套定理、編輯位移函數
//! 都建立在「[a,b)」之上。

/// 半開字節區間 [start, end) 。任何節點 / token 的 span 一律滿足 start <= end。
///
/// Fields remain `pub` for ergonomic destructuring across the crate, but the
/// only supported constructors are [`Span::new`] / [`Span::try_new`]. Direct
/// struct literals that violate `start <= end` are considered a logic bug.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

/// Error returned when constructing a span with `start > end`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InvalidSpan {
    pub start: u32,
    pub end: u32,
}

impl std::fmt::Display for InvalidSpan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "invalid span [{}, {}): start > end",
            self.start, self.end
        )
    }
}

impl std::error::Error for InvalidSpan {}

impl Span {
    /// Construct a half-open span. Panics in **all** build modes if `start > end`.
    /// Prefer [`Span::try_new`] at API boundaries that receive untrusted bounds.
    pub fn new(start: u32, end: u32) -> Span {
        match Self::try_new(start, end) {
            Ok(s) => s,
            Err(e) => panic!("{e}"),
        }
    }

    /// Checked constructor: returns [`InvalidSpan`] when `start > end`.
    pub fn try_new(start: u32, end: u32) -> Result<Span, InvalidSpan> {
        if start <= end {
            Ok(Span { start, end })
        } else {
            Err(InvalidSpan { start, end })
        }
    }

    pub fn len(&self) -> u32 {
        self.end - self.start
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// 區間包含(允許等式):σ(u) ⊆ σ(v)
    pub fn contains(&self, other: &Span) -> bool {
        self.start <= other.start && other.end <= self.end
    }

    /// 區間重疊(部分或全部;接觸不算重疊,因為半開)。
    pub fn overlaps(&self, other: &Span) -> bool {
        self.start < other.end && other.start < self.end
    }

    /// Translate both endpoints by `delta`. Returns `None` on underflow
    /// (result &lt; 0) or overflow (result &gt; `u32::MAX`). Same-delta shifts
    /// preserve `start <= end` when they succeed.
    pub fn checked_shift(self, delta: i64) -> Option<Span> {
        let start = (self.start as i64).checked_add(delta)?;
        let end = (self.end as i64).checked_add(delta)?;
        if !(0..=u32::MAX as i64).contains(&start) || !(0..=u32::MAX as i64).contains(&end) {
            return None;
        }
        // Invariant: start <= end is preserved by identical delta.
        debug_assert!(start <= end);
        Some(Span {
            start: start as u32,
            end: end as u32,
        })
    }
}

impl std::fmt::Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}, {})", self.start, self.end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_new_accepts_empty_and_ordered() {
        assert_eq!(Span::try_new(0, 0).unwrap(), Span { start: 0, end: 0 });
        assert_eq!(Span::try_new(3, 3).unwrap(), Span { start: 3, end: 3 });
        assert_eq!(Span::try_new(1, 4).unwrap(), Span { start: 1, end: 4 });
        assert_eq!(
            Span::try_new(0, u32::MAX).unwrap(),
            Span {
                start: 0,
                end: u32::MAX
            }
        );
    }

    #[test]
    fn try_new_rejects_start_gt_end() {
        assert_eq!(Span::try_new(5, 4), Err(InvalidSpan { start: 5, end: 4 }));
        assert_eq!(
            Span::try_new(u32::MAX, 0),
            Err(InvalidSpan {
                start: u32::MAX,
                end: 0
            })
        );
    }

    #[test]
    #[should_panic(expected = "invalid span")]
    fn new_panics_on_invalid_in_all_builds() {
        let _ = Span::new(2, 1);
    }

    #[test]
    fn checked_shift_identity_and_basic() {
        let s = Span::new(10, 20);
        assert_eq!(s.checked_shift(0), Some(Span::new(10, 20)));
        assert_eq!(s.checked_shift(5), Some(Span::new(15, 25)));
        assert_eq!(s.checked_shift(-10), Some(Span::new(0, 10)));
    }

    #[test]
    fn checked_shift_underflow() {
        let s = Span::new(0, 5);
        assert_eq!(s.checked_shift(-1), None);
        let s = Span::new(3, 8);
        assert_eq!(s.checked_shift(-4), None);
        assert_eq!(s.checked_shift(-3), Some(Span::new(0, 5)));
    }

    #[test]
    fn checked_shift_overflow() {
        let s = Span::new(u32::MAX - 2, u32::MAX);
        assert_eq!(s.checked_shift(1), None);
        assert_eq!(s.checked_shift(0), Some(s));
        let s = Span::new(u32::MAX - 5, u32::MAX - 1);
        assert_eq!(s.checked_shift(1), Some(Span::new(u32::MAX - 4, u32::MAX)));
        assert_eq!(s.checked_shift(2), None);
    }

    #[test]
    fn checked_shift_empty_at_boundaries() {
        let empty0 = Span::new(0, 0);
        assert_eq!(empty0.checked_shift(-1), None);
        assert_eq!(empty0.checked_shift(0), Some(empty0));
        let empty_max = Span::new(u32::MAX, u32::MAX);
        assert_eq!(empty_max.checked_shift(1), None);
        assert_eq!(empty_max.checked_shift(0), Some(empty_max));
        assert_eq!(
            empty_max.checked_shift(-1),
            Some(Span::new(u32::MAX - 1, u32::MAX - 1))
        );
    }
}
