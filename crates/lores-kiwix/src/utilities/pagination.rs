/// Pagination parameters and logic for slicing a result set.
#[derive(Debug, Default)]
pub struct Paginator {
    start: Option<usize>,
    count: Option<usize>,
}

impl Paginator {
    const DEFAULT_COUNT: usize = 10;

    /// Create a paginator from optional start index and count values.
    pub fn new(start: Option<usize>, count: Option<usize>) -> Self {
        Self { start, count }
    }

    /// Slice `items` according to the configured start index and count.
    pub fn page<'a, T>(&self, items: &'a [T]) -> Page<'a, T> {
        let range = self.range(items.len());
        Page {
            items: &items[range.clone()],
            total: items.len(),
            start: range.start,
        }
    }

    /// Compute the pagination range for a collection of the given total size.
    fn range(&self, total: usize) -> std::ops::Range<usize> {
        let start = self.start.unwrap_or(0).min(total);
        let count = self.count.unwrap_or(Self::DEFAULT_COUNT);
        let end = (start + count).min(total);
        start..end
    }

    /// Return a paginator for the next segment of a combined sequence, where
    /// `first_len` items have already been paged over.
    ///
    /// The returned paginator's start and count are adjusted so that applying
    /// it to the second segment yields the remainder of the original page.
    pub fn tail(&self, first_len: usize) -> Self {
        let original_start = self.start.unwrap_or(0);
        let start = original_start.saturating_sub(first_len);

        let count = self.count.map(|c| {
            let taken_from_first = if original_start < first_len {
                (first_len - original_start).min(c)
            } else {
                0
            };
            c - taken_from_first
        });

        Self::new(Some(start), count)
    }

    /// Return the configured start index, if any.
    pub fn start(&self) -> Option<usize> {
        self.start
    }

    /// Return the configured count, if any.
    pub fn count(&self) -> Option<usize> {
        self.count
    }

    /// Return the resolved start index for a collection of the given total size.
    pub fn start_index(&self, total: usize) -> usize {
        self.start.unwrap_or(0).min(total)
    }
}

/// The result of applying a [`Paginator`] to a slice.
#[derive(Debug)]
pub struct Page<'a, T> {
    pub items: &'a [T],
    total: usize,
    start: usize,
}

impl<'a, T> Page<'a, T> {
    /// Returns true if the paginator reached the end of this source.
    pub fn is_exhausted(&self) -> bool {
        self.start + self.items.len() >= self.total
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paginator_pages_through_items() {
        let items: Vec<u32> = (0..25).collect();
        let paginator = Paginator::new(Some(5), Some(10));
        let page = paginator.page(&items);
        assert_eq!(page.items, &[5, 6, 7, 8, 9, 10, 11, 12, 13, 14]);
        assert!(!page.is_exhausted());
    }

    #[test]
    fn paginator_exhausted_at_end_of_source() {
        let items: Vec<u32> = (0..10).collect();
        let paginator = Paginator::new(Some(5), Some(10));
        let page = paginator.page(&items);
        assert!(page.is_exhausted());
    }

    #[test]
    fn tail_fills_remaining_page_space() {
        let paginator = Paginator::new(Some(5), Some(10));
        let tail = paginator.tail(7);
        let items: Vec<u32> = (0..10).collect();
        let page = tail.page(&items);
        assert_eq!(page.start, 0);
        assert_eq!(page.items, &[0, 1, 2, 3, 4, 5, 6, 7]);
    }

    #[test]
    fn tail_offsets_start_past_first_segment() {
        let paginator = Paginator::new(Some(12), Some(5));
        let tail = paginator.tail(7);
        let items: Vec<u32> = (0..10).collect();
        let page = tail.page(&items);
        assert_eq!(page.start, 5);
        assert_eq!(page.items, &[5, 6, 7, 8, 9]);
    }
}
