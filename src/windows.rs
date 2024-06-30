use std::fs::{File, OpenOptions};
use std::io::ErrorKind;
use std::os::windows::{fs::OpenOptionsExt, io::AsRawHandle};
use std::path::PathBuf;
use windows::Win32::System::Memory::{MapViewOfFile, MEMORY_BASIC_INFORMATION, VirtualQuery, MEMORY_MAPPED_VIEW_ADDRESS};
use windows::Win32::Foundation::{HANDLE};

use crate::{log::*, ShmemConf};
use crate::Error;
use crate::windows_utils::*;

#[derive(Clone, Default)]
pub struct ShmemConfExt {
    allow_raw: bool,
}

impl ShmemConf {
    /// If set to true, enables openning raw shared memory that is not managed by this crate
    pub fn allow_raw(mut self, allow: bool) -> Self {
        self.ext.allow_raw = allow;
        self
    }
}

pub struct MapData {
    pub view: SharedMemory,
}

impl MapData {
    pub fn set_owner(&mut self, is_owner: bool) -> bool {
        self.view.set_owner(is_owner)
    }
    pub fn as_mut_ptr(&self) -> *mut u8 {
        self.view.as_ptr() as _
    }

    pub fn name(&self) -> &str {
        self.view.name()
    }

    pub fn map_size(&self) -> usize {
        self.view.size()
    }
}

/// Returns the path to a temporary directory in which to store files backing the shared memory. If it
/// doesn't exist, the directory is created.
fn get_tmp_dir() -> Result<PathBuf, Error> {
    debug!("Getting & creating shared_memory-rs temp dir");
    let mut path = std::env::temp_dir();
    path.push("shared_memory-rs");

    if path.is_dir() {
        return Ok(path);
    }

    match std::fs::create_dir_all(path.as_path()) {
        Ok(_) => Ok(path),
        Err(e) if e.kind() == ErrorKind::AlreadyExists => Ok(path),
        Err(e) => Err(Error::UnknownOsError(e.raw_os_error().unwrap() as _)),
    }
}

fn new_map(
    unique_id: &str,
    map_size: usize,
    create: bool,
    _allow_raw: bool,
) -> crate::Result<MapData> {
    let view = SharedMemory::new(unique_id, map_size, create)?;
    Ok(MapData {
        view,
    })
}

//Creates a mapping specified by the uid and size
pub fn create_mapping(unique_id: &str, map_size: usize) -> Result<MapData, Error> {
    new_map(unique_id, map_size, true, false)
}

//Opens an existing mapping specified by its uid
pub fn open_mapping(
    unique_id: &str,
    map_size: usize,
    ext: &ShmemConfExt,
) -> Result<MapData, Error> {
    new_map(unique_id, map_size, false, ext.allow_raw)
}
