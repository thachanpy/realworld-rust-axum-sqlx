use crate::core::error::error::AppError;
use crate::launcher::ServiceLauncher;

mod api;
mod core;
mod db;
mod job;
mod launcher;
mod resources;
mod service;
mod utils;
mod worker;

#[tokio::main]
async fn main() -> Result<(), AppError> {
  let launcher: ServiceLauncher = ServiceLauncher::new();
  launcher.start().await
}
