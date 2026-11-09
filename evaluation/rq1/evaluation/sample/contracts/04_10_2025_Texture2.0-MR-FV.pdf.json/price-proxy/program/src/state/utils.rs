pub fn str_to_array<const SIZE: usize>(str: &str) -> [u8; SIZE] {
    let bytes = str.as_bytes();
    let mut array = [0; SIZE];
    array[..bytes.len()].copy_from_slice(bytes);
    array
}

#[cfg(not(target_os = "solana"))]
pub fn bytes_to_cow(bytes: &[u8]) -> std::borrow::Cow<'_, str> {
    std::ffi::CStr::from_bytes_until_nul(bytes)
        .ok()
        .map(|x| x.to_string_lossy())
        .unwrap_or_else(|| String::from_utf8_lossy(bytes))
}

#[cfg(feature = "with-serde")]
pub mod with_serde {
    pub use display_json::*;
    pub use serde::{Deserialize, Serialize};
    pub use serde_with::{serde_as, As, DisplayFromStr, FromInto};

    pub use texture_common::math::decimal::decimal_bits_serde;

    use super::{bytes_to_cow, str_to_array};

    pub mod array_as_str_serde {
        pub fn serialize<S, const SIZE: usize>(
            array: &[u8; SIZE],
            serializer: S,
        ) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            let cow = super::bytes_to_cow(array);
            serde::Serialize::serialize(&cow, serializer)
        }

        pub fn deserialize<'de, D, const SIZE: usize>(
            deserializer: D,
        ) -> Result<[u8; SIZE], D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let str: &str = serde::Deserialize::deserialize(deserializer)?;
            let array = super::str_to_array(str);
            Ok(array)
        }
    }

    pub mod timestamp_as_datetime_serde {
        pub fn serialize<S>(timestamp: &i64, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            use chrono::{DateTime, Utc};
            let dt = DateTime::<Utc>::from_timestamp(*timestamp, 0);
            serde::Serialize::serialize(&dt, serializer)
        }

        pub fn deserialize<'de, D>(deserializer: D) -> Result<i64, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            use chrono::{DateTime, Utc};
            let dt: DateTime<Utc> = serde::Deserialize::deserialize(deserializer)?;
            Ok(dt.timestamp())
        }
    }

    pub mod display_from_str_serde {
        use std::{fmt::Display, str::FromStr};

        pub fn serialize<T, S>(this: &T, serializer: S) -> Result<S::Ok, S::Error>
        where
            T: Display,
            S: serde::Serializer,
        {
            let str = this.to_string();
            serde::Serialize::serialize(&str, serializer)
        }

        pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
        where
            T: FromStr,
            T::Err: Display,
            D: serde::Deserializer<'de>,
        {
            let str: &str = serde::Deserialize::deserialize(deserializer)?;
            T::from_str(str).map_err(serde::de::Error::custom)
        }
    }
}

macro_rules! source_enum_from_str_derive_infallibale {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $(#[$unk_meta:meta])*
            Unknown = 0,
            $(
                #[from_str($($from_str:literal),+ $(,)?)]
                $(#[$item_meta:meta])*
                $item:ident $(= $item_val:literal)?
            ),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        $vis enum $name {
            $(#[$unk_meta])*
            Unknown = 0,
            $( $(#[$item_meta])* $item $(= $item_val)?, )+
        }

        impl FromStr for PriceFeedSource {
            type Err = Infallible;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let s = s.to_lowercase();
                Ok(match s.as_str() {
                    $( $($from_str)|+ => Self::$item, )+
                    _ => Self::Unknown,
                })
            }
        }
    }
}
pub(crate) use source_enum_from_str_derive_infallibale;
