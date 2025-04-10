use std::fs::File;
use std::io;

// Structure to hold file stats for detecting file changes
#[cfg(unix)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FileId {
    pub dev: u64, // device ID
    pub ino: u64, // inode number
}

#[cfg(windows)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FileId {
    pub volume_serial_number: u32,
    pub file_index: u64,
}

// Get a unique file identifier that survives across renames
#[cfg(unix)]
pub fn get_file_id(file: &File) -> io::Result<FileId> {
    use std::os::unix::fs::MetadataExt;
    let metadata = file.metadata()?;
    Ok(FileId {
        dev: metadata.dev(),
        ino: metadata.ino(),
    })
}

#[cfg(windows)]
pub fn get_file_id(file: &File) -> io::Result<FileId> {
    use std::os::windows::fs::MetadataExt;
    let metadata = file.metadata()?;
    Ok(FileId {
        volume_serial_number: metadata.volume_serial_number(),
        file_index: metadata.file_index(),
    })
} 