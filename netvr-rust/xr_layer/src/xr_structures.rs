pub struct XrIterator {
    ptr: *const openxr_sys::BaseInStructure,
}

macro_rules! implement {
    ($method: ident for $t: ty) => {
        pub fn event_data_buffer(input: *const $t) -> XrIterator {
            XrIterator {
                ptr: unsafe { std::mem::transmute(input) },
            }
        }
    };
}

impl XrIterator {
    implement!(event_data_buffer for openxr_sys::EventDataBuffer);
}

impl Iterator for XrIterator {
    type Item = DecodedStruct;

    fn next(&mut self) -> Option<Self::Item> {
        let res: *const openxr_sys::BaseInStructure = unsafe { std::mem::transmute(self.ptr) };
        if res.is_null() {
            return None;
        }
        self.ptr = unsafe { std::mem::transmute((*res).next) };
        Some(DecodedStruct::from(res))
    }
}

pub struct DecodedStruct {
    pub ty: openxr_sys::StructureType,
    data: *const openxr_sys::BaseInStructure,
}

macro_rules! implement {
    ($( $method: ident reads $id: ident ), *,) => {
        impl DecodedStruct {
            $(
                #[allow(dead_code)]
                pub fn $method(&self) -> Option<openxr_sys::$id> {
                    if self.data.is_null() { return None; }
                    Some(unsafe {
                        *std::mem::transmute::<
                            *const openxr_sys::BaseInStructure,
                            *const openxr_sys::$id,
                        >(self.data)
                    })
                }
            )*
        }

    };
}

implement!(
    into_event_data_session_state_changed reads EventDataSessionStateChanged,
    into_event_data_interaction_profile_changed reads EventDataInteractionProfileChanged,
    into_event_data_buffer reads EventDataBuffer,
);

impl DecodedStruct {
    fn from(arg: *const openxr_sys::BaseInStructure) -> Self {
        let ty = unsafe { *arg }.ty;

        Self { ty, data: arg }
    }
}
