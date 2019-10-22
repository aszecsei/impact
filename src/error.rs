use failure::Fail;

#[derive(Debug, Fail)]
pub enum ImpactError {
    #[fail(display = "invalid padding size: {}", size)]
    InvalidPadding {
        size: u8,
    },
    #[fail(display = "I/O error: {}", err)]
    IoError {
        err: std::io::Error,
    },
    #[fail(display = "Image error: {}", err)]
    ImageError {
        err: image::ImageError,
    }
}

impl From<std::io::Error> for ImpactError {
    fn from(err: std::io::Error) -> ImpactError {
        ImpactError::IoError { err }
    }
}

impl From<image::ImageError> for ImpactError {
    fn from(err: image::ImageError) -> ImpactError {
        ImpactError::ImageError { err }
    }
}

pub type Result<T> = std::result::Result<T, ImpactError>;