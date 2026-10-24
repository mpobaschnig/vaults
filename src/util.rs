// uuid.rs
//
// Copyright 2025 Martin Pobaschnig
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.
//
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::user_config_manager::UserConfigManager;
use gtk::glib;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::str::FromStr;

/// A UUID type backed by glib's UUID implementation.
///
/// Wraps a hyphenated UUID string (e.g., `550e8400-e29b-41d4-a716-446655440000`).
/// Provides generation via `glib::uuid_string_random()` and validation via
/// `glib::uuid_string_is_valid()`.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Uuid(String);

impl Default for Uuid {
    fn default() -> Self {
        Self::nil()
    }
}

impl Uuid {
    /// Generate a new random UUID (RFC 4122 version 4).
    pub fn new_v4() -> Self {
        let uuid_str = glib::uuid_string_random().to_string();
        Uuid(uuid_str)
    }

    /// Returns the nil UUID (`00000000-0000-0000-0000-000000000000`).
    pub fn nil() -> Self {
        Uuid(String::from("00000000-0000-0000-0000-000000000000"))
    }

    /// Parse a UUID from a string.
    pub fn parse_str(s: &str) -> Result<Self, UuidParseError> {
        if glib::uuid_string_is_valid(s) {
            Ok(Uuid(s.to_string()))
        } else {
            Err(UuidParseError)
        }
    }

    /// Returns the UUID as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Uuid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Debug for Uuid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Uuid(\"{}\")", self.0)
    }
}

impl FromStr for Uuid {
    type Err = UuidParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse_str(s)
    }
}

impl Serialize for Uuid {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for Uuid {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Uuid::parse_str(&s).map_err(serde::de::Error::custom)
    }
}

/// Error returned when a UUID string is invalid.
#[derive(Debug, Clone, Copy)]
pub struct UuidParseError;

impl fmt::Display for UuidParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid UUID format")
    }
}

impl std::error::Error for UuidParseError {}

/// Generate a unique UUID that does not already exist in the user config.
pub fn generate_uuid() -> Uuid {
    let map = UserConfigManager::instance().get_map();
    for _ in 0..1000 {
        let uuid = Uuid::new_v4();
        if map.contains_key(&uuid) {
            log::debug!(
                "Generated UUID {} already exists, generating a new one",
                uuid
            );
            continue;
        }
        log::debug!("Generated unique UUID: {}", uuid);
        return uuid;
    }
    log::error!("Failed to generate a unique UUID after 1000 attempts");
    Uuid::nil()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::hash::{Hash, Hasher};

    #[test]
    fn test_new_v4_generates_valid_uuid() {
        let uuid = Uuid::new_v4();
        assert!(glib::uuid_string_is_valid(uuid.as_str()));
    }

    #[test]
    fn test_new_v4_uniqueness() {
        let mut uuids = HashSet::new();
        for _ in 0..100 {
            let uuid = Uuid::new_v4();
            assert!(uuids.insert(uuid), "Generated duplicate UUID");
        }
    }

    #[test]
    fn test_nil() {
        let nil = Uuid::nil();
        assert_eq!(nil.as_str(), "00000000-0000-0000-0000-000000000000");
        assert!(glib::uuid_string_is_valid(nil.as_str()));
    }

    #[test]
    fn test_nil_consistency() {
        let nil1 = Uuid::nil();
        let nil2 = Uuid::nil();
        assert_eq!(nil1, nil2);
    }

    #[test]
    fn test_parse_str_valid() {
        let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000");
        assert!(uuid.is_ok());
        assert_eq!(
            uuid.unwrap().as_str(),
            "550e8400-e29b-41d4-a716-446655440000"
        );
    }

    #[test]
    fn test_parse_str_invalid() {
        let result = Uuid::parse_str("not-a-uuid");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_str_empty() {
        let result = Uuid::parse_str("");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_str_malformed() {
        let result = Uuid::parse_str("550e8400-e29b-41d4-a716");
        assert!(result.is_err());
    }

    #[test]
    fn test_as_str() {
        let uuid = Uuid::parse_str("a1b2c3d4-e5f6-7890-abcd-ef1234567890").unwrap();
        assert_eq!(uuid.as_str(), "a1b2c3d4-e5f6-7890-abcd-ef1234567890");
    }

    #[test]
    fn test_display() {
        let uuid = Uuid::parse_str("12345678-1234-5678-1234-567812345678").unwrap();
        assert_eq!(format!("{}", uuid), "12345678-1234-5678-1234-567812345678");
    }

    #[test]
    fn test_debug() {
        let uuid = Uuid::parse_str("12345678-1234-5678-1234-567812345678").unwrap();
        assert_eq!(
            format!("{:?}", uuid),
            "Uuid(\"12345678-1234-5678-1234-567812345678\")"
        );
    }

    #[test]
    fn test_partial_eq() {
        let uuid1 = Uuid::parse_str("11111111-2222-3333-4444-555555555555").unwrap();
        let uuid2 = Uuid::parse_str("11111111-2222-3333-4444-555555555555").unwrap();
        let uuid3 = Uuid::parse_str("66666666-7777-8888-9999-aaaaaaaaaaaa").unwrap();
        assert_eq!(uuid1, uuid2);
        assert_ne!(uuid1, uuid3);
    }

    #[test]
    fn test_hash_consistency() {
        let uuid1 = Uuid::parse_str("abcdef01-2345-6789-abcd-ef0123456789").unwrap();
        let uuid2 = Uuid::parse_str("abcdef01-2345-6789-abcd-ef0123456789").unwrap();
        let mut hasher1 = std::collections::hash_map::DefaultHasher::new();
        let mut hasher2 = std::collections::hash_map::DefaultHasher::new();
        uuid1.hash(&mut hasher1);
        uuid2.hash(&mut hasher2);
        assert_eq!(hasher1.finish(), hasher2.finish());
    }

    #[test]
    fn test_hash_used_in_hashset() {
        let uuid = Uuid::parse_str("deadbeef-1234-5678-9abc-def012345678").unwrap();
        let mut set = HashSet::new();
        set.insert(uuid.clone());
        assert!(set.contains(&uuid));
    }

    #[test]
    fn test_from_str_valid() {
        let uuid: Result<Uuid, _> = "aabbccdd-eeff-1122-3344-556677889900".parse();
        assert!(uuid.is_ok());
        assert_eq!(
            uuid.unwrap().as_str(),
            "aabbccdd-eeff-1122-3344-556677889900"
        );
    }

    #[test]
    fn test_from_str_invalid() {
        let result: Result<Uuid, _> = "invalid".parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_serialize() {
        let uuid = Uuid::parse_str("11223344-5566-7788-99aa-bbccddeeff00").unwrap();
        let serialized = serde_json::to_string(&uuid).unwrap();
        assert_eq!(serialized, "\"11223344-5566-7788-99aa-bbccddeeff00\"");
    }

    #[test]
    fn test_deserialize_valid() {
        let json = "\"11223344-5566-7788-99aa-bbccddeeff00\"";
        let uuid: Uuid = serde_json::from_str(json).unwrap();
        assert_eq!(uuid.as_str(), "11223344-5566-7788-99aa-bbccddeeff00");
    }

    #[test]
    fn test_deserialize_invalid() {
        let json = "\"not-a-uuid\"";
        let result: Result<Uuid, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_serialize_deserialize_roundtrip() {
        let uuid = Uuid::parse_str("ff00ff00-00ff-00ff-00ff-00ff00ff00ff").unwrap();
        let serialized = serde_json::to_string(&uuid).unwrap();
        let deserialized: Uuid = serde_json::from_str(&serialized).unwrap();
        assert_eq!(uuid, deserialized);
    }

    #[test]
    fn test_uuid_parse_error_display() {
        let err = UuidParseError;
        assert_eq!(format!("{}", err), "invalid UUID format");
    }

    #[test]
    fn test_uuid_parse_error_is_error() {
        let err = UuidParseError;
        let _: &dyn std::error::Error = &err;
    }

    #[test]
    fn test_default_is_nil() {
        let default_uuid = Uuid::default();
        assert_eq!(default_uuid, Uuid::nil());
    }

    #[test]
    fn test_clone() {
        let uuid = Uuid::new_v4();
        let cloned = uuid.clone();
        assert_eq!(uuid, cloned);
        assert_eq!(uuid.as_str(), cloned.as_str());
    }

    #[test]
    fn test_generate_uuid_is_valid() {
        let uuid = generate_uuid();
        assert!(glib::uuid_string_is_valid(uuid.as_str()));
    }
}
