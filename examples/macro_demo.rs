//! Macro demonstration example for Artemis framework.
//!
//! This example specifically focuses on demonstrating the convenience macros
//! provided by Artemis for working with enums and type mapping.

use artemis::{
    Engine, map_collector, map_executor,
    types::{Collector, CollectorStream, Executor, Strategy},
};
use async_trait::async_trait;
use eyre::Result;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

// Define specific event types
#[derive(Clone, Debug)]
pub struct NumberEvent {
    pub value: i32,
}

#[derive(Clone, Debug)]
pub struct TextEvent {
    pub message: String,
}

// Define a unified event enum
#[derive(Clone, Debug)]
pub enum Event {
    Number(NumberEvent),
    Text(TextEvent),
}

// Define specific action types
#[derive(Clone, Debug)]
pub struct DoubleAction {
    pub result: i32,
}

#[derive(Clone, Debug)]
pub struct UppercaseAction {
    pub result: String,
}

// Define a unified action enum
#[derive(Clone, Debug)]
pub enum Action {
    Double(DoubleAction),
    Uppercase(UppercaseAction),
}

// Collectors for specific event types
pub struct NumberCollector;

#[async_trait]
impl Collector<NumberEvent> for NumberCollector {
    fn name(&self) -> &str {
        "NumberCollector"
    }

    async fn get_event_stream(&self) -> Result<CollectorStream<'_, NumberEvent>> {
        let (tx, rx) = mpsc::unbounded_channel();

        tokio::spawn(async move {
            for i in 1..=3 {
                let event = NumberEvent { value: i };
                println!("🔢 NumberCollector generated: {:?}", event);
                if tx.send(event).is_err() {
                    break;
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
            }
        });

        Ok(Box::pin(UnboundedReceiverStream::new(rx)))
    }
}

pub struct TextCollector;

#[async_trait]
impl Collector<TextEvent> for TextCollector {
    fn name(&self) -> &str {
        "TextCollector"
    }

    async fn get_event_stream(&self) -> Result<CollectorStream<'_, TextEvent>> {
        let (tx, rx) = mpsc::unbounded_channel();

        tokio::spawn(async move {
            let messages = ["hello", "world", "artemis"];
            for msg in messages.iter() {
                let event = TextEvent {
                    message: msg.to_string(),
                };
                println!("📝 TextCollector generated: {:?}", event);
                if tx.send(event).is_err() {
                    break;
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(400)).await;
            }
        });

        Ok(Box::pin(UnboundedReceiverStream::new(rx)))
    }
}

// Strategy that handles unified events
pub struct UnifiedStrategy;

#[async_trait]
impl Strategy<Event, Action> for UnifiedStrategy {
    fn name(&self) -> &str {
        "UnifiedStrategy"
    }

    async fn process_event(&mut self, event: Event) -> Vec<Action> {
        match event {
            Event::Number(num_event) => {
                let action = DoubleAction {
                    result: num_event.value * 2,
                };
                println!(
                    "🔄 Processing number {} -> double {}",
                    num_event.value, action.result
                );
                vec![Action::Double(action)]
            }
            Event::Text(text_event) => {
                let action = UppercaseAction {
                    result: text_event.message.to_uppercase(),
                };
                println!(
                    "🔄 Processing text '{}' -> uppercase '{}'",
                    text_event.message, action.result
                );
                vec![Action::Uppercase(action)]
            }
        }
    }
}

// Executors for specific action types
pub struct DoubleExecutor;

#[async_trait]
impl Executor<DoubleAction> for DoubleExecutor {
    fn name(&self) -> &str {
        "DoubleExecutor"
    }

    async fn execute(&self, action: DoubleAction) -> Result<()> {
        println!("🎯 DoubleExecutor: Result = {}", action.result);
        Ok(())
    }
}

pub struct UppercaseExecutor;

#[async_trait]
impl Executor<UppercaseAction> for UppercaseExecutor {
    fn name(&self) -> &str {
        "UppercaseExecutor"
    }

    async fn execute(&self, action: UppercaseAction) -> Result<()> {
        println!("🎯 UppercaseExecutor: Result = '{}'", action.result);
        Ok(())
    }
}

fn demonstrate_macro_variants() {
    println!("\n📚 Available Artemis Macros vs Manual Implementation:");

    println!("\n1. 🔧 map_collector!(collector, EnumVariant)");
    println!("   ✅ Macro: map_collector!(NumberCollector, Event::Number)");
    println!("   ❌ Manual: Box::new(CollectorMap::new(NumberCollector, |e| Event::Number(e)))");
    println!("   - Wraps collector output in enum variant");
    println!("   - Automatically boxes the collector");

    println!("\n2. 🔧 map_boxed_collector!(boxed_collector, EnumVariant)");
    println!("   ✅ Macro: map_boxed_collector!(Box::new(collector), Event::Variant)");
    println!("   ❌ Manual: Box::new(CollectorMap::new(*boxed_collector, closure))");
    println!("   - Same as map_collector! but takes already boxed collector");

    println!("\n3. 🔧 map_executor!(executor, EnumVariant)");
    println!("   ✅ Macro: map_executor!(DoubleExecutor, Action::Double)");
    println!("   ❌ Manual: Box::new(ExecutorMap::new(DoubleExecutor, |a| match a {{");
    println!("              Action::Double(val) => Some(val), _ => None }}))");
    println!("   - Filters enum actions and passes specific type to executor");
    println!("   - Automatically boxes the executor");

    println!("\n4. 🔧 map_boxed_executor!(boxed_executor, EnumVariant)");
    println!("   ✅ Macro: map_boxed_executor!(Box::new(executor), Action::Variant)");
    println!("   ❌ Manual: Box::new(ExecutorMap::new(*boxed_executor, match_closure))");
    println!("   - Same as map_executor! but takes already boxed executor");

    println!("\n🔄 Example transformations:");
    println!("• NumberEvent --[map_collector!]--> Event::Number(NumberEvent)");
    println!("• Event::Double(DoubleAction) --[map_executor!]--> DoubleAction");

    println!("\n💡 Benefits of using macros:");
    println!("   🚀 Less verbose code (3-5x shorter)");
    println!("   🛡️  Type safety with compile-time checks");
    println!("   🎯 Clear intent and readable code");
    println!("   ⚡ No runtime overhead");
}

fn demonstrate_without_macros() {
    println!("\n🔍 Code Comparison - With vs Without Macros:");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    println!("\n📝 Collector Setup:");
    println!("✅ With macro:");
    println!("   engine.add_collector(map_collector!(NumberCollector, Event::Number));");
    println!();
    println!("❌ Without macro (manual):");
    println!("   engine.add_collector(Box::new(CollectorMap::new(");
    println!("       NumberCollector,");
    println!("       |event: NumberEvent| Event::Number(event)");
    println!("   )));");

    println!("\n📝 Executor Setup:");
    println!("✅ With macro:");
    println!("   engine.add_executor(map_executor!(DoubleExecutor, Action::Double));");
    println!();
    println!("❌ Without macro (manual):");
    println!("   engine.add_executor(Box::new(ExecutorMap::new(");
    println!("       DoubleExecutor,");
    println!("       |action: Action| match action {{");
    println!("           Action::Double(double_action) => Some(double_action),");
    println!("           _ => None,");
    println!("       }}");
    println!("   )));");

    println!("\n📊 Line Count Comparison:");
    println!("   Macro approach:  1 line per component");
    println!("   Manual approach: 5-7 lines per component");
    println!("   Code reduction:  ~80% less boilerplate!");
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Artemis Macro Demonstration Example");
    println!("======================================");

    demonstrate_macro_variants();
    demonstrate_without_macros();

    println!("\n🏗️  Building engine with macros...");

    let mut engine: Engine<Event, Action> = Engine::new();

    // Method 1: Using map_collector! macro (recommended)
    println!("\n📦 Adding collectors with map_collector! macro:");
    engine.add_collector(map_collector!(NumberCollector, Event::Number));
    engine.add_collector(map_collector!(TextCollector, Event::Text));

    // Without macros, you would need to manually create CollectorMap:
    // engine.add_collector(Box::new(CollectorMap::new(
    //     NumberCollector,
    //     |event: NumberEvent| Event::Number(event)
    // )));
    // engine.add_collector(Box::new(CollectorMap::new(
    //     TextCollector,
    //     |event: TextEvent| Event::Text(event)
    // )));

    // Method 2: Using map_boxed_collector! macro (when you already have boxed collectors)
    println!("   Alternative: map_boxed_collector!(Box::new(collector), Event::Variant)");
    // engine.add_collector(map_boxed_collector!(Box::new(NumberCollector), Event::Number));

    // Add strategy
    engine.add_strategy(Box::new(UnifiedStrategy));

    // Method 1: Using map_executor! macro (recommended)
    println!("\n📦 Adding executors with map_executor! macro:");
    engine.add_executor(map_executor!(DoubleExecutor, Action::Double));
    engine.add_executor(map_executor!(UppercaseExecutor, Action::Uppercase));

    // Without macros, you would need to manually create ExecutorMap:
    // engine.add_executor(Box::new(ExecutorMap::new(
    //     DoubleExecutor,
    //     |action: Action| match action {
    //         Action::Double(double_action) => Some(double_action),
    //         _ => None,
    //     }
    // )));
    // engine.add_executor(Box::new(ExecutorMap::new(
    //     UppercaseExecutor,
    //     |action: Action| match action {
    //         Action::Uppercase(uppercase_action) => Some(uppercase_action),
    //         _ => None,
    //     }
    // )));

    // Method 2: Using map_boxed_executor! macro (when you already have boxed executors)
    println!("   Alternative: map_boxed_executor!(Box::new(executor), Action::Variant)");
    // engine.add_executor(map_boxed_executor!(Box::new(DoubleExecutor), Action::Double));

    println!("\n▶️  Running engine...");

    match engine.run_and_join().await {
        Ok(_) => {
            println!("\n🏁 Macro demonstration completed successfully!");
            println!("\n💡 Key Benefits of Using Macros:");
            println!("   ✅ Reduced boilerplate code");
            println!("   ✅ Type-safe mapping between specific and enum types");
            println!("   ✅ Automatic boxing and trait object creation");
            println!("   ✅ Clear, readable code expressing intent");
            println!("   ✅ Compile-time verification of enum variants");
        }
        Err(e) => eprintln!("❌ Error: {}", e),
    }

    Ok(())
}
