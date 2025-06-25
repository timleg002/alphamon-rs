use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid format or length of response data")]
    InvalidFormat,

    #[error("Invalid battery capacity parameter!")]
    InvalidBatteryCapacityParameter,

    #[error("Invalid float encountered")]
    FloatParse(#[from] std::num::ParseFloatError),

    #[error("Invalid int encountered")]
    IntParse(#[from] std::num::ParseIntError),

    #[error("Invalid length of message parameter")]
    InvalidParameterLength(#[from] std::array::TryFromSliceError),

    #[error("The buffer is too small (expected: {expected}, provided {provided})")]
    BufferTooSmall { expected: usize, provided: usize },

    #[error("An error occured during an I/O operation")]
    Io(#[from] std::io::Error),

    #[cfg(feature = "serial")]
    #[error("An error occured with serial port: {}", .0.description)]
    SerialPort(#[from] serialport::Error),

    #[cfg(feature = "usb-hidapi")]
    #[error("An error occured with the HID: {}", .0)]
    HidApi(#[from] hidapi::HidError),
}