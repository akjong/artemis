# Artemis Framework - AI Coding Instructions

## Architecture Overview

Artemis is an event-driven framework built around three core components that form an async pipeline:

1. **Collectors** (`types::Collector<E>`) - Event sources that emit `CollectorStream<E>` 
2. **Strategies** (`types::Strategy<E,A>`) - Event processors that convert events to actions
3. **Executors** (`types::Executor<A>`) - Action handlers that perform final operations

The `Engine<E,A>` orchestrates these components using tokio broadcast channels, spawning separate tasks for each component type.

## Key Patterns & Conventions

### Engine Flow Pattern
```rust
// Standard engine setup pattern used throughout examples
let mut engine = Engine::new();
engine.add_collector(Box::new(collector));
engine.add_strategy(Box::new(strategy));  
engine.add_executor(Box::new(executor));
engine.run_and_join().await?;
```

### Async Trait Implementation
All core traits use `#[async_trait]` - always include this attribute when implementing:
- `Collector::get_event_stream()` returns `Result<CollectorStream<'_, E>>`
- `Strategy::process_event()` returns `Vec<A>` (multiple actions allowed)
- `Executor::execute()` returns `Result<()>`

### Event Stream Creation Pattern
Collectors use this consistent pattern for creating event streams:
```rust
let (tx, rx) = mpsc::unbounded_channel();
tokio::spawn(async move { /* emit events via tx */ });
Ok(Box::pin(UnboundedReceiverStream::new(rx)))
```

### Type Mapping with Macros
The framework's key abstraction: use provided macros for enum type mapping instead of manual `CollectorMap`/`ExecutorMap`:

- `map_collector!(collector, EnumVariant)` - wraps collector output in enum variant
- `map_executor!(executor, EnumVariant)` - filters enum actions to specific executor
- `map_boxed_*` variants for pre-boxed components

## Development Workflows

### Build & Test Commands (using Just)
```bash
just format  # Format code with taplo + cargo fmt
just lint    # Check formatting + clippy + machete  
just test    # Run all tests
```

### Example Running
```bash
cargo run --example basic      # Shows macro usage patterns
cargo run --example advanced   # Multi-type enum handling
cargo run --example macro_demo # All macro variants
```

### Testing Patterns
Integration tests in `tests/` use shared test components (`TestCollector`, `TestStrategy`, `TestExecutor`) with Arc<Mutex<>> for state verification. Tests wait for processing completion with timeouts and `set.abort_all()` cleanup.

## Critical Implementation Details

### Engine Validation Rules
Engine startup enforces component dependencies:
- Executors require strategies
- Strategies require collectors  
- Collectors require strategies
- Empty engine (no components) is allowed

### Channel Configuration
Engine uses configurable broadcast channels:
- `with_event_channel_capacity(n)` - collector→strategy buffer
- `with_action_channel_capacity(n)` - strategy→executor buffer
- Defaults: 1024 each

### Error Handling Convention
- Collectors/Executors: log errors, continue processing other items
- Strategies: return empty `Vec<A>` for unhandled events
- Engine: fails fast on validation errors, graceful shutdown on channel closure

### Strategy State Sync
Strategies have `sync_state()` called once at engine startup - use for initialization like fetching external state. Default implementation does nothing.

## Project Structure
- `src/engine.rs` - Core orchestration logic
- `src/types.rs` - Trait definitions and mapping utilities  
- `src/macros.rs` - Type mapping convenience macros
- `examples/` - Demonstrate patterns: basic→advanced→macro_demo
- `tests/integration_tests.rs` - End-to-end testing patterns

## Dependencies Context
- `async-trait` - Required for all trait implementations
- `tokio-stream` - For `CollectorStream` and `UnboundedReceiverStream`
- `eyre` - Error handling (prefer over `anyhow`)
- `tracing` - Structured logging with target modules
- Edition 2024 + nightly fmt/clippy in CI
