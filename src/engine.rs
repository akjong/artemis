//! The core engine module that orchestrates the data flow between collectors, strategies, and executors.

use tokio::{
    sync::broadcast::{self, Sender},
    task::JoinSet,
};
use tokio_stream::StreamExt;
use tracing::{error, info, warn};

use crate::types::{Collector, Executor, Module, Strategy};

/// The main engine that orchestrates the data flow between collectors, strategies, and executors.
///
/// The engine creates separate async tasks for each collector, strategy, and executor,
/// connecting them through broadcast channels. Events flow from collectors to strategies,
/// which then generate actions that are executed by executors.
///
/// # Type Parameters
///
/// * `E` - The event type that flows from collectors to strategies
/// * `A` - The action type that flows from strategies to executors
///
/// # Example
///
/// ```rust,ignore
/// use artemis::Engine;
///
/// let mut engine: Engine<MyEvent, MyAction> = Engine::new()
///     .with_event_channel_capacity(2048)
///     .with_action_channel_capacity(1024);
///
/// engine.add_collector(Box::new(my_collector));
/// engine.add_strategy(Box::new(my_strategy));
/// engine.add_executor(Box::new(my_executor));
///
/// engine.run_and_join().await?;
/// ```
/// The main engine. This struct is responsible for orchestrating the
/// data flow between collectors, strategies, and executors.
pub struct Engine<E, A> {
    /// The set of collectors that the engine will use to collect events.
    collectors: Vec<Box<dyn Collector<E>>>,

    /// The set of strategies that the engine will use to process events.
    strategies: Vec<Box<dyn Strategy<E, A>>>,

    /// The set of executors that the engine will use to execute actions.
    executors: Vec<Box<dyn Executor<A>>>,

    /// The capacity of the event channel.
    event_channel_capacity: usize,

    /// The capacity of the action channel.
    action_channel_capacity: usize,
}

impl<E, A> Engine<E, A> {
    /// Creates a new engine with default channel capacities.
    ///
    /// Default capacities:
    /// - Event channel: 1024
    /// - Action channel: 1024
    pub fn new() -> Self {
        Self {
            collectors: vec![],
            strategies: vec![],
            executors: vec![],
            event_channel_capacity: 1024,
            action_channel_capacity: 1024,
        }
    }

    /// Sets the capacity of the event channel that connects collectors to strategies.
    ///
    /// # Arguments
    ///
    /// * `capacity` - The number of events that can be buffered in the channel
    pub fn with_event_channel_capacity(mut self, capacity: usize) -> Self {
        self.event_channel_capacity = capacity;
        self
    }

    /// Sets the capacity of the action channel that connects strategies to executors.
    ///
    /// # Arguments
    ///
    /// * `capacity` - The number of actions that can be buffered in the channel
    pub fn with_action_channel_capacity(mut self, capacity: usize) -> Self {
        self.action_channel_capacity = capacity;
        self
    }
}

impl<E, A> Default for Engine<E, A> {
    fn default() -> Self {
        Self::new()
    }
}

impl<E, A> Engine<E, A>
where
    E: Send + Clone + 'static + std::fmt::Debug,
    A: Send + Clone + 'static + std::fmt::Debug,
{
    /// Adds a collector to be used by the engine.
    ///
    /// # Arguments
    ///
    /// * `collector` - A boxed collector that implements the `Collector` trait
    pub fn add_collector(&mut self, collector: Box<dyn Collector<E>>) {
        self.collectors.push(collector);
    }

    /// Adds a strategy to be used by the engine.
    ///
    /// # Arguments
    ///
    /// * `strategy` - A boxed strategy that implements the `Strategy` trait
    pub fn add_strategy(&mut self, strategy: Box<dyn Strategy<E, A>>) {
        self.strategies.push(strategy);
    }

    /// Adds an executor to be used by the engine.
    ///
    /// # Arguments
    ///
    /// * `executor` - A boxed executor that implements the `Executor` trait
    pub fn add_executor(&mut self, executor: Box<dyn Executor<A>>) {
        self.executors.push(executor);
    }

    /// Starts the engine and returns a JoinSet containing all spawned tasks.
    ///
    /// This method spawns separate async tasks for each collector, strategy, and executor,
    /// connecting them through broadcast channels. The returned JoinSet can be used to
    /// wait for all tasks or handle them individually.
    ///
    /// # Returns
    ///
    /// A `Result` containing either:
    /// - `Ok(JoinSet<()>)` - A JoinSet with all spawned tasks
    /// - `Err(Box<dyn std::error::Error>)` - An error if validation fails
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No collectors are configured
    /// - No strategies are configured  
    /// - No executors are configured
    /// - A strategy fails to sync its initial state
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let mut set = engine.run().await?;
    ///
    /// // Handle tasks individually
    /// while let Some(result) = set.join_next().await {
    ///     if let Err(err) = result {
    ///         eprintln!("Task failed: {}", err);
    ///     }
    /// }
    /// ```
    pub async fn run(self) -> Result<JoinSet<()>, Box<dyn std::error::Error>> {
        let (event_sender, _): (Sender<E>, _) = broadcast::channel(self.event_channel_capacity);
        let (action_sender, _): (Sender<A>, _) = broadcast::channel(self.action_channel_capacity);

        let mut set = JoinSet::new();

        // Allow running with no components for testing purposes
        if self.collectors.is_empty() && self.strategies.is_empty() && self.executors.is_empty() {
            info!(target: Module::ENGINE, "no components configured, engine will exit immediately");
            return Ok(set);
        }

        if !self.executors.is_empty() && self.strategies.is_empty() {
            return Err("executors configured but no strategies provided".into());
        }

        if !self.strategies.is_empty() && self.collectors.is_empty() {
            return Err("strategies configured but no collectors provided".into());
        }

        if !self.collectors.is_empty() && self.strategies.is_empty() {
            return Err("collectors configured but no strategies provided".into());
        }

        // Spawn executors in separate threads.
        for executor in self.executors {
            let mut receiver = action_sender.subscribe();
            set.spawn(async move {
                info!(target: Module::ENGINE, "starting executor: {}", executor.name());
                loop {
                    match receiver.recv().await {
                        Ok(action) => match executor.execute(action).await {
                            Ok(_) => {}
                            Err(e) => {
                                error!(target: Module::ENGINE, "error executing action: {}", e)
                            }
                        },
                        Err(broadcast::error::RecvError::Closed) => {
                            info!(target: Module::ENGINE, "action channel closed, shutting down executor: {}", executor.name());
                            break;
                        }
                        Err(broadcast::error::RecvError::Lagged(skipped)) => {
                            // Drop backlog to keep up; don't break the loop.
                            warn!(target: Module::ENGINE, "executor {} lagged by {} actions; skipping to latest", executor.name(), skipped);
                            continue;
                        }
                    }
                }
            });
        }

    // Spawn strategies in separate threads.
        for mut strategy in self.strategies {
            let mut event_receiver = event_sender.subscribe();
            let action_sender = action_sender.clone();
            strategy.sync_state().await?;

            set.spawn(async move {
                info!(target: Module::ENGINE, "starting strategy: {}", strategy.name());
                loop {
                    match event_receiver.recv().await {
                        Ok(event) => {
                            for action in strategy.process_event(event).await {
                                if let Err(e) = action_sender.send(action) {
                                    error!(target: Module::ENGINE, "error sending action: {}", e);
                                }
                            }
                        }
                        Err(broadcast::error::RecvError::Closed) => {
                            info!(target: Module::ENGINE, "event channel closed, shutting down strategy: {}", strategy.name());
                            break;
                        }
                        Err(broadcast::error::RecvError::Lagged(skipped)) => {
                            // Skip directly to latest without sleeping to reduce head-of-line blocking.
                            warn!(target: Module::ENGINE, "strategy {} lagged by {} events; skipping to latest", strategy.name(), skipped);
                            continue;
                        }
                    }
                }
            });
        }

        // Spawn collectors in separate threads.
        for collector in self.collectors {
            let event_sender = event_sender.clone();
            set.spawn(async move {
                info!(target: Module::ENGINE, "starting collector: {}", collector.name());
                match collector.get_event_stream().await {
                    Ok(mut event_stream) => {
                        while let Some(event) = event_stream.next().await {
                            if let Err(e) = event_sender.send(event) {
                                error!(target: Module::ENGINE, "error sending event: {}", e);
                            }
                        }
                        info!(target: Module::ENGINE, "collector {} finished", collector.name());
                    }
                    Err(e) => {
                        error!(target: Module::ENGINE, "error getting event stream for collector {}: {}", collector.name(), e);
                    }
                }
            });
        }

        // Drop the original senders to signal shutdown once all clones (held by tasks) are dropped
        drop(event_sender);
        drop(action_sender);

        Ok(set)
    }
    /// Starts the engine and waits for all tasks to complete.
    ///
    /// This is a convenience method that calls `run()` and then waits for all
    /// spawned tasks to complete. If any task fails, an error is logged but
    /// the method continues to wait for other tasks.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure of the operation.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The engine fails to start (same conditions as `run()`)
    /// - Any critical system error occurs
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // This will run until all tasks complete or fail
    /// engine.run_and_join().await?;
    /// ```
    pub async fn run_and_join(self) -> Result<(), Box<dyn std::error::Error>> {
        let mut set = self.run().await?;

        while let Some(event) = set.join_next().await {
            if let Err(err) = event {
                error!(target: Module::ENGINE, "engine terminated unexpectedly: {err:#}");
            }
        }

        Ok(())
    }
}
