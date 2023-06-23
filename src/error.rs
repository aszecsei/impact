use thiserror::Error;

#[derive(Debug, Error)]
pub enum ImpactError {
    #[error("invalid padding size: {}", size)]
    InvalidPadding {
        size: u8,
    },
    #[error("I/O error: {}", err)]
    IoError {
        err: std::io::Error,
    },
    #[error("Image error: {}", err)]
    ImageError {
        err: image::ImageError,
    },
    #[error("can't fit image in atlas")]
    CantFitError,
    #[error("xml error: {}", err)]
    XmlError {
        err: xml::writer::Error
    },
    #[error("log error: {}", err)]
    LoggerError {
        err: log::SetLoggerError
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

impl From<log::SetLoggerError> for ImpactError {
    fn from(err: log::SetLoggerError) -> ImpactError {
        ImpactError::LoggerError { err }
    }
}

pub type Result<T> = std::result::Result<T, ImpactError>;