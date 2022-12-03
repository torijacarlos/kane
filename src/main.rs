use belvo::account::api as account_api;
use belvo::client::{BelvoClient, BelvoKey, Environment};
use belvo::link::{api as link_api, AccessMode, LinkBase, LinkDetail, LinkFilters, LinkStatus};
use serde::Deserialize;
use std::error::Error;
use std::fs::File;
use std::io::{Error as IoError, Read};

#[derive(Deserialize)]
struct Config {
    belvo: BelvoKey,
    institution: LinkBase,
}

impl Config {
    /// Attemps to load a config file from the `file_path` provided
    fn from_file(file_path: String) -> Result<Config, IoError> {
        let mut config_file = File::open(file_path).expect("Config file not found");
        let mut contents: String = String::from("");

        config_file.read_to_string(&mut contents)?;
        let config: Config = toml::from_str(&contents)?;

        Ok(config)
    }
}

async fn get_valid_link(config: &Config, belvo_client: &BelvoClient) -> Option<LinkDetail> {
    let filters = LinkFilters {
        access_mode: Some(AccessMode::Single),
        status: Some(LinkStatus::Valid),
    };
    if let Ok(results) = link_api::list(&filters, &belvo_client).await {
        if results.count != 0 {
            return results.results.to_vec().pop();
        }
    }
    if let Ok(link) = link_api::register(&config.institution, &belvo_client).await {
        return Some(link);
    }

    return None;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let file_path = format!("./config.{}.toml", Environment::Development);
    let config = Config::from_file(file_path).unwrap();
    let belvo_client = BelvoClient::new(config.belvo.to_owned(), Environment::Development);
    let link: Option<LinkDetail> = get_valid_link(&config, &belvo_client).await;

    if let Some(l) = link {
        println!("{:?}", l);
        let accounts = account_api::list(&l.id, false, &belvo_client).await?;
    } else {
        println!("No link found or created");
    }

    Ok(())
}
