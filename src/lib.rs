#[macro_use]
extern crate log;

pub mod device;
pub mod model;
pub mod error;

type Result<T> = std::result::Result<T, error::Error>;