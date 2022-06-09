# netvr-rust

Implements openxr layer-style interface meant to synchronize xr devices across network in a way that is (except for setup) transparent to host application.

## Lifecycle

Before doing any openxr calls and after loading openxr-loader the app calls `netvr_hook_get_instance_proc_addr` with `xrGetInstanceProcAddr` pointer it got from openxr-loader.
After that point the application calls the resulting function pointer instead of `xrGetInstanceProcAddr`. This means that netvr has option to insert itself between application and openxr runtime when needed without explicit support from application. This assumes that application uses `xrGetInstanceProcAddr` exclusively instead of trampolines provided by openxr-loader.

To support openxr-loader we would have to expose proper openxr layer, which is definitely possible, but since loader is not really supported on android it is unnecessary.

## Public API

The .dll (or .so on non-windows platforms) has three exported functions - `netvr_hook_get_instance_proc_addr`, `netvr_manual_destroy_instance` and `netvr_set_logger`.

**netvr_hook_get_instance_proc_addr** is the most important function. You call this function before you call any openxr functions (see lifecycle) and it sets everything up automatically. You should set `automatic_destroy` to 1. It has following signature:

```c
void *netvr_hook_get_instance_proc_addr(void *func, bool automatic_destroy);
// First argument and return type are of the same type as xrGetInstanceProcAddr
// from OpenXR spec.
```

**netvr_set_logger** let's you set up log handler, which allows you to redirect logging from netvr from standard output to anywhere you choose. The default logger is not terribly efficient, so you might want to do this even in case you want to print to stdout.

```c++
#include <stdint.h>
typedef void (*netvr_logger_callback)(int32_t level, const char *);
void netvr_set_logger(netvr_logger_callback callback);
// level is one of: 1 - Info, 2 - Warn, 3 - Error
```

**netvr_manual_destroy_instance** if you do library unloading on instance destroy then your application would crash. To work around this you can set `automatic_destroy` parameter to hook function to `0`. That will prevent it from taking over `xrDestroyInstance` which will allow you to unload the library before that function is called.

That unfortunately means that you have to finalize library yourself by calling this `netvr_manual_destroy_instance` yourself.

```c
void netvr_manual_destroy_instance(XrInstance instance);
// instance is OpenXR instance handle
```

## Overriding a function

**Step 1** Add definition to `XrInstanceFunctions` struct. This is technically not a prerequisite for overriding a function, but you will probably want to call the original function, which this allows you to do.

```rust
implement!(
    // ... existing fields
    pub poll_event: pfn::PollEvent,
)
```

**Step 2** Define a function to override your chosen function with. See relevant definition in [openxr_sys's docs](https://docs.rs/openxr-sys/latest/openxr_sys/pfn/index.html) to get required signature. I use prefix `override_` for those functions. Depending on which (if any) handle the function uses you will have to choose correct function.

You can use `xr_wrap` helper to simplify your implementation. It takes closure, calls it and converts its return value from `Result<(), openxr_sys::Result>` to `openxr_sys::Result`. This allows you to use the `?` operator.

Example boilerplate:

```rust
extern "system" fn override_poll_event(
    instance_handle: openxr_sys::Instance,
    event_data: *mut openxr_sys::EventDataBuffer,
) -> openxr_sys::Result {
    xr_wrap(|| {
        let instance = get_instance("xrPollEvent", instance_handle)?;
        unsafe { (instance.poll_event)(instance_handle, event_data) }
            .into_result()
    })
}
```

**Step 3** Add a check to `override_get_instance_proc_addr`. This will error out if your `override_` function has wrong signature.

```rust
check!(pfn::PollEvent, Self::override_poll_event);
```

**Step 4** Test that an app which uses this function does not crash at this point. Steps above should not change the behavior.

**Step 5** Implement your override.

## Compiling for Quest

Make sure that your rust and unity installations are located in path without spaces. One way to do that for rust is to set following environment variables system-wide:

- `CARGO_HOME` to `C:\Stuff\Rust\cargo`
- `RUSTUP_HOME` to `C:\Stuff\Rust\rustup`

And only after doing so installing rust using rustup. Alternatively you can use WSL.

Then to build the project you have to create `.cargo/config.toml` file containing correct paths. Use `.cargo/config-example.toml` as a guide.

```bash
# Install rust target for android
rustup target add aarch64-linux-android
# Compile
cargo build --target aarch64-linux-android
```
