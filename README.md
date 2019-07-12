# com_ptr

A smart pointer for Windows COM Interfaces

## Example
Creates a ComPtr from `CreateDXGIFactory1` function.

```rust
use winapi::shared::dxgi::*;
use winapi::um::winnt::HRESULT;
use winapi::Interface;
use com_ptr::{ComPtr, hresult};

fn create_dxgi_factory<T: Interface>() -> Result<ComPtr<T>, HRESULT> {
    ComPtr::new(|| {
        let mut obj = std::ptr::null_mut();
        let res = unsafe { CreateDXGIFactory1(&T::uuidof(), &mut obj) };
        hresult(obj as *mut T, res)
    })
}
```


## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
