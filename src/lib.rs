#![cfg_attr(not(feature = "pyo3"), no_std)]

#[cfg(feature = "pyo3")]
pub mod binding;
pub mod nmea_frame;
pub mod nmea_message;
