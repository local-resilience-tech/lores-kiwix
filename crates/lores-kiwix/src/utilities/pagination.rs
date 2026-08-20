/// Pagination parameters and logic for slicing a result set.
#[derive(Debug, Default)]
pub struct Paginator {
    start: Option<usize>,
    count: Option<i64>,
}

impl Paginator {
    const DEFAULT_COUNT: i64 = 10;

    /// Create a paginator from optional start index and count values.
    pub fn new(start: Option<usize>, count: Option<i64>) -> Self {
        Self { start, count }
    }

    /// Slice `items` according to the configured start index and count.
    pub fn page<'a, T>(&self, items: &'a [T]) -> Page<'a, T> {
        let total = items.len();
        let start = self.start.unwrap_or(0).min(total);
        let count = self.count.unwrap_or(Self::DEFAULT_COUNT);
        let intended_count = if count < 0 {
            total.saturating_sub(start)
        } else {
            count as usize
        };
        let end = (start + intended_count).min(total);
        Page {
            items: &items[start..end],
            total,
            start,
        }
    }

    /// Return the configured start index, if any.
    pub fn start(&self) -> Option<usize> {
        self.start
    }

    /// Return the configured count, if any.
    pub fn count(&self) -> Option<i64> {
        self.count
    }
}

/// The result of applying a [`Paginator`] to a slice.
#[derive(Debug)]
pub struct Page<'a, T> {
    pub items: &'a [T],
    pub total: usize,
    pub start: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paginator_pages_through_items() {
        let items: Vec<u32> = (0..25).collect();
        let paginator = Paginator::new(Some(5), Some(10));
        let page = paginator.page(&items);
        assert_eq!(page.total, 25);
        assert_eq!(page.start, 5);
        assert_eq!(page.items, &[5, 6, 7, 8, 9, 10, 11, 12, 13, 14]);
    }

    #[test]
    fn paginator_negative_count_returns_remaining_items() {
        let items: Vec<u32> = (0..25).collect();
        let paginator = Paginator::new(Some(20), Some(-1));
        let page = paginator.page(&items);
        assert_eq!(page.total, 25);
        assert_eq!(page.start, 20);
        assert_eq!(page.items, &[20, 21, 22, 23, 24]);
    }
}
