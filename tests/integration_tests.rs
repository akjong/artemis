//! Tests for the Artemis framework core functionality.

use std::sync::{Arc, Mutex};

use artemis::{
    Engine,
    types::{Collector, CollectorStream, Executor, Strategy},
};
use async_trait::async_trait;
use eyre::Result;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

/// Test event type
#[derive(Clone, Debug, PartialEq)]
pub struct TestEvent {
    pub id: u32,
    pub data: String,
}

/// Test action type
#[derive(Clone, Debug, PartialEq)]
pub struct TestAction {
    pub id: u32,
    pub processed_data: String,
}

/// Test collector that generates a predefined set of events
pub struct TestCollector {
    events: Vec<TestEvent>,
}

impl TestCollector {
    pub fn new(events: Vec<TestEvent>) -> Self {
        Self { events }
    }
}

#[async_trait]
impl Collector<TestEvent> for TestCollector {
    fn name(&self) -> &str {
        "TestCollector"
    }

    async fn get_event_stream(&self) -> Result<CollectorStream<'_, TestEvent>> {
        let (tx, rx) = mpsc::unbounded_channel();
        let events = self.events.clone();

        tokio::spawn(async move {
            for event in events {
                if tx.send(event).is_err() {
                    break;
                }
                // Small delay to simulate real-world timing
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            }
        });

        Ok(Box::pin(UnboundedReceiverStream::new(rx)))
    }
}

/// Test strategy that converts events to actions
pub struct TestStrategy {
    processed_count: Arc<Mutex<u32>>,
}

impl TestStrategy {
    pub fn new() -> Self {
        Self {
            processed_count: Arc::new(Mutex::new(0)),
        }
    }

    pub fn get_processed_count(&self) -> u32 {
        *self.processed_count.lock().unwrap()
    }
}

#[async_trait]
impl Strategy<TestEvent, TestAction> for TestStrategy {
    fn name(&self) -> &str {
        "TestStrategy"
    }

    async fn process_event(&mut self, event: TestEvent) -> Vec<TestAction> {
        let mut count = self.processed_count.lock().unwrap();
        *count += 1;

        vec![TestAction {
            id: event.id,
            processed_data: format!("Processed: {}", event.data),
        }]
    }
}

/// Test executor that collects executed actions
pub struct TestExecutor {
    executed_actions: Arc<Mutex<Vec<TestAction>>>,
}

impl TestExecutor {
    pub fn new() -> Self {
        Self {
            executed_actions: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn get_executed_actions(&self) -> Vec<TestAction> {
        self.executed_actions.lock().unwrap().clone()
    }
}

#[async_trait]
impl Executor<TestAction> for TestExecutor {
    fn name(&self) -> &str {
        "TestExecutor"
    }

    async fn execute(&self, action: TestAction) -> Result<()> {
        let mut actions = self.executed_actions.lock().unwrap();
        actions.push(action);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use tokio::time::Duration;

    use super::*;

    #[tokio::test]
    async fn test_basic_engine_flow() {
        // Create test data
        let test_events = vec![
            TestEvent {
                id: 1,
                data: "Event 1".to_string(),
            },
            TestEvent {
                id: 2,
                data: "Event 2".to_string(),
            },
            TestEvent {
                id: 3,
                data: "Event 3".to_string(),
            },
        ];

        // Create components
        let collector = TestCollector::new(test_events.clone());
        let strategy = TestStrategy::new();
        let executor = TestExecutor::new();

        // Get references to check results later
        let strategy_count_ref = strategy.processed_count.clone();
        let executor_actions_ref = executor.executed_actions.clone();

        // Create and configure engine
        let mut engine = Engine::new();
        engine.add_collector(Box::new(collector));
        engine.add_strategy(Box::new(strategy));
        engine.add_executor(Box::new(executor));

        // Run the engine with a timeout to prevent hanging
        let mut set = engine
            .run()
            .await
            .expect("Engine should start successfully");

        // Wait for a reasonable amount of time for processing to complete
        let mut completed_tasks = 0;
        let start_time = std::time::Instant::now();

        while completed_tasks < 3 && start_time.elapsed().as_secs() < 5 {
            tokio::select! {
                Some(result) = set.join_next() => {
                    completed_tasks += 1;
                    if let Err(err) = result {
                        panic!("Task failed: {}", err);
                    }
                }
                _ = tokio::time::sleep(Duration::from_millis(100)) => {
                    // Check if all events have been processed
                    let processed_count = *strategy_count_ref.lock().unwrap();
                    let executed_count = executor_actions_ref.lock().unwrap().len();

                    if processed_count >= 3 && executed_count >= 3 {
                        break;
                    }
                }
            }
        }

        // Verify that all events were processed
        let processed_count = *strategy_count_ref.lock().unwrap();
        assert!(
            processed_count >= 3,
            "Strategy should process all 3 events, got {}",
            processed_count
        );

        // Verify that all actions were executed
        let executed_actions = executor_actions_ref.lock().unwrap();
        assert!(
            executed_actions.len() >= 3,
            "Executor should execute all 3 actions, got {}",
            executed_actions.len()
        );

        // Verify the content of executed actions (check first 3)
        for (i, action) in executed_actions.iter().take(3).enumerate() {
            let expected_id = (i + 1) as u32;
            assert_eq!(action.id, expected_id, "Action ID should match event ID");
            assert_eq!(
                action.processed_data,
                format!("Processed: Event {}", expected_id),
                "Action data should be processed correctly"
            );
        }

        // Abort remaining tasks
        set.abort_all();
    }

    #[tokio::test]
    async fn test_empty_collector() {
        // Test with no events
        let collector = TestCollector::new(vec![]);
        let strategy = TestStrategy::new();
        let executor = TestExecutor::new();

        let strategy_count_ref = strategy.processed_count.clone();
        let executor_actions_ref = executor.executed_actions.clone();

        let mut engine = Engine::new();
        engine.add_collector(Box::new(collector));
        engine.add_strategy(Box::new(strategy));
        engine.add_executor(Box::new(executor));

        let mut set = engine
            .run()
            .await
            .expect("Engine should start successfully");

        // Wait briefly to ensure no events are processed
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Verify no processing occurred
        let processed_count = *strategy_count_ref.lock().unwrap();
        assert_eq!(processed_count, 0, "Strategy should process no events");

        let executed_actions = executor_actions_ref.lock().unwrap();
        assert_eq!(
            executed_actions.len(),
            0,
            "Executor should execute no actions"
        );

        // Abort remaining tasks
        set.abort_all();
    }

    #[tokio::test]
    async fn test_multiple_collectors() {
        // Create multiple collectors
        let events1 = vec![
            TestEvent {
                id: 1,
                data: "Collector1 Event1".to_string(),
            },
            TestEvent {
                id: 2,
                data: "Collector1 Event2".to_string(),
            },
        ];
        let events2 = vec![TestEvent {
            id: 3,
            data: "Collector2 Event1".to_string(),
        }];

        let collector1 = TestCollector::new(events1);
        let collector2 = TestCollector::new(events2);
        let strategy = TestStrategy::new();
        let executor = TestExecutor::new();

        let strategy_count_ref = strategy.processed_count.clone();
        let executor_actions_ref = executor.executed_actions.clone();

        let mut engine = Engine::new();
        engine.add_collector(Box::new(collector1));
        engine.add_collector(Box::new(collector2));
        engine.add_strategy(Box::new(strategy));
        engine.add_executor(Box::new(executor));

        let mut set = engine
            .run()
            .await
            .expect("Engine should start successfully");

        // Wait for processing to complete
        let start_time = std::time::Instant::now();

        while start_time.elapsed().as_secs() < 3 {
            let processed_count = *strategy_count_ref.lock().unwrap();
            let executed_count = executor_actions_ref.lock().unwrap().len();

            if processed_count >= 3 && executed_count >= 3 {
                break;
            }

            tokio::time::sleep(Duration::from_millis(50)).await;
        }

        // Verify total processing
        let processed_count = *strategy_count_ref.lock().unwrap();
        assert!(
            processed_count >= 3,
            "Strategy should process all 3 events from both collectors, got {}",
            processed_count
        );

        let executed_actions = executor_actions_ref.lock().unwrap();
        assert!(
            executed_actions.len() >= 3,
            "Executor should execute all 3 actions, got {}",
            executed_actions.len()
        );

        // Abort remaining tasks
        set.abort_all();
    }

    #[tokio::test]
    async fn test_engine_with_no_components() {
        // Test engine with no components added
        let engine: Engine<TestEvent, TestAction> = Engine::new();

        let mut set = engine
            .run()
            .await
            .expect("Engine should start successfully even with no components");

        // Should return immediately with no tasks
        assert_eq!(
            set.len(),
            0,
            "Should have no tasks when no components are added"
        );

        // Try to join next (should return None immediately)
        let result = set.join_next().await;
        assert!(result.is_none(), "Should have no tasks to join");
    }
}
