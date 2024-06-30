use windows::core::PCSTR;
use windows::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE};
use windows::Win32::System::Memory::{CreateFileMappingA, FILE_MAP_ALL_ACCESS, MapViewOfFile, MEMORY_BASIC_INFORMATION, MEMORY_MAPPED_VIEW_ADDRESS, OpenFileMappingA, PAGE_READWRITE, UnmapViewOfFile, VirtualQuery};
use crate::Error;

pub struct SharedMemory {
    name: String,
    size: usize,
    owner: bool,
    handle: HANDLE,
    ptr: MEMORY_MAPPED_VIEW_ADDRESS,
}

impl Drop for SharedMemory {
    fn drop(&mut self) {
        unsafe {
            if self.owner {
                if let Err(e) = UnmapViewOfFile(self.ptr) {
                    crate::log::error!("{e}");
                }
            }
            if let Err(e) = CloseHandle(self.handle) {
                crate::log::error!("{e}");
            }
        }
    }
}

impl SharedMemory {
    pub fn new(name: &str, size: usize, create: bool) -> crate::Result<Self> {
        let mut size = size;
        let name_c = std::ffi::CString::new(name).unwrap();
        let name_c = PCSTR::from_raw(name_c.as_ptr() as _);
        unsafe {
            let handle = if create {
                OpenFileMappingA(
                    FILE_MAP_ALL_ACCESS.0,
                    false,
                    name_c,
                )?
            } else {
                CreateFileMappingA(
                    INVALID_HANDLE_VALUE,
                    None,
                    PAGE_READWRITE,
                    0,
                    size as u32,
                    name_c,
                )?
            };
            let ptr = MapViewOfFile(handle,
                                    FILE_MAP_ALL_ACCESS, 0, 0, size);
            if !create {
                // open
                let mut info = MEMORY_BASIC_INFORMATION::default();
                let res = VirtualQuery(Some(ptr.Value as _), &mut info, size_of::<MEMORY_BASIC_INFORMATION>());
                if res == 0 {
                    let err = windows::core::Error::from_win32();
                    crate::log::error!("{err}");
                    return Err(err.into());
                }
                if size == 0 || size > info.RegionSize {
                    size = info.RegionSize;
                }
            }
            let res = Self {
                name: name.to_string(),
                size,
                owner: create,
                handle,
                ptr,
            };
            Ok(res)
        }
    }
}

impl SharedMemory {
    pub fn size(&self) -> usize {
        self.size
    }
    pub fn name(&self) -> &str {
        self.name.as_str()
    }
    pub fn owner(&self) -> bool {
        self.owner
    }
    pub fn set_owner(&mut self, owner: bool) -> bool {
        let old = self.owner;
        self.owner = owner;
        old
    }
    pub fn as_ptr(&self) -> *const std::ffi::c_void {
        self.ptr.Value as _
    }

    pub fn as_mut_ptr(&mut self) -> *mut std::ffi::c_void {
        self.ptr.Value
    }

    pub unsafe fn as_slice(&self) -> &[u8] {
        std::slice::from_raw_parts(self.as_ptr() as _, self.size)
    }
    pub unsafe fn as_mut_slice(&mut self) -> &mut [u8] {
        std::slice::from_raw_parts_mut(self.as_mut_ptr() as _, self.size)
    }

    pub unsafe fn as_slice_t<T>(&self) -> crate::Result<&[T]> {
        if self.size % size_of::<T>() == 0 {
            let a = std::slice::from_raw_parts(self.as_ptr() as _, self.size / size_of::<T>());
            return Ok(a);
        }
        Err(Error::MapSizeUnmatched(self.size, size_of::<T>()))
    }

    pub unsafe fn as_mut_slice_t<T>(&mut self) -> crate::Result<&mut [T]> {
        if self.size % size_of::<T>() == 0 {
            let a = std::slice::from_raw_parts_mut(self.as_ptr() as _, self.size / size_of::<T>());
            return Ok(a);
        }
        Err(Error::MapSizeUnmatched(self.size, size_of::<T>()))
    }
}
