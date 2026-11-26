//! Table shell around GPUI Kit's `DataTable`.
//!
//! GPUI Kit renders rows, virtualizes them, fixes columns and opens a context
//! menu. What a console still needs is the part around the grid: which columns
//! are visible, how many rows a page holds and how many there are in total, a
//! per-row action menu, and selection that survives a page change.
//!
//! This module is being built first because every screen of a management console
//! is a table, and each of those four pieces is currently reimplemented per
//! screen.
//!
//! Status: skeleton. The public surface is defined here so the shape is settled
//! before the widgets exist.

/// Rows per page, the values a table offers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageSize {
    Ten,
    Twenty,
    Fifty,
    Hundred,
}

impl PageSize {
    /// The number of rows this choice means.
    pub fn rows(self) -> usize {
        match self {
            PageSize::Ten => 10,
            PageSize::Twenty => 20,
            PageSize::Fifty => 50,
            PageSize::Hundred => 100,
        }
    }

    /// Every choice, in the order a selector shows them.
    pub fn all() -> &'static [PageSize] {
        &[
            PageSize::Ten,
            PageSize::Twenty,
            PageSize::Fifty,
            PageSize::Hundred,
        ]
    }
}

/// Paging state that a table and its footer share.
#[derive(Debug, Clone)]
pub struct Paging {
    page: usize,
    size: PageSize,
    total: usize,
}

impl Default for Paging {
    fn default() -> Self {
        Self {
            page: 1,
            size: PageSize::Ten,
            total: 0,
        }
    }
}

impl Paging {
    /// Current page, 1-based.
    pub fn page(&self) -> usize {
        self.page
    }

    /// Rows per page.
    pub fn size(&self) -> PageSize {
        self.size
    }

    /// Total rows the server reported.
    pub fn total(&self) -> usize {
        self.total
    }

    /// Pages, at least one so an empty table still renders "1 / 1".
    pub fn pages(&self) -> usize {
        let size = self.size.rows();
        self.total.div_ceil(size).max(1)
    }

    /// Advertising a new total also clamps the current page, because a filter
    /// that shrinks the result set must not leave the table on page 9 of 2.
    pub fn set_total(&mut self, total: usize) {
        self.total = total;
        let pages = self.pages();
        if self.page > pages {
            self.page = pages;
        }
    }

    /// Move to a page, ignoring out-of-range requests.
    pub fn go_to(&mut self, page: usize) {
        let pages = self.pages();
        self.page = page.clamp(1, pages);
    }

    /// Change the page size, keeping the first visible row where it was so the
    /// user does not lose their place.
    pub fn set_size(&mut self, size: PageSize) {
        let first_row = (self.page - 1) * self.size.rows();
        self.size = size;
        self.page = (first_row / size.rows() + 1).clamp(1, self.pages());
    }

    /// Offset to request from the server.
    pub fn offset(&self) -> usize {
        (self.page - 1) * self.size.rows()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pages_is_at_least_one_for_an_empty_table() {
        let paging = Paging::default();
        assert_eq!(paging.pages(), 1);
    }

    #[test]
    fn a_shrinking_result_set_clamps_the_current_page() {
        let mut paging = Paging::default();
        paging.set_total(95);
        paging.go_to(10);
        assert_eq!(paging.page(), 10);

        paging.set_total(12);
        assert_eq!(paging.page(), 2);
    }

    #[test]
    fn changing_page_size_keeps_the_first_visible_row() {
        let mut paging = Paging::default();
        paging.set_total(100);
        paging.go_to(3);
        assert_eq!(paging.offset(), 20);

        paging.set_size(PageSize::Fifty);
        assert_eq!(paging.offset(), 0);
        assert_eq!(paging.page(), 1);
    }

    #[test]
    fn go_to_ignores_requests_past_the_end() {
        let mut paging = Paging::default();
        paging.set_total(30);
        paging.go_to(99);
        assert_eq!(paging.page(), 3);
        paging.go_to(0);
        assert_eq!(paging.page(), 1);
    }
}
