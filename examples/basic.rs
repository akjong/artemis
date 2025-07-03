//! Basic example demonstrating how to use the Artemis framework.
//!
//! This example shows how to create collectors, strategies, and executors,
//! and how to wire them together using the Engine. It also demonstrates
//! the usage of convenience macros for working with enums.

use artemis::{
    Engine, map_collector, map_executor,
    types::{Collector, CollectorStream, Executor, Strategy},
};
use async_trait::async_trait;
use eyre::Result;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

/// Example event type representing a simple message
#[derive(Clone, Debug)]
pub struct MessageEvent {
    pub id: u64,
    pub content: String,
    pub timestamp: std::time::SystemTime,
}

/// Example action type representing a response to a message
#[derive(Clone, Debug)]
pub struct ResponseAction {
    pub original_id: u64,
    pub response: String,
}

/// A simple collector that generates message events
pub struct MessageCollector {
    message_count: u64,
}

impl MessageCollector {
    pub fn new() -> Self {
        Self { message_count: 10 }
    }
}

#[async_trait]
impl Collector<MessageEvent> for MessageCollector {
    fn name(&self) -> &str {
        "MessageCollector"
    }

    async fn get_event_stream(&self) -> Result<CollectorStream<'_, MessageEvent>> {
        let (tx, rx) = mpsc::unbounded_channel();
        let message_count = self.message_count;

        // Spawn a task to generate events
        tokio::spawn(async move {
            for i in 0..message_count {
                let event = MessageEvent {
                    id: i,
                    content: format!("Message {}", i),
                    timestamp: std::time::SystemTime::now(),
                };

                println!("📨 Generated event: {:?}", event);

                if tx.send(event).is_err() {
                    eprintln!("❌ Failed to send event, receiver dropped");
                    break;
                }

                // Add some delay between events
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            }

            println!("✅ MessageCollector finished generating events");
        });

        let stream = UnboundedReceiverStream::new(rx);
        Ok(Box::pin(stream))
    }
}

/// A system collector that generates system events
pub struct SystemCollector {
    event_count: u64,
}

impl SystemCollector {
    pub fn new() -> Self {
        Self { event_count: 3 }
    }
}

#[async_trait]
impl Collector<SystemEvent> for SystemCollector {
    fn name(&self) -> &str {
        "SystemCollector"
    }

    async fn get_event_stream(&self) -> Result<CollectorStream<'_, SystemEvent>> {
        let (tx, rx) = mpsc::unbounded_channel();
        let event_count = self.event_count;

        // Spawn a task to generate system events
        tokio::spawn(async move {
            for i in 0..event_count {
                let event = SystemEvent {
                    level: "INFO".to_string(),
                    message: format!("System status update {}", i),
                };

                println!("🖥️  Generated system event: {:?}", event);

                if tx.send(event).is_err() {
                    eprintln!("❌ Failed to send system event, receiver dropped");
                    break;
                }

                // Add some delay between events
                tokio::time::sleep(tokio::time::Duration::from_millis(800)).await;
            }

            println!("✅ SystemCollector finished generating events");
        });

        let stream = UnboundedReceiverStream::new(rx);
        Ok(Box::pin(stream))
    }
}

/// A strategy that processes events and creates actions
pub struct UnifiedStrategy;

#[async_trait]
impl Strategy<Event, Action> for UnifiedStrategy {
    fn name(&self) -> &str {
        "UnifiedStrategy"
    }

    async fn process_event(&mut self, event: Event) -> Vec<Action> {
        match event {
            Event::Message(msg_event) => {
                // Process message events and create response actions
                let action = ResponseAction {
                    original_id: msg_event.id,
                    response: format!("Echo: {}", msg_event.content),
                };

                println!(
                    "🔄 Strategy processed message event {} -> {:?}",
                    msg_event.id, action
                );
                vec![Action::Response(action)]
            }
            Event::System(sys_event) => {
                // Process system events and create log actions
                let action = LogAction {
                    level: sys_event.level.clone(),
                    content: format!("Logged: {}", sys_event.message),
                };

                println!("🔄 Strategy processed system event -> {:?}", action);
                vec![Action::Log(action)]
            }
        }
    }
}

/// An executor that handles response actions
pub struct PrintExecutor;

#[async_trait]
impl Executor<ResponseAction> for PrintExecutor {
    fn name(&self) -> &str {
        "PrintExecutor"
    }

    async fn execute(&self, action: ResponseAction) -> Result<()> {
        println!(
            "📤 Executing action for message {}: {}",
            action.original_id, action.response
        );

        // Simulate some work
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        Ok(())
    }
}

/// An executor that handles log actions
pub struct LogExecutor;

#[async_trait]
impl Executor<LogAction> for LogExecutor {
    fn name(&self) -> &str {
        "LogExecutor"
    }

    async fn execute(&self, action: LogAction) -> Result<()> {
        println!("📝 [{}] {}", action.level, action.content);

        // Simulate some work
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        Ok(())
    }
}

/// Combined event enum that can hold different event types
#[derive(Clone, Debug)]
pub enum Event {
    Message(MessageEvent),
    System(SystemEvent),
}

/// Combined action enum that can hold different action types
#[derive(Clone, Debug)]
pub enum Action {
    Response(ResponseAction),
    Log(LogAction),
}

/// System event type for demonstration
#[derive(Clone, Debug)]
pub struct SystemEvent {
    pub level: String,
    pub message: String,
}

/// Log action type for demonstration
#[derive(Clone, Debug)]
pub struct LogAction {
    pub level: String,
    pub content: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Starting Artemis basic example with macros...");

    // Create the engine with the unified event and action types
    let mut engine: Engine<Event, Action> = Engine::new();

    println!("📦 Demonstrating macro usage for type mapping...");

    // Add collectors using macros to map specific event types to the Event enum
    // The map_collector! macro automatically wraps the collector's output
    engine.add_collector(map_collector!(MessageCollector::new(), Event::Message));
    engine.add_collector(map_collector!(SystemCollector::new(), Event::System));

    // Without macros, you would need to write:
    // engine.add_collector(Box::new(CollectorMap::new(
    //     MessageCollector::new(),
    //     |event: MessageEvent| Event::Message(event)
    // )));
    // engine.add_collector(Box::new(CollectorMap::new(
    //     SystemCollector::new(),
    //     |event: SystemEvent| Event::System(event)
    // )));

    // Add the unified strategy that handles both event types
    engine.add_strategy(Box::new(UnifiedStrategy));

    // Add executors using macros to map specific action types from the Action enum
    // The map_executor! macro automatically filters and unwraps the actions
    engine.add_executor(map_executor!(PrintExecutor, Action::Response));
    engine.add_executor(map_executor!(LogExecutor, Action::Log));

    // Without macros, you would need to write:
    // engine.add_executor(Box::new(ExecutorMap::new(
    //     PrintExecutor,
    //     |action: Action| match action {
    //         Action::Response(response_action) => Some(response_action),
    //         _ => None,
    //     }
    // )));
    // engine.add_executor(Box::new(ExecutorMap::new(
    //     LogExecutor,
    //     |action: Action| match action {
    //         Action::Log(log_action) => Some(log_action),
    //         _ => None,
    //     }
    // )));

    println!("⚙️  Engine configured with:");
    println!("   📨 MessageCollector -> Event::Message (via map_collector! macro)");
    println!("   🖥️  SystemCollector -> Event::System (via map_collector! macro)");
    println!("   🔄 UnifiedStrategy handles both Event types");
    println!("   📤 PrintExecutor <- Action::Response (via map_executor! macro)");
    println!("   📝 LogExecutor <- Action::Log (via map_executor! macro)");

    // Run the engine
    println!("▶️  Starting engine...");

    match engine.run_and_join().await {
        Ok(_) => println!("🏁 Example with macros completed successfully!"),
        Err(e) => eprintln!("❌ Error running engine: {}", e),
    }

    println!("\n💡 Macro benefits demonstrated:");
    println!("   • map_collector! automatically wraps events in enum variants");
    println!("   • map_executor! automatically filters and unwraps actions from enums");
    println!("   • Clean separation between specific types and unified enum types");
    println!("   • Type-safe mapping without manual boilerplate code");

    Ok(())
}
