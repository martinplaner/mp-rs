mod generator;
mod routes;

use crate::generator::{CompoundGenerator, Generator};
use crate::routes::{routes, AppContext};
use clap::Parser;
use std::fs::File;
use std::io::BufReader;
use std::path;
use std::process::exit;
use std::sync::Arc;
use tokio::signal;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Path to word list (one word per line)
    #[arg(short, long, default_value = "words.txt")]
    file: String,

    /// TCP address for the server to listen on, in the form 'host:port'
    #[arg(short, long, default_value = "0.0.0.0:8080")]
    listen: String,

    /// Default fallback query term, if not provided
    #[arg(short, long, default_value = "MP")]
    default_query: String,

    /// Run generation once with the given query and print result, then quit
    #[arg(short, long)]
    once: Option<String>,
}

const DELIMITER: &str = "-";

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let file = File::open(path::Path::new(&cli.file)).expect("Error loading file");
    let reader = BufReader::new(file);

    let generator =
        CompoundGenerator::new(reader, DELIMITER).expect("Error initializing generator");

    if let Some(term) = cli.once {
        let word = generator.generate(&term).expect("Error generating word");
        println!("{}", word);
        exit(0);
    };

    let router = routes(AppContext {
        generator: Arc::new(generator),
        default_term: cli.default_query,
    });

    let listener = tokio::net::TcpListener::bind(cli.listen)
        .await
        .expect("Can't bind to address");

    println!("Listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
