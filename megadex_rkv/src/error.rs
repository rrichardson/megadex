use bincode::ErrorKind as BinError;
use failure::Fail;
use rkv::{ StoreError, LmdbError };
use std::io::Error as IoError;
use std::sync::PoisonError;

#[derive(Debug, Fail)]
pub enum MegadexDbError {
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
    #[fail(display = "Lmdb Error : {}", 0)]
    LmdbError(LmdbError)
}

impl From<IoError> for MegadexDbError {
    fn from(err: IoError) -> Self {
        MegadexDbError::IoError(err)
    }
}

impl<T> From<PoisonError<T>> for MegadexDbError {
    fn from(err: PoisonError<T>) -> Self {
        MegadexDbError::MutexError(format!("{}", err))
    }
}

impl From<StoreError> for MegadexDbError {
    fn from(err: StoreError) -> Self {
        MegadexDbError::RkvError(err)
    }
}
impl From<LmdbError> for MegadexDbError {
    fn from(err: LmdbError) -> Self {
        MegadexDbError::LmdbError(err)
    }
}

impl From<Box<BinError>> for MegadexDbError {
    fn from(err: Box<BinError>) -> Self {
        MegadexDbError::BincodeError(err)
    }
}

impl PartialEq for MegadexDbError {
    fn eq(&self, other: &MegadexDbError) -> bool {
        use crate::MegadexDbError::*;
        match self {
            RkvError(_) => {
                if let RkvError(_) = other {
                    true
                } else {
                    false
                }
            },
            LmdbError(_) => {
                if let LmdbError(_) = other {
                    true
                } else {
                    false
                }
            },
            BincodeError(_) => {
                if let BincodeError(_) = other {
                    true
                } else {
                    false
                }
            },
            IoError(_) => {
                if let IoError(_) = other {
                    true
                } else {
                    false
                }
            },
            MutexError(e) => {
                if let MutexError(s) = other {
                    e == s
                } else {
                    false
                }
            },
            IndexUndefined(e) => {
                if let IndexUndefined(s) = other {
                    e == s
                } else {
                    false
                }
            },
            InvalidType(e, i) => {
                if let InvalidType(a, b) = other {
                    e == a && i == b
                } else {
                    false
                }
            },
            ValueError(e) => {
                if let ValueError(s) = other {
                    e == s
                } else {
                    false
                }
            },
        }
    }
}
