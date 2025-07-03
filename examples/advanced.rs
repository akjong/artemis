//! Advanced example demonstrating enum-based event/action types with mapping.
//!
//! This example shows how to work with multiple event and action types
//! using enums and the provided mapping macros.

use artemis::{
    Engine, map_collector, map_executor,
    types::{Collector, CollectorStream, Executor, Strategy},
};
use async_trait::async_trait;
use eyre::Result;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

/// Combined event enum that can hold different event types
#[derive(Clone, Debug)]
pub enum Event {
    Block(BlockEvent),
    Transaction(TransactionEvent),
}

/// Combined action enum that can hold different action types
#[derive(Clone, Debug)]
pub enum Action {
    Log(LogAction),
    Alert(AlertAction),
}

/// Block event from a blockchain
#[derive(Clone, Debug)]
pub struct BlockEvent {
    pub block_number: u64,
    pub hash: String,
    pub transaction_count: u32,
}

/// Transaction event from a blockchain
#[derive(Clone, Debug)]
pub struct TransactionEvent {
    pub tx_hash: String,
    pub from: String,
    pub to: String,
    pub value: u64,
}

/// Log action for recording information
#[derive(Clone, Debug)]
pub struct LogAction {
    pub level: String,
    pub message: String,
}

/// Alert action for notifications
#[derive(Clone, Debug)]
pub struct AlertAction {
    pub severity: String,
    pub title: String,
    pub description: String,
}

/// Collector that generates block events
pub struct BlockCollector {
    block_count: u64,
}

impl BlockCollector {
    pub fn new(block_count: u64) -> Self {
        Self { block_count }
    }
}

#[async_trait]
impl Collector<BlockEvent> for BlockCollector {
    fn name(&self) -> &str {
        "BlockCollector"
    }

    async fn get_event_stream(&self) -> Result<CollectorStream<'_, BlockEvent>> {
        let (tx, rx) = mpsc::unbounded_channel();
        let block_count = self.block_count;

        tokio::spawn(async move {
            for i in 1..=block_count {
                let event = BlockEvent {
                    block_number: i,
                    hash: format!("0x{:064x}", i),
                    transaction_count: (i % 10 + 1) as u32,
                };

                println!("🟦 Generated block event: {:?}", event);

                if tx.send(event).is_err() {
                    break;
                }

                tokio::time::sleep(tokio::time::Duration::from_millis(800)).await;
            }
        });

        Ok(Box::pin(UnboundedReceiverStream::new(rx)))
    }
}

/// Collector that generates transaction events
pub struct TransactionCollector {
    tx_count: u64,
}

impl TransactionCollector {
    pub fn new(tx_count: u64) -> Self {
        Self { tx_count }
    }
}

#[async_trait]
impl Collector<TransactionEvent> for TransactionCollector {
    fn name(&self) -> &str {
        "TransactionCollector"
    }

    async fn get_event_stream(&self) -> Result<CollectorStream<'_, TransactionEvent>> {
        let (tx, rx) = mpsc::unbounded_channel();
        let tx_count = self.tx_count;

        tokio::spawn(async move {
            for i in 1..=tx_count {
                let event = TransactionEvent {
                    tx_hash: format!("0x{:064x}", i + 1000),
                    from: format!("0x{:040x}", i),
                    to: format!("0x{:040x}", i + 1),
                    value: i * 1000,
                };

                println!("💳 Generated transaction event: {:?}", event);

                if tx.send(event).is_err() {
                    break;
                }

                tokio::time::sleep(tokio::time::Duration::from_millis(600)).await;
            }
        });

        Ok(Box::pin(UnboundedReceiverStream::new(rx)))
    }
}

/// Strategy that processes block events and creates log actions
pub struct BlockAnalysisStrategy;

#[async_trait]
impl Strategy<Event, Action> for BlockAnalysisStrategy {
    fn name(&self) -> &str {
        "BlockAnalysisStrategy"
    }

    async fn process_event(&mut self, event: Event) -> Vec<Action> {
        match event {
            Event::Block(block_event) => {
                let mut actions = vec![];

                // Always log block information
                actions.push(Action::Log(LogAction {
                    level: "INFO".to_string(),
                    message: format!(
                        "Block {} processed with {} transactions",
                        block_event.block_number, block_event.transaction_count
                    ),
                }));

                // Alert if block has many transactions
                if block_event.transaction_count > 5 {
                    actions.push(Action::Alert(AlertAction {
                        severity: "HIGH".to_string(),
                        title: "High Activity Block".to_string(),
                        description: format!(
                            "Block {} contains {} transactions",
                            block_event.block_number, block_event.transaction_count
                        ),
                    }));
                }

                println!(
                    "📊 BlockAnalysisStrategy generated {} actions",
                    actions.len()
                );
                actions
            }
            _ => {
                // This strategy only handles block events
                vec![]
            }
        }
    }
}

/// Strategy that processes transaction events and creates actions
pub struct TransactionMonitorStrategy;

#[async_trait]
impl Strategy<Event, Action> for TransactionMonitorStrategy {
    fn name(&self) -> &str {
        "TransactionMonitorStrategy"
    }

    async fn process_event(&mut self, event: Event) -> Vec<Action> {
        match event {
            Event::Transaction(tx_event) => {
                let mut actions = vec![];

                // Log all transactions
                actions.push(Action::Log(LogAction {
                    level: "DEBUG".to_string(),
                    message: format!(
                        "Transaction {} from {} to {} with value {}",
                        tx_event.tx_hash, tx_event.from, tx_event.to, tx_event.value
                    ),
                }));

                // Alert for high-value transactions
                if tx_event.value > 5000 {
                    actions.push(Action::Alert(AlertAction {
                        severity: "MEDIUM".to_string(),
                        title: "High Value Transaction".to_string(),
                        description: format!(
                            "Transaction {} with value {} detected",
                            tx_event.tx_hash, tx_event.value
                        ),
                    }));
                }

                println!(
                    "🔍 TransactionMonitorStrategy generated {} actions",
                    actions.len()
                );
                actions
            }
            _ => {
                // This strategy only handles transaction events
                vec![]
            }
        }
    }
}

/// Executor that handles log actions
pub struct LogExecutor;

#[async_trait]
impl Executor<LogAction> for LogExecutor {
    fn name(&self) -> &str {
        "LogExecutor"
    }

    async fn execute(&self, action: LogAction) -> Result<()> {
        println!("📝 [{}] {}", action.level, action.message);
        Ok(())
    }
}

/// Executor that handles alert actions
pub struct AlertExecutor;

#[async_trait]
impl Executor<AlertAction> for AlertExecutor {
    fn name(&self) -> &str {
        "AlertExecutor"
    }

    async fn execute(&self, action: AlertAction) -> Result<()> {
        println!(
            "🚨 [{}] {}: {}",
            action.severity, action.title, action.description
        );

        // Simulate sending notification
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Starting Artemis advanced example...");

    // Create the engine
    let mut engine = Engine::new();

    // Add mapped collectors - these wrap the specific event types in the Event enum
    // The map_collector! macro automatically converts:
    //   BlockEvent -> Event::Block(BlockEvent)
    //   TransactionEvent -> Event::Transaction(TransactionEvent)
    engine.add_collector(map_collector!(BlockCollector::new(5), Event::Block));
    engine.add_collector(map_collector!(
        TransactionCollector::new(8),
        Event::Transaction
    ));

    // Without macros, you would need to manually create the mapping:
    // use artemis::types::CollectorMap;
    // engine.add_collector(Box::new(CollectorMap::new(
    //     BlockCollector::new(5),
    //     |event: BlockEvent| Event::Block(event)
    // )));
    // engine.add_collector(Box::new(CollectorMap::new(
    //     TransactionCollector::new(8),
    //     |event: TransactionEvent| Event::Transaction(event)
    // )));

    // Add strategies that work with the combined Event enum
    engine.add_strategy(Box::new(BlockAnalysisStrategy));
    engine.add_strategy(Box::new(TransactionMonitorStrategy));

    // Add mapped executors - these extract specific actions from the Action enum
    // The map_executor! macro automatically filters and converts:
    //   Action::Log(LogAction) -> LogAction (passed to LogExecutor)
    //   Action::Alert(AlertAction) -> AlertAction (passed to AlertExecutor)
    //   Other Action variants are ignored by each executor
    engine.add_executor(map_executor!(LogExecutor, Action::Log));
    engine.add_executor(map_executor!(AlertExecutor, Action::Alert));

    // Without macros, you would need to manually create the pattern matching:
    // use artemis::types::ExecutorMap;
    // engine.add_executor(Box::new(ExecutorMap::new(
    //     LogExecutor,
    //     |action: Action| match action {
    //         Action::Log(log_action) => Some(log_action),
    //         _ => None,
    //     }
    // )));
    // engine.add_executor(Box::new(ExecutorMap::new(
    //     AlertExecutor,
    //     |action: Action| match action {
    //         Action::Alert(alert_action) => Some(alert_action),
    //         _ => None,
    //     }
    // )));

    println!("⚙️  Engine configured with multiple collectors, strategies, and executors");

    // Run the engine
    println!("▶️  Starting engine...");

    match engine.run_and_join().await {
        Ok(_) => {
            println!("🏁 Advanced example completed successfully!");
            println!("\n💡 Macro benefits in this advanced example:");
            println!("   ✅ Simplified enum mapping for multiple event/action types");
            println!("   ✅ Type-safe filtering without manual pattern matching");
            println!(
                "   ✅ Clean separation between specific collectors/executors and unified engine"
            );
            println!("   ✅ Reduced boilerplate code for complex type hierarchies");
        }
        Err(e) => eprintln!("❌ Error running engine: {}", e),
    }

    Ok(())
}
