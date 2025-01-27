use clap::{Parser, Subcommand};
use colored::Colorize;
use std::error::Error;
use std::time::Duration;
use tokio::time;
use zenoh::{config::Config, Session};

mod message_registry;
use message_registry::MessageRegistry;

// Include the generated proto code
pub use self::proto::*;
pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/zspy.rs"));
}

// Include the auto-generated registry
mod registry {
    include!(concat!(env!("OUT_DIR"), "/registry.rs"));
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Subscribe and display messages for a given key
    Echo {
        /// The key expression to subscribe to
        key: String,
        /// Optional protobuf message type
        #[arg(long)]
        r#type: Option<String>,
    },
    /// Publish a message to a given key
    Pub {
        /// The key to publish to
        key: String,
        /// The message to publish (JSON format if type is specified)
        value: String,
        /// Optional protobuf message type
        #[arg(long)]
        r#type: Option<String>,
        /// Number of messages to publish (0 for infinite)
        #[arg(long, default_value = "1")]
        repeat: u64,
        /// Publishing rate in Hz
        #[arg(long, default_value = "1.0")]
        rate: f64,
    },
    /// List active publishers/subscribers
    List,
    /// Message type operations
    Types {
        #[command(subcommand)]
        command: TypeCommands,
    },
}

#[derive(Subcommand)]
enum TypeCommands {
    /// List available message types
    List,
    /// Show schema for a message type
    Show {
        /// The message type to show
        name: String,
    },
}

fn create_message_registry() -> MessageRegistry {
    let mut registry = MessageRegistry::new();
    registry::register_messages(&mut registry);
    registry
}

async fn handle_echo(
    session: &Session,
    key: &str,
    msg_type: Option<&str>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("Subscribing to key: {}", key.cyan());
    let registry = create_message_registry();
    let subscriber = session.declare_subscriber(key).await?;

    while let Ok(sample) = subscriber.recv_async().await {
        let payload = sample.payload().to_bytes();
        let display_value = if let Some(type_name) = msg_type {
            registry
                .decode(type_name, &payload)
                .unwrap_or_else(|e| format!("Error decoding message: {}", e))
        } else {
            String::from_utf8_lossy(&payload).to_string()
        };

        println!(
            ">> [{}] '{}': '{}'",
            "Received".green(),
            sample.key_expr().as_str().cyan(),
            display_value.yellow()
        );
    }

    Ok(())
}

async fn handle_pub(
    session: &Session,
    key: &str,
    value: &str,
    msg_type: Option<&str>,
    repeat: u64,
    rate: f64,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("Publishing to key: {}", key.cyan());
    println!("Value: {}", value.yellow());
    if repeat == 0 {
        println!("Publishing continuously at {} Hz", rate);
    } else {
        println!("Publishing {} messages at {} Hz", repeat, rate);
    }

    let registry = create_message_registry();
    let interval = Duration::from_secs_f64(1.0 / rate);
    let mut interval_timer = time::interval(interval);
    let mut count = 0;

    let payload = if let Some(type_name) = msg_type {
        registry.encode(type_name, value)?
    } else {
        value.as_bytes().to_vec()
    };

    loop {
        interval_timer.tick().await;
        session.put(key, payload.clone()).await?;
        count += 1;

        if repeat > 0 && count >= repeat {
            break;
        }

        print!("\rPublished {} messages", count);
        std::io::Write::flush(&mut std::io::stdout())?;
    }
    println!("\n{}", "Publishing completed!".green());

    Ok(())
}

async fn handle_list(session: &Session) -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("Discovering active publishers and subscribers...");
    // TODO: Implement proper discovery using Zenoh's discovery mechanisms
    println!("Connected to Zenoh session with ID: {}", session.zid());
    Ok(())
}

fn handle_types_list() {
    let registry = create_message_registry();
    println!("Available message types:");
    for msg_type in registry.list_types() {
        println!("  - {}", msg_type);
    }
}

fn handle_types_show(name: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
    let registry = create_message_registry();
    if let Some(schema) = registry.get_schema(name) {
        println!("Message type: {}", name);
        println!("Schema:\n{}", schema);
        Ok(())
    } else {
        Err(format!("Unknown message type: {}", name).into())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let cli = Cli::parse();

    // Initialize Zenoh session
    let config = Config::default();
    let session = zenoh::open(config).await?;

    match &cli.command {
        Commands::Echo { key, r#type } => {
            handle_echo(&session, key, r#type.as_deref()).await?;
        }
        Commands::Pub {
            key,
            value,
            r#type,
            repeat,
            rate,
        } => {
            handle_pub(&session, key, value, r#type.as_deref(), *repeat, *rate).await?;
        }
        Commands::List => {
            handle_list(&session).await?;
        }
        Commands::Types { command } => match command {
            TypeCommands::List => handle_types_list(),
            TypeCommands::Show { name } => handle_types_show(name)?,
        },
    }

    session.close().await?;
    Ok(())
}
