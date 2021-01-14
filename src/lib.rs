//! A smart pointer for Windows COM Interfaces.
//!
//! # Examples
//! Creates a ComPtr from `CreateDXGIFactory1` function.
//!
//! ```
//! use winapi::shared::dxgi::*;
//! use winapi::um::winnt::HRESULT;
//! use winapi::Interface;
//! use com_ptr::{ComPtr, HResult, hresult};
//!
//! fn create_dxgi_factory<T: Interface>() -> Result<ComPtr<T>, HResult> {
//!     ComPtr::new(|| {
//!         let mut obj = std::ptr::null_mut();
//!         let res = unsafe { CreateDXGIFactory1(&T::uuidof(), &mut obj) };
//!         hresult(obj as *mut T, res)
//!     })
//! }
//! ```
//!
#![cfg(windows)]

use std::ops::Deref;
use std::ptr::{null_mut, NonNull};
use winapi::shared::guiddef::REFCLSID;
use winapi::shared::minwindef::DWORD;
use winapi::um::combaseapi::CoCreateInstance;
use winapi::um::unknwnbase::IUnknown;
use winapi::um::winnt::HRESULT;
use winapi::um::winbase::*;
use winapi::Interface;

/// A object that wraps HRESULT.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(transparent)]
pub struct HResult(pub HRESULT);

impl HResult {
    #[inline]
    pub fn is_succeed(&self) -> bool {
        self.0 >= 0
    }
    
    #[inline]
    pub fn is_failed(&self) -> bool {
        self.0 < 0
    }
    
    #[inline]
    pub fn code(&self) -> HRESULT {
        self.0
    }
}

impl std::fmt::Display for HResult {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        unsafe {
            let mut p: *mut u16 = std::ptr::null_mut();
            let len = FormatMessageW(
                FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_SYSTEM,
                std::ptr::null(),
                self.0 as u32,
                0,
                std::mem::transmute(&mut p),
                0,
                std::ptr::null_mut()
            );
            let buffer = std::slice::from_raw_parts(p, len as usize);
            let ret = write!(f, "{}", String::from_utf16_lossy(buffer));
            LocalFree(p as _);
            ret
        }
    }
}

impl std::error::Error for HResult {}

/// Returns a object when success.
///
/// If `res` is success, returns a object. OtherWise, returns a HResult object.
pub fn hresult<T>(obj: T, res: HRESULT) -> Result<T, HResult> {
    if res < 0 {
        Err(HResult(res))
    } else {
        Ok(obj)
    }
}

/// A smart pointer for COM Interfaces.
pub struct ComPtr<T: Interface> {
    p: NonNull<T>,
}

impl<T: Interface> ComPtr<T> {
    /// Creates a new ComPtr from a closure.
    ///
    /// ## Safety
    /// 'f' must returns non-null.
    pub fn new<F, E>(f: F) -> Result<ComPtr<T>, E>
    where
        F: FnOnce() -> Result<*mut T, E>,
    {
        unsafe { Ok(ComPtr::from_raw(f()?)) }
    }

    /// Creates a new ComPtr from a raw pointer.
    ///
    /// ## Safety
    /// 'ptr' must be non-null.
    #[inline]
    pub unsafe fn from_raw(ptr: *mut T) -> ComPtr<T> {
        ComPtr {
            p: NonNull::new(ptr).expect("ComPtr should not be null."),
        }
    }

    /// Returns a pointer
    #[inline]
    pub fn as_ptr(&self) -> *mut T {
        self.p.as_ptr()
    }

    /// Returns a reference
    #[inline]
    pub fn as_ref(&self) -> &T {
        unsafe { self.p.as_ref() }
    }

    /// Returns a `ComPtr<U>` when interface `T` support interface `U`.
    pub fn query_interface<U: Interface>(&self) -> Result<ComPtr<U>, HResult> {
        unsafe {
            let mut p = null_mut();
            let res = self.as_unknown().QueryInterface(&U::uuidof(), &mut p);
            hresult(ComPtr::from_raw(p as *mut U), res)
        }
    }

    #[inline]
    fn as_unknown(&self) -> &IUnknown {
        unsafe { &*(self.as_ptr() as *mut IUnknown) }
    }

    #[inline]
    fn add_ref(&self) {
        unsafe { self.as_unknown().AddRef() };
    }

    #[inline]
    fn release(&self) {
        unsafe { self.as_unknown().Release() };
    }
}

impl<T: Interface> Deref for ComPtr<T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.as_ref()
    }
}

impl<T: Interface> Clone for ComPtr<T> {
    fn clone(&self) -> Self {
        self.add_ref();
        ComPtr { p: self.p.clone() }
    }
}

impl<T: Interface> Drop for ComPtr<T> {
    fn drop(&mut self) {
        self.release();
    }
}

impl<T: Interface> PartialEq for ComPtr<T> {
    fn eq(&self, other: &Self) -> bool {
        self.as_ptr() == other.as_ptr()
    }
}

impl<T: Interface> Eq for ComPtr<T> {}

impl<T: Interface> PartialOrd for ComPtr<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.as_ptr().partial_cmp(&other.as_ptr())
    }
}

impl<T: Interface> Ord for ComPtr<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_ptr().cmp(&other.as_ptr())
    }
}

impl<T: Interface> std::fmt::Debug for ComPtr<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self.as_ptr())
    }
}

unsafe impl<T: Interface> Send for ComPtr<T> {}
unsafe impl<T: Interface> Sync for ComPtr<T> {}

/// Creates a ComPtr of the class associated with a specified CLSID.
pub fn co_create_instance<T: Interface>(
    clsid: REFCLSID,
    outer: Option<*mut IUnknown>,
    clsctx: DWORD,
) -> Result<ComPtr<T>, HResult> {
    ComPtr::new(|| {
        let mut obj = null_mut();
        let outer = match outer {
            Some(p) => p,
            None => null_mut(),
        };
        let res = unsafe { CoCreateInstance(clsid, outer, clsctx, &T::uuidof(), &mut obj) };
        hresult(obj as *mut T, res)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use winapi::shared::wtypesbase::CLSCTX_INPROC_SERVER;
    use winapi::um::objbase::CoInitialize;
    use winapi::um::wincodec::*;

    #[test]
    fn co_create_instance_test() {
        unsafe { CoInitialize(null_mut()) };

        let p = co_create_instance::<IWICImagingFactory>(
            &CLSID_WICImagingFactory,
            None,
            CLSCTX_INPROC_SERVER,
        );
        if let Err(res) = p {
            panic!("HRESULT: 0x{:<08x}", res.code());
        }
        assert!(p == p);
        assert!(p <= p);
        println!("{:?}", p);
    }
    
    fn ret_result() -> Result<(), HResult> {
        Ok(())
    }
    
    #[test]
    fn anyhow_test() {
        fn result() -> anyhow::Result<()> {
            Ok(ret_result()?)
        }
        result().ok().unwrap();
    }
    
    #[test]
    #[ignore]
    fn display_test() {
        let ret = HResult(0);
        println!("0x{:<08x} {}", ret.code(), ret);
    }
}
