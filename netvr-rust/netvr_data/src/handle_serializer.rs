#![allow(dead_code)]

macro_rules! handle {
    ($mod:ident, $id:ident) => {
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
