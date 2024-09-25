use crate::resources::config::PostgresConfig;
use rand::prelude::SliceRandom;
use rand::rngs::ThreadRng;
use rand::thread_rng;
use sqlx::postgres::PgPoolOptions;
use sqlx::Postgres;
use tracing::{error, info};

pub type DbPool = sqlx::Pool<Postgres>;

#[derive(Clone)]
pub struct PostgresDatabase {
  primary: DbPool,
  replicas: Vec<DbPool>,
}

impl PostgresDatabase {
  pub(crate) async fn connect(config: PostgresConfig) -> Result<Self, sqlx::Error> {
    let primary: DbPool = Self::initialize_pool(&config.master_url, &config).await?;

    let replica_addresses: Vec<&str> = config.replica_urls.split(",").collect::<Vec<&str>>();
    let mut replicas: Vec<DbPool> = Vec::new();

    for address in replica_addresses {
      let replica_config: PostgresConfig = config.clone();
      match Self::initialize_pool(&address.to_string(), &replica_config).await {
        Ok(replica_pool) => replicas.push(replica_pool),
        Err(e) => {
          error!("Failed to connect to a replica database: {}", e);
          return Err(e);
        }
      }
    }

    Ok(Self { primary, replicas })
  }

  async fn initialize_pool(
    address: &String,
    config: &PostgresConfig,
  ) -> Result<DbPool, sqlx::Error> {
    let conn_str: String = Self::build_connection_string(address, config);
    match PgPoolOptions::new()
      .max_connections(config.max_connections)
      .connect(&conn_str)
      .await
    {
      Ok(pool) => {
        info!("Successfully connected to the database {}. ", address);
        Ok(pool)
      }
      Err(e) => {
        error!("Failed to connect to the database {}: {}", address, e);
        Err(e)
      }
    }
  }

  fn build_connection_string(address: &String, config: &PostgresConfig) -> String {
    format!(
      "postgres://{}:{}@{}/{}",
      config.username, config.password, address, config.db_name,
    )
  }

  pub fn primary(&self) -> &DbPool {
    &self.primary
  }

  pub fn replica(&self) -> &DbPool {
    let replicas: &Vec<DbPool> = &self.replicas;
    if replicas.is_empty() {
      &self.primary()
    } else {
      let mut rng: ThreadRng = thread_rng();
      replicas.choose(&mut rng).unwrap_or(&self.primary())
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use sqlx::Error;

  #[tokio::test]
  async fn test_connect_success() {
    let config = PostgresConfig {
      username: "local".to_string(),
      password: "local".to_string(),
      master_url: "localhost".to_string(),
      replica_urls: "localhost".to_string(),
      db_name: "local".to_string(),
      max_connections: 10,
    };

    let result: Result<PostgresDatabase, Error> = PostgresDatabase::connect(config).await;

    assert!(result.is_ok());

    let db_pool: DbPool = result.unwrap().primary;
    assert!(db_pool.acquire().await.is_ok());
  }

  #[tokio::test]
  async fn test_connect_fail() {
    let config = PostgresConfig {
      username: "local".to_string(),
      password: "wrong_password".to_string(),
      master_url: "localhost".to_string(),
      replica_urls: "localhost".to_string(),
      db_name: "rust".to_string(),
      max_connections: 10,
    };

    let result: Result<PostgresDatabase, Error> = PostgresDatabase::connect(config).await;

    assert!(result.is_err());
  }
}
