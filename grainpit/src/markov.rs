use rand::seq::IteratorRandom;

use crate::markov::chain::Chain;
use regex::regex;

pub mod chain;

#[derive(Debug, Clone)]
pub struct Markov {
    pub html_chain: Chain,
    pub url_name_chain: Chain,
    pub url_chain: Chain,
    pub config_chain: Chain,
}

impl Markov {
    pub fn new() -> Self {
        Self {
            html_chain: Chain::new(include_str!("../data/html.txt")),
            url_name_chain: Chain::new(include_str!("../data/url_name.txt")),
            url_chain: Chain::new(include_str!("../data/url.txt")),
            config_chain: Chain::new(include_str!("../data/config.txt")),
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
        let start_path = if std::env::var("GRAINPIT_EXTRAURLS").is_ok() {
            if rand::random_bool(
                std::env::var("GRAINPIT_EXTRAURLS_CHANCE")
                    .unwrap_or("5".to_owned())
                    .parse::<u8>()
                    .unwrap() as f64
                    / 100.0,
            ) {
                let rng = &mut rand::rng();
                std::env::var("GRAINPIT_EXTRAURLS")
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

impl Default for Markov {
    fn default() -> Self {
        Markov::new()
    }
}
