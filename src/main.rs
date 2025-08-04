#![warn(clippy::pedantic)]

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
    let providers = std::env::args()
        .skip(1)
        .map(Provider::Custom)
        .chain([Provider::Ipfy, Provider::IpInfo])
        .map(Provider::url)
        .collect::<Result<_, _>>()
        .map_err(Error::Url)?;

    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(Error::Runtime)?
        .block_on(async_main(providers))
}

async fn async_main(providers: Vec<url::Url>) -> Result {
    let ip = try_fetch_ip(providers).await.ok_or(Error::NoIp)?;
    Ok(())
}

async fn try_fetch_ip(providers: Vec<url::Url>) -> Option<std::net::Ipv4Addr> {
    #[tracing::instrument(ret, err, skip_all, fields(%provider), "Fetching IP")]
    async fn fetch_ip(
        client: &reqwest::Client,
        provider: url::Url,
    ) -> Result<std::net::Ipv4Addr, IpFetchError> {
        client
            .get(provider)
            .send()
            .await
            .map_err(IpFetchError::Request)?
            .text()
            .await
            .map_err(IpFetchError::Body)?
            .parse()
            .map_err(IpFetchError::Parse)
    }

    let client = reqwest::Client::new();
    let mut visited = std::collections::HashSet::with_capacity(providers.len());
    for provider in providers {
        if visited.contains(&provider) {
            continue;
        }
        if let Ok(ip) = fetch_ip(&client, provider.clone()).await {
            return Some(ip);
        }
        visited.insert(provider);
    }

    None
}

#[derive(Clone, Debug)]
enum Provider {
    Custom(String),
    Ipfy,
    IpInfo,
}

impl Provider {
    fn url(self) -> std::result::Result<reqwest::Url, url::ParseError> {
        match self {
            Self::Custom(ref string) => reqwest::Url::parse(string),
            Self::Ipfy => reqwest::Url::parse("https://api.ipify.org"),
            Self::IpInfo => reqwest::Url::parse("https://ipinfo.io/ip"),
        }
    }
}

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("Failed to initialize logging: {0}")]
    Tracing(#[from] tracing::subscriber::SetGlobalDefaultError),
    #[error("Failed to parse url: {0}")]
    Url(url::ParseError),
    #[error("Failed to initialize runtime: {0}")]
    Runtime(std::io::Error),
    #[error("Unable to fetch IP from any provider")]
    NoIp,
}

#[derive(Debug, thiserror::Error)]
enum IpFetchError {
    #[error("Failed to make request: {0}")]
    Request(reqwest::Error),
    #[error("Failed to extract body: {0}")]
    Body(reqwest::Error),
    #[error("Failed to parse body: {0}")]
    Parse(std::net::AddrParseError),
}
