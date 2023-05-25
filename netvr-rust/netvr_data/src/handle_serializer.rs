#![allow(dead_code)]

/// Create a handle serializer so that raw openxr handles can be used in serde
/// structs instead of having to drop down to u64.
macro_rules! handle {
    ($mod:ident, $id:ident) => {
        #[cfg(not(target_arch = "wasm32"))]
        pub mod $mod {
            use serde::{self, Deserialize, Deserializer, Serializer};

            pub fn serialize<S>(val: &openxr_sys::$id, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.serialize_u64(val.into_raw())
            }

            pub fn deserialize<'de, D>(deserializer: D) -> Result<openxr_sys::$id, D::Error>
            where
                D: Deserializer<'de>,
            {
                Ok(openxr_sys::$id::from_raw(u64::deserialize(deserializer)?))
            }
        }
    };
}

handle!(instance, Instance);
handle!(session, Session);
handle!(action_set, ActionSet);
handle!(action, Action);
