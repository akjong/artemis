use std::sync::{Arc, Mutex};

use artemis::{Engine, types::{Collector, CollectorStream, Executor, Strategy}};
use async_trait::async_trait;
use eyre::{Result, eyre};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

#[derive(Clone, Debug)]
struct Evt(u32);
#[derive(Clone, Debug)]
struct Act(u32);

struct OkCollector;
#[async_trait]
impl Collector<Evt> for OkCollector {
    fn name(&self) -> &str { "OkCollector" }
    async fn get_event_stream(&self) -> Result<CollectorStream<'_, Evt>> {
        let (tx, rx) = mpsc::unbounded_channel();
        tokio::spawn(async move {
            let _ = tx.send(Evt(1));
        });
        Ok(Box::pin(UnboundedReceiverStream::new(rx)))
    }
}

struct FailingCollector;
#[async_trait]
impl Collector<Evt> for FailingCollector {
    fn name(&self) -> &str { "FailingCollector" }
    async fn get_event_stream(&self) -> Result<CollectorStream<'_, Evt>> {
        Err(eyre!("boom"))
    }
}

struct SimpleStrategy(Arc<Mutex<u32>>);
#[async_trait]
impl Strategy<Evt, Act> for SimpleStrategy {
    fn name(&self) -> &str { "SimpleStrategy" }
    async fn sync_state(&mut self) -> Result<()> { Ok(()) }
    async fn process_event(&mut self, e: Evt) -> Vec<Act> {
        *self.0.lock().unwrap() += 1;
        vec![Act(e.0)]
    }
}

struct SyncFailStrategy;
#[async_trait]
impl Strategy<Evt, Act> for SyncFailStrategy {
    fn name(&self) -> &str { "SyncFailStrategy" }
    async fn sync_state(&mut self) -> Result<()> { Err(eyre!("sync failed")) }
    async fn process_event(&mut self, _e: Evt) -> Vec<Act> { vec![] }
}

struct SimpleExecutor(Arc<Mutex<Vec<Act>>>);
#[async_trait]
impl Executor<Act> for SimpleExecutor {
    fn name(&self) -> &str { "SimpleExecutor" }
    async fn execute(&self, a: Act) -> Result<()> {
    // Read the field to avoid dead_code warning and to simulate minimal use
    let _id = a.0;
        self.0.lock().unwrap().push(a);
        Ok(())
    }
}

#[tokio::test]
async fn executors_without_strategies_should_error() {
    let mut engine: Engine<Evt, Act> = Engine::new();
    engine.add_executor(Box::new(SimpleExecutor(Arc::new(Mutex::new(vec![])))));
    let res = engine.run().await;
    assert!(res.is_err(), "expected error when executors exist but no strategies");
}

#[tokio::test]
async fn strategies_without_collectors_should_error() {
    let mut engine: Engine<Evt, Act> = Engine::new();
    engine.add_strategy(Box::new(SimpleStrategy(Arc::new(Mutex::new(0)))));
    let res = engine.run().await;
    assert!(res.is_err(), "expected error when strategies exist but no collectors");
}

#[tokio::test]
async fn collectors_without_strategies_should_error() {
    let mut engine: Engine<Evt, Act> = Engine::new();
    engine.add_collector(Box::new(OkCollector));
    let res = engine.run().await;
    assert!(res.is_err(), "expected error when collectors exist but no strategies");
}

#[tokio::test]
async fn strategy_sync_state_error_propagates() {
    let mut engine: Engine<Evt, Act> = Engine::new();
    engine.add_collector(Box::new(OkCollector));
    engine.add_strategy(Box::new(SyncFailStrategy));
    engine.add_executor(Box::new(SimpleExecutor(Arc::new(Mutex::new(vec![])))));
    let res = engine.run().await;
    assert!(res.is_err(), "expected error when strategy sync_state fails");
}

#[tokio::test]
async fn failing_collector_does_not_panic_and_engine_finishes() {
    let mut engine: Engine<Evt, Act> = Engine::new();
    engine.add_collector(Box::new(FailingCollector));
    engine.add_strategy(Box::new(SimpleStrategy(Arc::new(Mutex::new(0)))));
    engine.add_executor(Box::new(SimpleExecutor(Arc::new(Mutex::new(vec![])))));

    // Should complete quickly since collector fails immediately and channels close
    let res = engine.run_and_join().await;
    assert!(res.is_ok(), "engine should handle collector failure gracefully");
}
