use std::iter;

use rand::seq::IndexedRandom;
use rustc_hash::FxHashMap;

#[derive(Clone, PartialEq, Debug, Default)]
pub struct Chain {
    tokens: FxHashMap<Vec<u32>, FxHashMap<u32, usize>>,
    vocab: FxHashMap<String, u32>,
    vocab_rev: Vec<String>,
}

impl Chain {
    const MAX_CONTEXT_SIZE: usize = 5;

    fn intern(&mut self, s: &str) -> u32 {
        if let Some(&id) = self.vocab.get(s) {
            return id;
        }
        let id = self.vocab_rev.len() as u32;
        self.vocab_rev.push(s.to_string());
        self.vocab.insert(s.to_string(), id);
        id
    }

    pub fn train(&mut self, text: &str) {
        let mut context: Vec<u32> = Vec::with_capacity(Self::MAX_CONTEXT_SIZE + 1);

        for token in Self::tokenize(text) {
            let token_id = self.intern(token);

            for cs in 0..=context.len() {
                let context_key: Vec<u32> = context[(context.len() - cs)..context.len()].to_vec();

                *self
                    .tokens
                    .entry(context_key)
                    .or_default()
                    .entry(token_id)
                    .or_default() += 1;
            }

            context.push(token_id);

            if context.len() > Self::MAX_CONTEXT_SIZE {
                context.remove(0);
            }
        }
    }

    pub fn generate(&self, length: usize) -> String {
        let mut out: Vec<u32> = Vec::with_capacity(length);
        let mut rng = rand::rng();

        while out.len() < length {
            let mut next_token = None;

            for cs in (0..=Self::MAX_CONTEXT_SIZE).rev() {
                if cs > out.len() {
                    continue;
                }

                let context: &[u32] = &out[(out.len() - cs)..out.len()];

                if let Some(next_tokens) = self.tokens.get(context) {
                    let next_tokens: Vec<_> = next_tokens.iter().collect();

                    next_token = Some(
                        *next_tokens
                            .choose_weighted(&mut rng, |(_token, frequency)| *frequency)
                            .unwrap()
                            .0,
                    );

                    break;
                }
            }

            if let Some(next_token) = next_token {
                out.push(next_token);
            } else {
                break;
            }
        }

        out.iter()
            .map(|id| self.vocab_rev[*id as usize].as_str())
            .collect()
    }

    fn tokenize(s: &str) -> impl Iterator<Item = &str> {
        let mut chars = s.char_indices().peekable();

        iter::from_fn(move || {
            let (idx, ch) = chars.next()?;

            if ch.is_alphanumeric() {
                let idx_from = idx;
                let mut idx_to = idx + ch.len_utf8();

                while let Some((idx, ch)) = chars.peek() {
                    if ch.is_alphanumeric() {
                        idx_to = idx + ch.len_utf8();
                        chars.next();
                    } else {
                        break;
                    }
                }

                Some(&s[idx_from..idx_to])
            } else if ch.is_whitespace() {
                let idx_from = idx;
                let mut idx_to = idx + ch.len_utf8();

                while let Some((idx, ch)) = chars.peek() {
                    if ch.is_whitespace() {
                        idx_to = idx + ch.len_utf8();
                        chars.next();
                    } else {
                        break;
                    }
                }

                Some(&s[idx_from..idx_to])
            } else if ch == '<' {
                let idx_from = idx;
                let mut idx_to = idx + ch.len_utf8();

                while let Some(&(idx, ch)) = chars.peek() {
                    idx_to = idx + ch.len_utf8();
                    chars.next();
                    if ch == '>' {
                        break;
                    }
                }

                Some(&s[idx_from..idx_to])
            } else {
                let idx_from = idx;
                let idx_to = idx + ch.len_utf8();

                Some(&s[idx_from..idx_to])
            }
        })
    }
}
