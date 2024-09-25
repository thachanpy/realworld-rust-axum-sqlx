#[derive(Debug)]
pub struct PaginationHelper {
  pub page: usize,
  pub per_page: usize,
}

impl PaginationHelper {
  pub fn new(page: Option<usize>, per_page: Option<usize>) -> Self {
    PaginationHelper {
      page: page.unwrap_or(1).max(1),
      per_page: per_page.unwrap_or(10).max(0),
    }
  }

  pub fn limit(&self) -> u64 {
    self.per_page as u64
  }

  pub fn offset(&self) -> u64 {
    ((self.page - 1) * self.per_page) as u64
  }
}
