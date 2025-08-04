pub struct Args {
    pub cname: String,
    pub domain: String,
    pub providers: Vec<url::Url>,
}

pub fn parse() -> Result<Args, Error> {
    let mut args = std::env::args().skip(1);

    let cname = args.next().ok_or(Error::CName)?;
    let domain = args.next().ok_or(Error::Domain)?;

    let providers = args
        .map(|p| url::Url::parse(&p))
        .collect::<Result<_, _>>()
        .map_err(Error::Provider)?;

    Ok(Args {
        cname,
        domain,
        providers,
    })
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Missing cname parameter")]
    CName,
    #[error("Missing domain parameter")]
    Domain,
    #[error("Invalid provider parameter")]
    Provider(url::ParseError),
}
