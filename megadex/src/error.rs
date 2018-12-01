use bincode::ErrorKind as BinError;
use failure::Fail;
use rkv::StoreError;
use std::io::Error as IoError;
use std::sync::PoisonError;

#[derive(Debug, Fail)]
pub enum MegadexError {
    #[fail(display = "Rkv error: {}", 0)]
    RkvError(StoreError),
    #[fail(display = "Bincode error: {}", 0)]
    BincodeError(Box<BinError>),
    #[fail(display = "Std io error : {}", 0)]
    IoError(IoError),
    #[fail(display = "Read Mutex Error : {}", 0)]
    MutexError(String),
    #[fail(display = "Index {} is not defined", 0)]
    IndexUndefined(String),
    #[fail(display = "Expected type {}, found type {}", 0, 1)]
    InvalidType(String, String),
    #[fail(display = "Value error : {}", 0)]
    ValueError(String),
}

impl From<IoError> for MegadexError {
    fn from(err: IoError) -> Self {
        MegadexError::IoError(err)
    }
}

impl<T> From<PoisonError<T>> for MegadexError {
    fn from(err: PoisonError<T>) -> Self {
        MegadexError::MutexError(format!("{}", err))
    }
}

impl From<StoreError> for MegadexError {
    fn from(err: StoreError) -> Self {
        MegadexError::RkvError(err)
    }
}

impl From<Box<BinError>> for MegadexError {
    fn from(err: Box<BinError>) -> Self {
        MegadexError::BincodeError(err)
    }
}

impl PartialEq for MegadexError {
    fn eq(&self, other: &MegadexError) -> bool {
        use crate::MegadexError::*;
        match self {
            RkvError(_) => {
                if let RkvError(_) = other {
                    true
                } else {
                    false
                }
            }
            BincodeError(_) => {
                if let BincodeError(_) = other {
                    true
                } else {
                    false
                }
            }
            IoError(_) => {
                if let IoError(_) = other {
                    true
                } else {
                    false
                }
            }
            MutexError(e) => {
                if let MutexError(s) = other {
                    e == s
                } else {
                    false
                }
            }
            IndexUndefined(e) => {
                if let IndexUndefined(s) = other {
                    e == s
                } else {
                    false
                }
            }
            InvalidType(e, i) => {
                if let InvalidType(a, b) = other {
                    e == a && i == b
                } else {
                    false
                }
            }
            ValueError(e) => {
                if let ValueError(s) = other {
                    e == s
                } else {
                    false
                }
            }
        }
    }
}
