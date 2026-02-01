//! DomainU256 - Wrapper type for U256 with proper serde and env parsing support
//!
//! Handles parsing from:
//! - Decimal strings: "5000000000000000000000" -> U256
//! - Hex strings with 0x prefix: "0x10f0cf064dd59200000" -> U256

use primitive_types::U256;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{self, Visitor};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct DomainU256(pub U256);

impl DomainU256 {
    /// Parse from a string (decimal or hex with 0x prefix)
    pub fn from_string(s: &str) -> Result<Self, String> {
        let cleaned = s.trim();

        if cleaned.starts_with("0x") || cleaned.starts_with("0X") {
            U256::from_str(cleaned)
                .map(DomainU256)
                .map_err(|e| format!("Failed to parse hex U256: {}", e))
        } else {
            U256::from_dec_str(cleaned)
                .map(DomainU256)
                .map_err(|e| format!("Failed to parse decimal U256: {}", e))
        }
    }
}

impl fmt::Display for DomainU256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Serialize for DomainU256 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Serialize as decimal string
        serializer.serialize_str(&self.0.to_string())
    }
}

impl<'de> Deserialize<'de> for DomainU256 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct DomainU256Visitor;

        impl<'de> Visitor<'de> for DomainU256Visitor {
            type Value = DomainU256;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string representing a U256 value in decimal or hex format")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                DomainU256::from_string(value).map_err(de::Error::custom)
            }

            fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.visit_str(&value)
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(DomainU256(U256::from(value)))
            }
        }

        deserializer.deserialize_any(DomainU256Visitor)
    }
}

impl From<U256> for DomainU256 {
    fn from(input: U256) -> Self {
        Self(input)
    }
}

impl From<DomainU256> for U256 {
    fn from(input: DomainU256) -> Self {
        input.0
    }
}

impl std::ops::Deref for DomainU256 {
    type Target = U256;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_large_decimal() {
        // 5000 tokens with 18 decimals
        let result = DomainU256::from_string("5000000000000000000000").unwrap();
        assert_eq!(result.to_string(), "5000000000000000000000");
    }

    #[test]
    fn test_parse_hex() {
        let result = DomainU256::from_string("0x10f0cf064dd59200000").unwrap();
        assert_eq!(result.to_string(), "5000000000000000000000");
    }

    #[test]
    fn test_deserialize_decimal_string() {
        let json = r#""1000000000000000000000""#;
        let result: DomainU256 = serde_json::from_str(json).unwrap();
        assert_eq!(result.to_string(), "1000000000000000000000");
    }

    #[test]
    fn test_serialize_to_decimal() {
        let value = DomainU256::from_string("5000000000000000000000").unwrap();
        let json = serde_json::to_string(&value).unwrap();
        assert_eq!(json, r#""5000000000000000000000""#);
    }
}
