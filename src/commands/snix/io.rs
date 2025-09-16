use crate::nixpkgs::NIXPKGS_PATH;
use bytes::Bytes;
use snix_eval::{EvalIO, FileType};
use std::ffi::{OsStr, OsString};
use std::io::Read;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::{fs, io};

pub struct NixpkgsIo;

impl NixpkgsIo {
    fn ensure_inside(path: &Path) -> io::Result<PathBuf> {
        let abs = if path.is_absolute() {
            path.to_path_buf()
        } else {
            NIXPKGS_PATH.join(path)
        };
        let canon = abs.canonicalize()?;
        if !canon.starts_with(&*NIXPKGS_PATH) {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                format!(
                    "Path {:?} is outside nixpkgs root",
                    canon.to_str().or(Some("???"))
                ),
            ));
        }
        Ok(canon)
    }
}

impl EvalIO for NixpkgsIo {
    fn path_exists(&self, path: &Path) -> io::Result<bool> {
        let exists = match Self::ensure_inside(path) {
            Ok(path) => path.exists(),
            Err(_) => false,
        };
        Ok(exists)
    }

    fn open(&self, path: &Path) -> io::Result<Box<dyn Read>> {
        let path = Self::ensure_inside(path)?;
        Ok(Box::new(fs::File::open(path)?))
    }

    fn file_type(&self, path: &Path) -> io::Result<FileType> {
        let path = Self::ensure_inside(path)?;
        let meta = fs::metadata(path)?;
        if meta.is_file() {
            Ok(FileType::Regular)
        } else if meta.is_dir() {
            Ok(FileType::Directory)
        } else {
            Ok(FileType::Symlink)
        }
    }

    fn read_dir(&self, path: &Path) -> io::Result<Vec<(Bytes, FileType)>> {
        let path = Self::ensure_inside(path)?;
        let mut out = Vec::new();
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let name = entry.file_name();
            let ftype = entry.file_type()?;
            let kind = if ftype.is_file() {
                FileType::Regular
            } else if ftype.is_dir() {
                FileType::Directory
            } else {
                FileType::Symlink
            };
            out.push((Bytes::from(name.as_bytes().to_owned()), kind));
        }
        Ok(out)
    }

    fn import_path(&self, path: &Path) -> io::Result<PathBuf> {
        Self::ensure_inside(path)
    }

    fn get_env(&self, key: &OsStr) -> Option<OsString> {
        std::env::var_os(key)
    }
}
