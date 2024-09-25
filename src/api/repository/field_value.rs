use chrono::{DateTime, Utc};
use sea_query::{SimpleExpr, Value};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum FieldValue {
  Text(String),            // Maps to PostgresSQL TEXT or VARCHAR
  Integer(i32),            // Maps to PostgresSQL INTEGER
  BigInt(i64),             // Maps to PostgresSQL BIGINT
  Float(f32),              // Maps to PostgresSQL FLOAT or DOUBLE PRECISION
  Enum(String),            // Maps to PostgresSQL ENUM
  DateTime(DateTime<Utc>), // Maps to PostgresSQL TIMESTAMP
}

impl FieldValue {
  pub fn to_simple_expr(&self) -> SimpleExpr {
    match self {
      FieldValue::Text(val) => SimpleExpr::Value(Value::String(Some(Box::new(val.clone())))),
      FieldValue::Integer(val) => SimpleExpr::Value(Value::Int(Option::from(*val))),
      FieldValue::BigInt(val) => SimpleExpr::Value(Value::BigInt(Option::from(*val))),
      FieldValue::Float(val) => SimpleExpr::Value(Value::Float(Some(*val))),
      FieldValue::Enum(val) => SimpleExpr::Value(Value::String(Some(Box::new(val.clone())))),
      FieldValue::DateTime(val) => {
        SimpleExpr::Value(Value::String(Some(Box::new(val.to_rfc3339()))))
      }
    }
  }
}
