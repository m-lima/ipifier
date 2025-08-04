#![warn(clippy::pedantic)]

mod args;
mod dns;
mod ip;

type Result<T = (), E = Error> = std::result::Result<T, E>;

fn main() -> std::process::ExitCode {
    if let Err(error) = setup_tracing() {
        eprintln!("{error}");
        return std::process::ExitCode::FAILURE;
    }

    match fallible_main() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            tracing::error!(%error);
            std::process::ExitCode::FAILURE
        }
    }
}

fn setup_tracing() -> Result {
    use tracing_subscriber::layer::SubscriberExt as _;

    let layer = treetrace::Layer::new(treetrace::Stderr, false, false);
    let subscriber = tracing_subscriber::registry().with(layer).with(
        tracing::level_filters::LevelFilter::from_level(tracing::Level::INFO),
    );
    tracing::subscriber::set_global_default(subscriber).map_err(Error::Tracing)
}

fn fallible_main() -> Result {
    let args = args::parse()?;

    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(Error::Runtime)?
        .block_on(async_main(args))
}

async fn async_main(args: args::Args) -> Result {
    let ip = ip::fetch(args.providers).await.ok_or(Error::NoIp)?;
    if let Err(error) = dns::update(args.token, args.record, &args.zone_id, ip).await {
        eprintln!("{error:#?}");
    }
    Ok(())
}

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("Failed to initialize logging: {0}")]
    Tracing(#[from] tracing::subscriber::SetGlobalDefaultError),
    #[error(transparent)]
    Args(#[from] args::Error),
    #[error("Failed to initialize runtime: {0}")]
    Runtime(std::io::Error),
    #[error("Unable to fetch IP from any provider")]
    NoIp,
}
