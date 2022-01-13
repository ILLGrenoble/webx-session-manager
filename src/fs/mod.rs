use std::ffi::CString;
use std::fs;
use std::fs::{OpenOptions, Permissions};
use std::os::unix::fs::PermissionsExt;

use crate::common::ApplicationError;

pub fn chown(path: &str, uid: u32, gid: u32) -> Result<(), ApplicationError> {
    let cpath =
        CString::new(path).map_err(|error| ApplicationError::environment(format!("{}", error)))?;
    match unsafe { libc::chown(cpath.as_ptr(), uid, gid) } {
        0 => Ok(()),
        code => Err(ApplicationError::environment(format!(
            "Error changing ownership of file {}: {}",
            path, code
        ))),
    }
}

pub fn mkdir(path: &str) -> Result<(), ApplicationError> {
    if fs::create_dir_all(path).is_err() {
        return Err(ApplicationError::environment(format!(
            "Could create directory for path: {}",
            path
        )));
    }
    Ok(())
}

pub fn chmod(path: &str, mode: u32) -> Result<(), ApplicationError> {
    let mode = Permissions::from_mode(mode);
    if fs::set_permissions(path, mode).is_err() {
        return Err(ApplicationError::environment(format!(
            "Could not change permissions: {}",
            path
        )));
    }
    Ok(())
}

pub fn touch(path: &str) -> Result<(), ApplicationError> {
    if OpenOptions::new()
        .create_new(true)
        .write(true)
        .append(true)
        .open(path)
        .is_err()
    {
        return Err(ApplicationError::environment(format!(
            "Could not create file: {}",
            path
        )));
    }
    Ok(())
}
