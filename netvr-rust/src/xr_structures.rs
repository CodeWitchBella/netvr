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

pub enum DecodedStructData {
    EventDataSessionStateChanged(openxr_sys::EventDataSessionStateChanged),
    EventDataInteractionProfileChanged(openxr_sys::EventDataInteractionProfileChanged),
    Unknown,
}

pub struct DecodedStruct {
    pub ty: openxr_sys::StructureType,
    pub data: DecodedStructData,
}

impl DecodedStruct {
    fn from(arg: *const openxr_sys::BaseInStructure) -> DecodedStruct {
        use DecodedStructData::*;
        let ty = unsafe { *arg }.ty;

        macro_rules! implement {
            ($type: pat, $id: ident) => {
                if let $type = ty {
                    return DecodedStruct {
                        ty,
                        data: $id(unsafe {
                            *std::mem::transmute::<
                                *const openxr_sys::BaseInStructure,
                                *const openxr_sys::$id,
                            >(arg)
                        }),
                    };
                }
            };
        }
        if let openxr_sys::StructureType::EVENT_DATA_SESSION_STATE_CHANGED = ty {
            return DecodedStruct {
                ty,
                data: EventDataSessionStateChanged(unsafe {
                    *std::mem::transmute::<
                        *const openxr_sys::BaseInStructure,
                        *const openxr_sys::EventDataSessionStateChanged,
                    >(arg)
                }),
            };
        }
        use openxr_sys::StructureType;
        implement!(
            StructureType::EVENT_DATA_SESSION_STATE_CHANGED,
            EventDataSessionStateChanged
        );
        implement!(
            StructureType::EVENT_DATA_INTERACTION_PROFILE_CHANGED,
            EventDataInteractionProfileChanged
        );

        DecodedStruct { ty, data: Unknown }
    }
}
