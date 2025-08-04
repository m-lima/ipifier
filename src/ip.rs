pub async fn fetch(providers: Vec<url::Url>) -> Option<std::net::Ipv4Addr> {
    let client = reqwest::Client::new();
    let mut visited = std::collections::HashSet::with_capacity(providers.len());

    for provider in providers.into_iter().chain(default_providers()) {
        if visited.contains(&provider) {
            continue;
        }

        if let Ok(ip) = fetch_inner(&client, provider.clone()).await {
            return Some(ip);
        }

        visited.insert(provider);
    }

    None
}

#[tracing::instrument(ret, err, skip_all, fields(%provider), "fetch")]
async fn fetch_inner(
    client: &reqwest::Client,
    provider: url::Url,
) -> Result<std::net::Ipv4Addr, Error> {
    client
        .get(provider)
        .send()
        .await
        .map_err(Error::Request)?
        .text()
        .await
        .map_err(Error::Body)?
        .parse()
        .map_err(Error::Parse)
}

fn default_providers() -> [url::Url; 2] {
    [
        url::Url::parse("https://api.ipify.org").unwrap(),
        url::Url::parse("https://ipinfo.io/ip").unwrap(),
    ]
}

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("Failed to make request: {0}")]
    Request(reqwest::Error),
    #[error("Failed to extract body: {0}")]
    Body(reqwest::Error),
    #[error("Failed to parse body: {0}")]
    Parse(std::net::AddrParseError),
}
