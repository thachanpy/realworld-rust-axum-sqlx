use sea_query::Order;

pub struct SortingHelper;

impl SortingHelper {
  pub fn map_order_direction(order_direction: &str) -> Order {
    match order_direction.to_lowercase().as_str() {
      "desc" => Order::Desc,
      _ => Order::Asc,
    }
  }
}
