use config::builder::DefaultState;
use config::{Config, ConfigBuilder, ConfigError, Environment, File, FileFormat};
use jsonwebtoken::Algorithm;
use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};

impl AppConfig {
  pub fn load() -> Result<AppConfig, ConfigError> {
    let base_config_data: String = Self::read_and_process_config("application.yaml");
    let mut builder: ConfigBuilder<DefaultState> = Self::build_config(&base_config_data)?;

    if let Ok(environment) = env::var("ENVIRONMENT") {
      if let Ok(env_config_data) = Self::process_environment_config(&environment) {
        builder = builder.add_source(File::from_str(&env_config_data, FileFormat::Yaml));
      }
    }

    let config: Config = builder.add_source(Environment::default()).build()?;
    config.try_deserialize::<AppConfig>()
  }

  fn read_and_process_config(file_name: &str) -> String {
    let config_path: PathBuf = Path::new(file!()).parent().unwrap().join(file_name);

    let config_path_exists = config_path.exists();
    if !config_path_exists {
      return String::new();
    }

    let config_data = std::fs::read_to_string(config_path).unwrap();
    Self::replace_place_holders(config_data)
  }

  fn replace_place_holders(config_str: String) -> String {
    Regex::new(r"\$\{([^:}]+)(?::([^}]*))?}")
      .unwrap()
      .replace_all(&config_str, |caps: &regex::Captures| {
        let var_name = &caps[1];
        let default_value = &caps[2];
        env::var(var_name).unwrap_or_else(|_| default_value.parse().unwrap())
      })
      .into_owned()
  }

  fn process_environment_config(environment: &str) -> Result<String, std::io::Error> {
    let environment_config_file: String = format!("application-{}.yaml", environment);
    let env_config_data: String = Self::read_and_process_config(&environment_config_file);
    Ok(env_config_data)
  }

  fn build_config(base_config_data: &str) -> Result<ConfigBuilder<DefaultState>, ConfigError> {
    let builder: ConfigBuilder<DefaultState> =
      Config::builder().add_source(File::from_str(base_config_data, FileFormat::Yaml));
    Ok(builder)
  }
}

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
  pub environment: String,
  pub server: ServerConfig,
  pub postgres: PostgresConfig,
  pub jwt: JwtConfig,
  pub aws: AWSConfig,
  pub oauth2: HashMap<String, OAuth2Config>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
  pub address: String,
  pub path_prefix: String,
  pub api: ApiConfig,
  pub worker: WorkerConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ApiConfig {
  pub port: u16,
  pub cors: CorsConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct WorkerConfig {
  pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CorsConfig {
  pub allowed_origin: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PostgresConfig {
  pub master_url: String,
  pub replica_urls: String,
  pub db_name: String,
  pub username: String,
  pub password: String,
  pub max_connections: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct JwtConfig {
  pub private_key_base64: String,
  pub public_key_base64: String,
  pub access_expiration_seconds: i64,
  pub refresh_expiration_seconds: i64,
  pub algorithm: Algorithm,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AWSConfig {
  pub region: String,
  pub s3: AWSS3Config,
  pub sqs: AWSSQSConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AWSS3Config {
  pub bucket_name: String,
  pub presigned_url_expiration_seconds: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AWSSQSConfig {
  pub jobs: HashMap<String, AWSSQSJobConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AWSSQSJobConfig {
  pub queue_url: String,
  pub replicas: u16,
  pub wait_time_seconds: i32,
  pub max_number_of_messages: i32,
  pub visibility_timeout: u16,
  pub delay_seconds: i32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct OAuth2Config {
  pub client_id: String,
  pub client_secret: String,
  pub auth_url: String,
  pub token_url: String,
  pub redirect_url: String,
  pub user_info_url: String,
  pub scopes: Vec<String>,
}

#[cfg(test)]
mod tests {
  use super::*;
  use serial_test::serial;
  use std::env;
  use std::fs;
  use tempfile::NamedTempFile;

  #[tokio::test]
  #[serial]
  async fn test_load_config_with_placeholders() {
    let temp_file: NamedTempFile = NamedTempFile::new().unwrap();
    let config_content = r#"
      server:
        address: ${SERVER_ADDRESS:0.0.0.0}
        api:
          port: ${SERVER_API_PORT:8080}
    "#;

    env::remove_var("SERVER_ADDRESS");
    env::remove_var("SERVER_API_PORT");

    let config_path: &str = temp_file.path().to_str().unwrap();
    fs::write(config_path, config_content).unwrap();

    let config: AppConfig = AppConfig::load().expect("Failed to load config");

    assert_eq!(config.server.address, "0.0.0.0");
    assert_eq!(config.server.api.port, 8080);
  }

  #[tokio::test]
  #[serial]
  async fn test_replace_place_holders() {
    let config_str: &str = r#"
      server:
        address: ${SERVER_ADDRESS:0.0.0.0}
        api:
          port: ${SERVER_API_PORT:8080}
    "#;

    env::set_var("SERVER_ADDRESS", "127.0.0.1");
    env::remove_var("SERVER_API_PORT");

    let replaced_config: String = AppConfig::replace_place_holders(config_str.to_string());

    assert_eq!(
      replaced_config,
      r#"
      server:
        address: 127.0.0.1
        api:
          port: 8080
    "#
    );

    env::remove_var("SERVER_ADDRESS");
  }
}
