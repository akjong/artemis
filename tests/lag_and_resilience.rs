use std::sync::{Arc, Mutex};

use artemis::{
    Engine,
    types::{Collector, CollectorStream, Executor, Strategy},
};
use async_trait::async_trait;
use eyre::Result;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

#[derive(Clone, Debug)]
struct Evt(u32);
#[derive(Clone, Debug)]
struct Act(u32);

// Fast collector emits many events quickly to trigger broadcast lag.
struct FastCollector(u32);
#[async_trait]
impl Collector<Evt> for FastCollector {
    fn name(&self) -> &str {
        "FastCollector"
    }
    async fn get_event_stream(&self) -> Result<CollectorStream<'_, Evt>> {
        let (tx, rx) = mpsc::unbounded_channel();
        let n = self.0;
        tokio::spawn(async move {
            for i in 0..n {
                let _ = tx.send(Evt(i));
            }
        });
        Ok(Box::pin(UnboundedReceiverStream::new(rx)))
    }
}

struct SlowStrategy(Arc<Mutex<u64>>);
#[async_trait]
impl Strategy<Evt, Act> for SlowStrategy {
    fn name(&self) -> &str {
        "SlowStrategy"
    }
    async fn process_event(&mut self, e: Evt) -> Vec<Act> {
        // simulate slow processing to provoke lag
        tokio::time::sleep(std::time::Duration::from_millis(2)).await;
        *self.0.lock().unwrap() += 1;
        vec![Act(e.0)]
    }
}

struct CollectExec(Arc<Mutex<Vec<u32>>>);
#[async_trait]
impl Executor<Act> for CollectExec {
    fn name(&self) -> &str {
        "CollectExec"
    }
    async fn execute(&self, a: Act) -> Result<()> {
        self.0.lock().unwrap().push(a.0);
        Ok(())
    }
}

#[tokio::test]
async fn lag_should_not_crash_and_should_progress() {
    // Small channel capacities to force lag situations
    let mut engine: Engine<Evt, Act> = Engine::new()
        .with_event_channel_capacity(8)
        .with_action_channel_capacity(8);

    let total = 500u32;
    engine.add_collector(Box::new(FastCollector(total)));
    let processed = Arc::new(Mutex::new(0u64));
    engine.add_strategy(Box::new(SlowStrategy(processed.clone())));

    let exec_actions = Arc::new(Mutex::new(Vec::new()));
    engine.add_executor(Box::new(CollectExec(exec_actions.clone())));

    // Run and wait for completion
    let mut set = engine.run().await.expect("engine should start");

    // Wait a bounded time; we expect progress (some events handled), not necessarily all due to lag skipping
    let start = std::time::Instant::now();
    while start.elapsed().as_millis() < 500 {
        if *processed.lock().unwrap() > 0 {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }

    assert!(
        *processed.lock().unwrap() > 0,
        "strategy should have processed at least one event despite lag"
    );

    set.abort_all();
}

#[tokio::test]
async fn run_and_join_finishes_when_collectors_end() {
    let mut engine: Engine<Evt, Act> = Engine::new();
    engine.add_collector(Box::new(FastCollector(10)));
    engine.add_strategy(Box::new(SlowStrategy(Arc::new(Mutex::new(0)))));
    engine.add_executor(Box::new(CollectExec(Arc::new(Mutex::new(vec![])))));

    // Should complete after collectors exhaust and channels close
    let res = engine.run_and_join().await;
    assert!(res.is_ok());
}
