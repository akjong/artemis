//! # Artemis
//!
//! Artemis is a high-performance, event-driven framework for building applications
//! that process streams of events through collectors, strategies, and executors.
//!
//! ## Architecture
//!
//! The framework consists of three main components:
//!
//! - **Collectors**: Sources of events that implement the [`Collector`] trait
//! - **Strategies**: Processing logic that transforms events into actions via the [`Strategy`] trait
//! - **Executors**: Action handlers that implement the [`Executor`] trait
//!
//! ## Basic Usage
//!
//! ```rust,ignore
//! use artemis::{Engine, types::{Collector, Strategy, Executor}};
//!
//! // Create an engine
//! let mut engine = Engine::new();
//!
//! // Add your collectors, strategies, and executors
//! engine.add_collector(Box::new(my_collector));
//! engine.add_strategy(Box::new(my_strategy));
//! engine.add_executor(Box::new(my_executor));
//!
//! // Run the engine
//! engine.run_and_join().await?;
//! ```
//!
//! [`Collector`]: types::Collector
//! [`Strategy`]: types::Strategy  
//! [`Executor`]: types::Executor

pub mod engine;
pub mod macros;
pub mod types;

// Re-export main types for convenience
pub use engine::Engine;
pub use types::{Collector, CollectorMap, Executor, ExecutorMap, Strategy};
