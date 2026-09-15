use rand::seq::{IndexedRandom, IteratorRandom};
use tracing::info;

use crate::markov::chain::Chain;
use regex::regex;

pub mod chain;

#[derive(Debug, Clone)]
pub struct Markov {
    pub html_chain: Chain,
    pub url_name_chain: Chain,
    pub url_chain: Chain,
    pub config_chain: Chain,
    extraurls: Option<Vec<String>>,
    extraurls_chance: f64,
}

impl Markov {
    pub fn new(extraurls: Option<Vec<String>>, extraurls_chance: f64) -> Self {
        if let Some(extraurls) = &extraurls {
            info!("loaded {} extraurls", extraurls.len());
        }
        Self {
            html_chain: Chain::new(include_str!("../data/html.txt")),
            url_name_chain: Chain::new(include_str!("../data/url_name.txt")),
            url_chain: Chain::new(include_str!("../data/url.txt")),
            config_chain: Chain::new(include_str!("../data/config.txt")),
            extraurls,
            extraurls_chance,
        }
    }

    pub fn gen_html(&self) -> String {
        let href_regex = regex!(r#"href="(.*?)""#);
        let mut generated = self.html_chain.generate(2048);
        for i in href_regex.find_iter(&generated.clone()) {
            generated = generated.replace(i.as_str(), &self.random_link(false));
        }

        let text_content_regex = regex!(r">([^<]+)<");
        let url_replacements: Vec<String> = text_content_regex
            .captures_iter(&generated)
            .flat_map(|caps| caps.get(0).unwrap().as_str().split_whitespace())
            .sample(&mut rand::rng(), 15)
            .into_iter()
            .map(|v| v.to_string())
            .collect();

        for r in url_replacements {
            if rand::random_bool(0.9) {
                generated = generated.replacen(
                    &r,
                    &format!("<a href='{}'>{}</a>", self.random_link(false), r),
                    1,
                );
            } else {
                generated = generated.replacen(
                    &r,
                    &format!("<img src='{}'>{}</img>", self.random_link(true), r),
                    1,
                );
            }
        }

        format!("<p>{}</p>", generated)
    }

    pub fn random_link(&self, image: bool) -> String {
        let start_path = if let Some(extraurls) = &self.extraurls {
            if rand::random_bool(self.extraurls_chance) {
                let rng = &mut rand::rng();
                extraurls
                    .choose(rng)
                    .unwrap()
                    .split(',')
                    .choose(rng)
                    .unwrap()
                    .to_owned()
            } else {
                "/".to_string()
            }
        } else {
            "/".to_string()
        };

        format!(
            "{}{}/{}/{}{}",
            start_path,
            self.url_chain.generate(4),
            self.url_chain.generate(4),
            self.url_chain.generate(24),
            if rand::random_bool(0.95) {
                if image { ".jpg" } else { ".html" }
            } else {
                if image { ".png" } else { "" }
            }
        )
    }
}
