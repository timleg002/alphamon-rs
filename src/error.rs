use std::{array::TryFromSliceError, num::{ParseFloatError, ParseIntError}};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid format or length of response data")]
    InvalidFormat,
    #[error("Invalid battery capacity parameter!")]
    InvalidBatteryCapacityParameter,
    #[error("Invalid float encountered")]
    FloatParse(#[from] ParseFloatError),
    #[error("Invalid int encountered")]
    IntParse(#[from] ParseIntError),
    #[error("Invalid length of message parameter")]
    InvalidParameterLength(#[from] TryFromSliceError),
    #[error("An error occured with serial port: {}", .0.description)]
    SerialPort(#[from] serialport::Error),
    #[error("An error occured during an I/O operation")]
    Io(#[from] std::io::Error)
}