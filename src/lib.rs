extern crate winapi;

use std::ops::Deref;
use std::ptr::{null_mut, NonNull};
use winapi::shared::guiddef::REFCLSID;
use winapi::shared::minwindef::DWORD;
use winapi::um::combaseapi::CoCreateInstance;
use winapi::um::unknwnbase::IUnknown;
use winapi::um::winnt::HRESULT;
use winapi::Interface;

pub fn hresult<T>(obj: T, res: HRESULT) -> Result<T, HRESULT> {
    if res < 0 {
        Err(res)
    } else {
        Ok(obj)
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ComPtr<T: Interface> {
    p: NonNull<T>,
}

impl<T: Interface> ComPtr<T> {
    pub fn new<F, E>(f: F) -> Result<ComPtr<T>, E>
    where
        F: FnOnce() -> Result<*mut T, E>,
    {
        unsafe { Ok(ComPtr::from_raw(f()?)) }
    }

    #[inline]
    pub unsafe fn from_raw(p: *mut T) -> ComPtr<T> {
        ComPtr {
            p: NonNull::new(p).expect("ComPtr should not be null."),
        }
    }

    #[inline]
    pub fn as_ptr(&self) -> *mut T {
        self.p.as_ptr()
    }

    #[inline]
    pub fn as_ref(&self) -> &T {
        unsafe { self.p.as_ref() }
    }

    pub fn query_interface<U: Interface>(&self) -> Result<ComPtr<U>, HRESULT> {
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

unsafe impl<T: Interface> Send for ComPtr<T> {}
unsafe impl<T: Interface> Sync for ComPtr<T> {}

pub fn co_create_instance<T: Interface>(
    clsid: REFCLSID,
    outer: Option<*mut IUnknown>,
    clsctx: DWORD,
) -> Result<ComPtr<T>, HRESULT> {
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
            panic!("HRESULT: 0x{:<08x}", res);
        }
    }
}
