#![cfg_attr(not(feature = "pyo3"), no_std)]

pub mod nmea_frame;
pub mod nmea_message;
#[cfg(feature = "pyo3")]
pub mod binding;
