# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Initial release of Artemis framework
- Core traits: `Collector`, `Strategy`, and `Executor`
- `Engine` for coordinating event processing pipeline
- `CollectorMap` and `ExecutorMap` for type transformations
- Convenience macros: `map_collector!`, `map_executor!`, `map_boxed_collector!`, `map_boxed_executor!`
- Comprehensive documentation and examples
- Basic and advanced usage examples
- Integration tests for core functionality

### Features

- Event-driven architecture with async/await support
- Type-safe event and action processing
- Flexible mapping between different event/action types
- Graceful shutdown handling
- Error propagation with `eyre` integration
- Stream-based event processing with `tokio-stream`

## [0.1.0] - 2025-07-03

### Changed

- Initial project structure
- Core framework implementation
- Documentation and examples
- Basic test coverage

[Unreleased]: https://github.com/akjong/artemis/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/akjong/artemis/releases/tag/v0.1.0
