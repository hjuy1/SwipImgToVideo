#[macro_export]
macro_rules! err_new {
    ($kind:expr, $message:expr) => {
        $crate::error::Error::new(file!(), line!(), column!(), $kind, $message)
    };
}

#[macro_export]
macro_rules! err_new_io {
    ($err:expr) => {
        $crate::error::Error::new(
            file!(),
            line!(),
            column!(),
            $crate::error::Kind::IoError($err.kind()),
            &$err.to_string(),
        )
    };
}

#[macro_export]
macro_rules! err_new_image {
    ($err:expr) => {
        $crate::error::Error::new(
            file!(),
            line!(),
            column!(),
            $crate::error::Kind::ImageError,
            &$err.to_string(),
        )
    };
}

#[macro_export]
macro_rules! err_new_tryfrom {
    ($err:expr) => {
        $crate::error::Error::new(
            file!(),
            line!(),
            column!(),
            $crate::error::Kind::TryFromIntError,
            &$err.to_string(),
        )
    };
}

pub struct Error {
    location: Option<String>,
    kind: Kind,
    message: String,
}

pub type Result<T> = std::result::Result<T, Error>;

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut f = f.debug_struct("Error");
        if let Some(ref location) = self.location {
            f.field("location", location);
        }
        f.field("kind", &self.kind)
            .field("message", &self.message)
            .finish()
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self {
            location: None,
            kind: Kind::IoError(value.kind()),
            message: value.to_string(),
        }
    }
}
impl From<std::num::TryFromIntError> for Error {
    fn from(value: std::num::TryFromIntError) -> Self {
        Self {
            location: None,
            kind: Kind::TryFromIntError,
            message: value.to_string(),
        }
    }
}
impl From<&str> for Error {
    fn from(value: &str) -> Self {
        Self {
            location: None,
            kind: Kind::Other,
            message: value.to_string(),
        }
    }
}

impl Error {
    #[must_use]
    pub fn new(file: &str, line: u32, column: u32, kind: Kind, message: &str) -> Self {
        Self {
            location: Some(format!("{file}:{line}:{column}")),
            kind,
            message: message.to_string(),
        }
    }
}

#[derive(Debug)]
pub enum Kind {
    IoError(std::io::ErrorKind),
    ImageError,
    InvalidFont,
    BigImgBuilderError,
    TryFromIntError,
    Other,
}
