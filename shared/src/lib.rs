//! Shared types and utilities for Rust Teams application
//! 
//! This crate contains all the shared data structures, DTOs, and utilities
//! used by both the backend and frontend.

pub mod models;
pub mod dto;
pub mod error;
pub mod validation;

pub use models::*;
pub use dto::*;
pub use error::*;
