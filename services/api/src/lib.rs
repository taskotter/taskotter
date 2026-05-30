#![forbid(unsafe_code)]
#![deny(missing_debug_implementations)]

pub mod api;
pub mod audit;
pub mod domain;
pub mod gateway_protocol;
pub mod policy;
pub mod runner_protocol;
pub mod usage;

pub const API_VERSION: &str = "v1";
