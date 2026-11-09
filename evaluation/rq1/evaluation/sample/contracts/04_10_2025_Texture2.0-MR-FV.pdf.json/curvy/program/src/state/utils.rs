pub fn str_to_array<const SIZE: usize>(str: &str) -> [u8; SIZE] {
    let bytes = str.as_bytes();
    let mut array = [0; SIZE];
    array[..bytes.len()].copy_from_slice(bytes);
    array
}

#[cfg(not(target_os = "solana"))]
#[cfg(feature = "with-serde")]
pub fn bytes_to_cow(bytes: &[u8]) -> std::borrow::Cow<'_, str> {
    std::ffi::CStr::from_bytes_until_nul(bytes)
        .ok()
        .map(|x| x.to_string_lossy())
        .unwrap_or_else(|| String::from_utf8_lossy(bytes))
}

#[cfg(feature = "with-serde")]
pub mod array_as_str_serde {
    use super::{bytes_to_cow, str_to_array};

    pub fn serialize<S, const SIZE: usize>(
        array: &[u8; SIZE],
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let cow = bytes_to_cow(array);
        serde::Serialize::serialize(&cow, serializer)
    }

    pub fn deserialize<'de, D, const SIZE: usize>(deserializer: D) -> Result<[u8; SIZE], D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let str: &str = serde::Deserialize::deserialize(deserializer)?;
        let array = str_to_array(str);
        Ok(array)
    }
}

#[cfg(feature = "with-serde")]
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
