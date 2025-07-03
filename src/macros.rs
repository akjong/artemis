//! # Artemis Macros
//!
//! This module provides convenient macros for working with collectors and executors
//! in the Artemis framework. These macros simplify the process of creating mapped
//! collectors and executors that work with specific action variants.

/// Creates a boxed executor that maps a specific action variant to a concrete action type.
///
/// This macro is useful when you have an executor that works with a specific action type,
/// but you need to integrate it into a system that uses a general action enum.
///
/// # Arguments
///
/// * `$executor` - The executor instance to wrap
/// * `$variant` - The enum variant path that this executor should handle
///
/// # Returns
///
/// A boxed `ExecutorMap` that filters actions and only processes the specified variant.
///
/// # Examples
///
/// ```rust,ignore
/// use artemis::map_boxed_executor;
///
/// enum Action {
///     Transfer(TransferAction),
///     Swap(SwapAction),
/// }
///
/// let transfer_executor = MyTransferExecutor::new();
/// let mapped_executor = map_boxed_executor!(transfer_executor, Action::Transfer);
/// ```
#[macro_export]
macro_rules! map_boxed_executor {
    ($executor: expr, $variant: path) => {
        Box::new($crate::ExecutorMap::new($executor, |action| match action {
            $variant(value) => Some(value),
            _ => None,
        }))
    };
}

/// Creates a boxed executor that maps a specific action variant to a concrete action type.
///
/// This is a convenience wrapper around [`map_boxed_executor!`] that automatically
/// boxes the provided executor.
///
/// # Arguments
///
/// * `$executor` - The executor instance to wrap (will be automatically boxed)
/// * `$variant` - The enum variant path that this executor should handle
///
/// # Returns
///
/// A boxed `ExecutorMap` that filters actions and only processes the specified variant.
///
/// # Examples
///
/// ```rust,ignore
/// use artemis::map_executor;
///
/// enum Action {
///     Transfer(TransferAction),
///     Swap(SwapAction),
/// }
///
/// let transfer_executor = MyTransferExecutor::new();
/// let mapped_executor = map_executor!(transfer_executor, Action::Transfer);
/// ```
#[macro_export]
macro_rules! map_executor {
    ($executor: expr, $variant: path) => {
        $crate::map_boxed_executor!(Box::new($executor), $variant)
    };
}

/// Creates a boxed collector that wraps events in a specific enum variant.
///
/// This macro is useful when you have a collector that produces a specific event type,
/// but you need to integrate it into a system that uses a general event enum.
///
/// # Arguments
///
/// * `$collector` - The collector instance to wrap
/// * `$variant` - The enum variant constructor to wrap events with
///
/// # Returns
///
/// A boxed `CollectorMap` that wraps produced events in the specified variant.
///
/// # Examples
///
/// ```rust,ignore
/// use artemis::map_boxed_collector;
///
/// enum Event {
///     Block(BlockEvent),
///     Transaction(TransactionEvent),
/// }
///
/// let block_collector = MyBlockCollector::new();
/// let mapped_collector = map_boxed_collector!(block_collector, Event::Block);
/// ```
#[macro_export]
macro_rules! map_boxed_collector {
    ($collector: expr, $variant: path) => {
        Box::new($crate::CollectorMap::new($collector, $variant))
    };
}

/// Creates a boxed collector that wraps events in a specific enum variant.
///
/// This is a convenience wrapper around [`map_boxed_collector!`] that automatically
/// boxes the provided collector.
///
/// # Arguments
///
/// * `$collector` - The collector instance to wrap (will be automatically boxed)
/// * `$variant` - The enum variant constructor to wrap events with
///
/// # Returns
///
/// A boxed `CollectorMap` that wraps produced events in the specified variant.
///
/// # Examples
///
/// ```rust,ignore
/// use artemis::map_collector;
///
/// enum Event {
///     Block(BlockEvent),
///     Transaction(TransactionEvent),
/// }
///
/// let block_collector = MyBlockCollector::new();
/// let mapped_collector = map_collector!(block_collector, Event::Block);
/// ```
#[macro_export]
macro_rules! map_collector {
    ($collector: expr, $variant: path) => {
        $crate::map_boxed_collector!(Box::new($collector), $variant)
    };
}
