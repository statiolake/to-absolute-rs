use std::env;
use std::error;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf, Prefix};
use std::result;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    CurrentIsRelative,
    UnsupportedPrefix,
    IoError(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, b: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::CurrentIsRelative => write!(
                b,
                "the path specified as current directory was relative path."
            ),
            Error::UnsupportedPrefix => {
                write!(b, "the path specified has the prefix that isn't supported.")
            }
            Error::IoError(ref e) => write!(b, "io::Error happened: {}", e),
        }
    }
}

impl error::Error for Error {}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::IoError(e)
    }
}

/// get the absolute path for specified file.
/// Note: the file must exist.
pub fn to_absolute(current: impl AsRef<Path>, relative: impl AsRef<Path>) -> Result<PathBuf> {
    let current = current.as_ref();
    let relative = relative.as_ref();
    if relative.is_absolute() {
        return Ok(relative.to_path_buf());
    }
    if !current.is_absolute() {
        return Err(Error::CurrentIsRelative);
    }

    // here: current is absolute path, relative is relative path.
    let joined = current.join(relative);

    canonicalize(joined)
}

/// get the absolute path for specified file, relative to current working
/// directory.
pub fn to_absolute_from_current_dir(relative: impl AsRef<Path>) -> Result<PathBuf> {
    let current_dir = env::current_dir()?;
    to_absolute(current_dir, relative)
}

fn canonicalize(path: impl AsRef<Path>) -> Result<PathBuf> {
    let canonicalized = fs::canonicalize(path.as_ref())?;
    let components = canonicalized.components().map(|component| match component {
        Component::Prefix(prefix) => match prefix.kind() {
            Prefix::Disk(disk) | Prefix::VerbatimDisk(disk) => {
                let disk = disk as char;
                Ok(format!("{}:", disk).into())
            }
            _ => return Err(Error::UnsupportedPrefix),
        },
        other => Ok(other.as_os_str().to_os_string()),
    });

    components.collect()
}

#[cfg(test)]
mod tests {
    use super::to_absolute;
    use super::Result;

    fn toabs(cur: &str, rel: &str) -> Result<String> {
        to_absolute(cur, rel).map(|x| x.display().to_string())
    }

    #[test]
    fn test_supported() {
        assert_eq!(
            r#"C:\Windows\System32"#,
            toabs(r#"C:\"#, r#".\Windows\System32"#).unwrap()
        );

        assert_eq!(
            r#"C:\Windows\System32"#,
            toabs(r#"C:\Program Files"#, r#"..\Windows\System32"#).unwrap()
        );

        assert_eq!(
            r#"C:\Windows\System32"#,
            toabs(
                r#"C:\Program Files\..\Windows\Fonts"#,
                r#"..\..\Windows\System32"#
            )
            .unwrap()
        );
    }

    #[test]
    fn test_unsupported() {
        assert!(toabs(r#"\\?\pictures"#, r#".\Windows\System32"#).is_err());

        // DOS Device Path Syntax must not have `.` or `..` or something...
        assert!(toabs(r#"\\?\C:\"#, r#".\Windows\System32"#).is_err());
    }
}
