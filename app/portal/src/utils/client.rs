use std::sync::OnceLock;

use reqwest::Client;

static CLIENT: OnceLock<Client> = OnceLock::new();

pub fn client() -> &'static Client {
	CLIENT.get_or_init(|| Client::builder().timeout(std::time::Duration::from_secs(10)).build().unwrap())
}
