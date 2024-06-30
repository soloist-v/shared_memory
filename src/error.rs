#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("You cannot create a shared memory mapping of 0 size")]
    MapSizeZero,
    #[error("Tried to open mapping without flink path or os_id")]
    NoLinkOrOsId,
    #[error("Creating the link file failed, {0}")]
    LinkCreateFailed(std::io::Error),
    #[error("Writing the link file failed, {0}")]
    LinkWriteFailed(std::io::Error),
    #[error("Shared memory link already exists")]
    LinkExists,
    #[error("Opening the link file failed, {0}")]
    LinkOpenFailed(std::io::Error),
    #[error("Reading the link file failed, {0}")]
    LinkReadFailed(std::io::Error),
    #[error("Requested link file does not exist")]
    LinkDoesNotExist,
    #[error("Shared memory OS specific ID already exists")]
    MappingIdExists,
    #[error("Creating the shared memory failed, os error {0}")]
    MapCreateFailed(u32),
    #[error("Opening the shared memory failed, os error {0}")]
    MapOpenFailed(u32),
    #[error("An unexpected OS error occurred, os error {0}")]
    UnknownOsError(u32),
    #[error(transparent)]
    WindowCoreError(#[from] windows::core::Error),
    #[error("map size and type is unmatched, memory size: {0}, type size: {1}")]
    MapSizeUnmatched(usize, usize),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;