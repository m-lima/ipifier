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

    let mut current = client
        .request(&endpoint)
        .await
        .map_err(Error::Request)?
        .result;

    if let Some(entry) = current.pop() {
        if !current.is_empty() {
            return Err(Error::WrongRecordCount(current.len() + 1));
        }

        let endpoint = cloudflare::endpoints::dns::dns::UpdateDnsRecord {
            zone_identifier,
            identifier: &entry.id,
            params: cloudflare::endpoints::dns::dns::UpdateDnsRecordParams {
                name: &entry.name,
                content: {
                    match entry.content {
                        cloudflare::endpoints::dns::dns::DnsContent::A { content } => {
                            if content == ip {
                                tracing::info!("DNS entry needs no update");
                                return Ok(());
                            }
                            cloudflare::endpoints::dns::dns::DnsContent::A { content: ip }
                        }
                        content => return Err(Error::UnexpectedRecordType(content)),
                    }
                },
                proxied: Some(false),
                ttl: None,
            },
        };

        client
            .request(&endpoint)
            .await
            .map_err(Error::Update)
            .map(|_| ())
    } else {
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
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Unable to initialize cloudflare client: {0}")]
    Client(cloudflare::framework::Error),
    #[error("Unable to make request to cloudflare: {0}")]
    Request(cloudflare::framework::response::ApiFailure),
    #[error("Unable to update the DNS record: {0}")]
    Update(cloudflare::framework::response::ApiFailure),
    #[error("Expected at most a single record, but got {0}")]
    WrongRecordCount(usize),
    #[error("Unexpcted content type: {0:?}")]
    UnexpectedRecordType(cloudflare::endpoints::dns::dns::DnsContent),
}
