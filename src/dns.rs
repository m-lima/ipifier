enum Endpoint<'a> {
    NotFound,
    NoChange,
    Update(cloudflare::endpoints::dns::dns::UpdateDnsRecord<'a>),
}
fn get_endpoint<'a, 'z: 'a>(
    entries: &'a [cloudflare::endpoints::dns::dns::DnsRecord],
    zone_identifier: &'z str,
    ip: std::net::Ipv4Addr,
) -> Endpoint<'a> {
    for entry in entries {
        if let cloudflare::endpoints::dns::dns::DnsContent::A { content } = entry.content {
            if content == ip {
                tracing::info!("DNS entry needs no update");
                return Endpoint::NoChange;
            }
            return Endpoint::Update(cloudflare::endpoints::dns::dns::UpdateDnsRecord {
                zone_identifier,
                identifier: &entry.id,
                params: cloudflare::endpoints::dns::dns::UpdateDnsRecordParams {
                    name: &entry.name,
                    content: cloudflare::endpoints::dns::dns::DnsContent::A { content: ip },
                    proxied: Some(false),
                    ttl: None,
                },
            });
        }
    }
    Endpoint::NotFound
}
pub async fn update(
    token: String,
    record: String,
    zone_identifier: &str,
    ip: std::net::Ipv4Addr,
) -> Result<(), Error> {
    let client = cloudflare::framework::client::async_api::Client::new(
        cloudflare::framework::auth::Credentials::UserAuthToken { token },
        cloudflare::framework::client::ClientConfig::default(),
        cloudflare::framework::Environment::Production,
    )
    .map_err(Error::Client)?;

    let endpoint = cloudflare::endpoints::dns::dns::ListDnsRecords {
        zone_identifier,
        params: cloudflare::endpoints::dns::dns::ListDnsRecordsParams {
            name: Some(record.clone()),
            ..Default::default()
        },
    };

    let current = client
        .request(&endpoint)
        .await
        .map_err(Error::Request)?
        .result;

    match get_endpoint(&current, zone_identifier, ip) {
        Endpoint::NotFound => {
            let endpoint = cloudflare::endpoints::dns::dns::CreateDnsRecord {
                zone_identifier,
                params: cloudflare::endpoints::dns::dns::CreateDnsRecordParams {
                    name: &record,
                    content: cloudflare::endpoints::dns::dns::DnsContent::A { content: ip },
                    proxied: Some(false),
                    ttl: None,
                    priority: None,
                },
            };

            client
                .request(&endpoint)
                .await
                .map_err(Error::Update)
                .map(|_| ())
        }
        Endpoint::NoChange => Ok(()),
        Endpoint::Update(endpoint) => client
            .request(&endpoint)
            .await
            .map_err(Error::Update)
            .map(|_| ()),
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Unable to initialize cloudflare client: {0}")]
    Client(cloudflare::framework::Error),
    #[error("Unable to make request to cloudflare: {0}")]
    Request(cloudflare::framework::response::ApiFailure),
    #[error("Unable to update the DNS record: {0}")]
    Update(cloudflare::framework::response::ApiFailure),
}
