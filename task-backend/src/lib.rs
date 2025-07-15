// src/lib.rs
pub mod api;
pub mod config;
pub mod db;
pub mod domain;
pub mod error;
pub mod middleware;
pub mod repository;
pub mod service;
pub mod shared;
pub mod types;
pub mod utils;

// Re-export commonly used types
pub use types::ApiResponse;
