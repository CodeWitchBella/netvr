use std::os::raw::c_char;
pub type Cstr = *const c_char;

#[macro_export]
macro_rules! internal_screaming {
    () => {
        panic!("AAAAAAAAAAAaaaaaaaaaaaaaaaaaaaaaaaaa");
    };
}
