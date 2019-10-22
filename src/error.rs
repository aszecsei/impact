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
    },
    #[fail(display = "can't fit image in atlas")]
    CantFitError,
    #[fail(display = "xml error: {}", err)]
    XmlError {
        err: xml::writer::Error
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

impl From<xml::writer::Error> for ImpactError {
    fn from(err: xml::writer::Error) -> ImpactError {
        ImpactError::XmlError { err }
    }
}

pub type Result<T> = std::result::Result<T, ImpactError>;