#![warn(rustdoc::invalid_rust_codeblocks)]
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

//! This library just starts and configures a logger and provides a friendly shutdown process for tokio apps.
//! Check [init] to see how to use it
use std::collections::HashMap;

use signal_hook::consts::signal::*;
use signal_hook_tokio::{Signals, Handle};
use futures::stream::StreamExt;
use log::info;
use anyhow::Result;
use tokio::task::JoinHandle;

/// Signal handler. It publish a message into the shutdown channel
async fn handle_signals(mut signals: Signals, tx: flume::Sender<i32>) {
    while let Some(signal) = signals.next().await {
        match signal {
            SIGHUP => {
                info!("Reloading");
            }
            SIGTERM | SIGINT | SIGQUIT => {
                info!("Exit signal received, doing friendly shutdown...");
                tx.send_async(0).await.unwrap();
                // Shutdown the system;
            },
            _ => unreachable!(),
        }
    }
}

/// Configures the logger format. If outfile is none, no log file will be written
pub fn setup_logger(outfile: Option<&str>, level: Option<log::LevelFilter>, extra_levels:  Option<HashMap<&str, log::LevelFilter>>) -> Result<()>{
    let mut chain = fern::Dispatch::new()
    // Perform allocation-free log formatting
    .format(|out, message, record| {
        out.finish(format_args!(
            "{}[{}][{}] {}",
            chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
            record.target(),
            record.level(),
            message
        ))
    })
    // Add blanket level filter -
    .level(level.unwrap_or(log::LevelFilter::Info));

    if let Some(extra_levels) = extra_levels {
        for (module, level) in extra_levels.into_iter() {
            chain = chain.level_for(module.to_owned(), level);
        }
    }
    
    
    if let Some(outfile) = outfile {
        chain = chain.chain(fern::log_file(outfile)?);
    }
    chain.chain(std::io::stdout())
    
    .apply()?;
    Ok(())

}

/// The shutdown context. It stores information about the signal handler task
pub struct InitContext {
    handle: Handle,
    join: JoinHandle<()>
}

impl InitContext {
    /// When the program ends, you should call this to friendly stop the signal handler
    pub async fn stop(self) -> Result<()> {
        self.handle.close();
        self.join.await?;
        Ok(())
    }
}

/// The main function. Call it as soon as possible in your program. It returns the context
/// and the channel to listen the exit requests
/// 
/// ```
/// use wixet_bootstrap::init;
/// use log::info;
/// 
/// #[tokio::main]
/// async fn main() {
///     info!("This log line will be ignored because the logger is not configured yet");
///     let (closer, exit) = init(Some("output.log")).await?; //If you provide None, it simple will not write a log file (just output)
///     info!("Hello to my application!")
/// 
///     // Do may awesome stuff spawing tokio tasks
/// 
///     // I use select here because it is common to listen for multiple signals, but you can just await the `exit` if not
///     tokio::select!{
///         _ = exit.recv_async() => {
///         info!("Shutdown process started");
///         // Do your friendly stop process here
///         // This code is run when ctrl+c or any other kill interrupt is received
///     }
/// };
/// 
/// // A friendly shutdown by deinitializing all "init" stuff.
/// closer.stop().await?;
/// info!("Bye");
/// 
/// }
/// ```
pub async fn init(log_file: Option<&str>, level: Option<log::LevelFilter>, extra_levels:  Option<HashMap<&str, log::LevelFilter>>) -> Result<(InitContext, flume::Receiver<i32>)> {
    setup_logger(log_file, level, extra_levels)?;
    // Signals
    let signals = Signals::new([
        SIGHUP,
        SIGTERM,
        SIGINT,
        SIGQUIT,
    ])?;
    let handle = signals.handle();
    let (tx, rx) = flume::unbounded();
    let signals_task = tokio::spawn(handle_signals(signals, tx));
    Ok((InitContext{handle, join: signals_task}, rx))
}