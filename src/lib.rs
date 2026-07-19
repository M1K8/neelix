#![cfg_attr(not(feature = "std"), no_std)]

// Always available so firmware-side code can import the shared protocol types
// with `default-features = false`.
pub mod nostd_types;

#[cfg(feature = "std")]
pub mod background;
#[cfg(feature = "std")]
pub mod config;
#[cfg(feature = "std")]
pub mod stats;
#[cfg(feature = "std")]
pub mod types;
#[cfg(feature = "std")]
pub mod ui;
