pub struct Args {
    pub token: String,
    pub record: String,
    pub zone_id: String,
    pub providers: Vec<url::Url>,
}

pub fn parse() -> Result<Args, Error> {
    let mut args = std::env::args().skip(1);

    let token = args.next().ok_or(Error::Token)?;
    let record = args.next().ok_or(Error::Record)?;
    let zone_id = args.next().ok_or(Error::ZoneId)?;

    let providers = args
        .map(|p| url::Url::parse(&p))
        .collect::<Result<_, _>>()
        .map_err(Error::Provider)?;

    Ok(Args {
        token,
        record,
        zone_id,
        providers,
    })
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Missing token parameter")]
    Token,
    #[error("Missing record parameter")]
    Record,
    #[error("Missing zone_id parameter")]
    ZoneId,
    #[error("Invalid provider parameter")]
    Provider(url::ParseError),
}
