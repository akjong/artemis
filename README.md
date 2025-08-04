# Artemis

[![Crates.io](https://img.shields.io/crates/v/artemis.svg)](https://crates.io/crates/artemis)
[![Documentation](https://docs.rs/artemis/badge.svg)](https://docs.rs/artemis)
[![License: Apache-2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

Artemis is a high-performance, event-driven framework for building applications that process streams of events through collectors, strategies, and executors. It provides a clean architecture for building reactive systems that can handle complex event processing workflows.

## Features

- **Event-driven architecture**: Built around the concept of event streams and reactive processing
- **Modular design**: Separate concerns with collectors, strategies, and executors
- **High performance**: Asynchronous processing with efficient resource utilization
- **Type safety**: Leverages Rust's type system for compile-time guarantees
- **Flexible mapping**: Built-in support for mapping between different event and action types
- **Convenient macros**: Simplify common patterns with provided macros

## Architecture

The framework consists of three main components:

### Collectors

Sources of events that implement the `Collector` trait. Collectors are responsible for gathering events from various sources (e.g., blockchain nodes, APIs, file systems) and feeding them into the processing pipeline.

### Strategies

Processing logic that transforms events into actions via the `Strategy` trait. Strategies contain your business logic and determine what actions should be taken in response to specific events.

### Executors

Action handlers that implement the `Executor` trait. Executors are responsible for carrying out the actions determined by strategies (e.g., sending transactions, making API calls, writing to databases).

## Quick Start

Add Artemis to your `Cargo.toml`:

```toml
[dependencies]
artemis = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
eyre = "0.6"
```

### Basic Example

```rust
use artemis::{Engine, types::{Collector, Strategy, Executor}};
use async_trait::async_trait;
use eyre::Result;

// Define your event and action types
#[derive(Clone, Debug)]
pub struct MyEvent {
    pub data: String,
}

#[derive(Clone, Debug)]
pub struct MyAction {
    pub response: String,
}

// Implement a collector
pub struct MyCollector;

#[async_trait]
impl Collector<MyEvent> for MyCollector {
    async fn get_event_stream(&self) -> Result<tokio::sync::mpsc::UnboundedReceiver<MyEvent>> {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        
        // Simulate event generation
        tokio::spawn(async move {
            for i in 0..10 {
                let event = MyEvent {
                    data: format!("Event {}", i),
                };
                if tx.send(event).is_err() {
                    break;
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        });
        
        Ok(rx)
    }
}

// Implement a strategy
pub struct MyStrategy;

#[async_trait]
impl Strategy<MyEvent, MyAction> for MyStrategy {
    async fn process_event(&self, event: MyEvent) -> Result<Option<MyAction>> {
        Ok(Some(MyAction {
            response: format!("Processed: {}", event.data),
        }))
    }
}

// Implement an executor
pub struct MyExecutor;

#[async_trait]
impl Executor<MyAction> for MyExecutor {
    async fn execute(&self, action: MyAction) -> Result<()> {
        println!("Executing: {}", action.response);
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Create an engine
    let mut engine = Engine::new();

    // Add components
    engine.add_collector(Box::new(MyCollector));
    engine.add_strategy(Box::new(MyStrategy));
    engine.add_executor(Box::new(MyExecutor));

    // Run the engine
    engine.run_and_join().await?;
    
    Ok(())
}
```

### Working with Enums and Macros

Artemis provides convenient macros when working with multiple event or action types using enums:

```rust
use artemis::{map_collector, map_executor};

#[derive(Clone, Debug)]
pub enum Event {
    TypeA(EventA),
    TypeB(EventB),
}

#[derive(Clone, Debug)]
pub enum Action {
    TypeX(ActionX),
    TypeY(ActionY),
}

// Map specific collectors and executors to enum variants
let collector_a = map_collector!(MyCollectorA::new(), Event::TypeA);
let executor_x = map_executor!(MyExecutorX::new(), Action::TypeX);

engine.add_collector(collector_a);
engine.add_executor(executor_x);
```

#### Available Macros

- **`map_collector!(collector, EnumVariant)`** - Wraps collector output in enum variant
- **`map_boxed_collector!(boxed_collector, EnumVariant)`** - Same as above but for already boxed collectors
- **`map_executor!(executor, EnumVariant)`** - Filters enum actions and passes specific type to executor  
- **`map_boxed_executor!(boxed_executor, EnumVariant)`** - Same as above but for already boxed executors

#### Macro Benefits

- ✅ **Reduced boilerplate** - No manual mapping code needed
- ✅ **Type safety** - Compile-time verification of enum variants
- ✅ **Clear intent** - Code clearly shows the mapping relationships
- ✅ **Automatic boxing** - Handles trait object creation automatically

## Advanced Usage

### Custom Error Handling

```rust
use artemis::Engine;
use eyre::{Result, WrapErr};

#[tokio::main]
async fn main() -> Result<()> {
    let mut engine = Engine::new();
    
    // Add your components...
    
    engine.run_and_join().await
        .wrap_err("Failed to run Artemis engine")?;
        
    Ok(())
}
```

### Graceful Shutdown

The engine supports graceful shutdown through Rust's standard `Drop` trait and async cancellation:

```rust
use tokio::signal;

#[tokio::main]
async fn main() -> Result<()> {
    let mut engine = Engine::new();
    
    // Add your components...
    
    // Set up graceful shutdown
    tokio::select! {
        result = engine.run_and_join() => {
            result?;
        }
        _ = signal::ctrl_c() => {
            println!("Received Ctrl+C, shutting down gracefully...");
        }
    }
    
    Ok(())
}
```

## Examples

The repository includes several examples demonstrating different aspects of the framework:

### Running Examples

```bash
cargo run --example basic
```

Demonstrates fundamental usage with macro-based type mapping between specific types and enums.

### Advanced Usage Example

```bash
cargo run --example advanced
```

Shows a more complex scenario with multiple collectors, strategies, and executors working with blockchain-like events.

### Macro Demonstration

```bash
cargo run --example macro_demo
```

Focused demonstration of all available macros and their benefits for type mapping.

## Testing

Run the test suite:

```bash
cargo test
```

Run with documentation tests:

```bash
cargo test --doc
```

Generate and view documentation:

```bash
cargo doc --open
```

## API Reference

### Core Traits

- [`Collector<T>`](https://docs.rs/artemis/latest/artemis/types/trait.Collector.html) - Sources of events
- [`Strategy<E, A>`](https://docs.rs/artemis/latest/artemis/types/trait.Strategy.html) - Event processing logic  
- [`Executor<T>`](https://docs.rs/artemis/latest/artemis/types/trait.Executor.html) - Action handlers

### Mapping Types

- [`CollectorMap<T, U>`](https://docs.rs/artemis/latest/artemis/types/struct.CollectorMap.html) - Maps collector output to different types
- [`ExecutorMap<T, U>`](https://docs.rs/artemis/latest/artemis/types/struct.ExecutorMap.html) - Maps executor input from different types

### Macros

- [`map_collector!`](https://docs.rs/artemis/latest/artemis/macro.map_collector.html) - Create mapped collectors
- [`map_executor!`](https://docs.rs/artemis/latest/artemis/macro.map_executor.html) - Create mapped executors
- [`map_boxed_collector!`](https://docs.rs/artemis/latest/artemis/macro.map_boxed_collector.html) - Create boxed mapped collectors
- [`map_boxed_executor!`](https://docs.rs/artemis/latest/artemis/macro.map_boxed_executor.html) - Create boxed mapped executors

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

## License

This project is licensed under the Apache-2.0 License - see the [LICENSE](LICENSE) file for details.
