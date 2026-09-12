use arrayvec::ArrayVec;
use rand::{RngExt, SeedableRng};
use rand_xoshiro::Xoshiro256Plus;
use rustc_hash::FxHashMap;

const MAX_CONTEXT_SIZE: usize = 3;
type Tokens = FxHashMap<ArrayVec<u32, MAX_CONTEXT_SIZE>, (Vec<(u32, usize)>, usize)>;

#[derive(Clone, PartialEq, Debug, Default)]
pub struct Chain {
    tokens: Tokens,
    tokenizer: Vec<String>,
}

impl Chain {
    #[tracing::instrument(skip_all)]
    pub fn new(text: &str) -> Self {
        // this whole thing is pretty ass and can probably be optimized but its only ran at start so as long as its not like 100ms+ wtv
        let mut chain = Self::default();
        let mut context: Vec<u32> = Vec::with_capacity(MAX_CONTEXT_SIZE + 1);
        let (tokenizer, tokens) = Self::make_tokenizer(text);
        chain.tokenizer = tokenizer;
        let mut temp_tokens: FxHashMap<ArrayVec<u32, MAX_CONTEXT_SIZE>, FxHashMap<u32, usize>> =
            FxHashMap::default();
        for token in tokens {
            for cs in 0..=context.len() {
                let context_key: ArrayVec<u32, MAX_CONTEXT_SIZE> = context
                    [(context.len() - cs)..context.len()]
                    .try_into()
                    .unwrap();

                *temp_tokens
                    .entry(context_key)
                    .or_default()
                    .entry(token)
                    .or_default() += 1;
            }

            context.push(token);

            if context.len() > MAX_CONTEXT_SIZE {
                context.remove(0);
            }
        }

        chain.tokens = temp_tokens
            .into_iter()
            .map(|(k, v)| {
                (
                    k,
                    (
                        v.clone().into_iter().collect(),
                        v.clone().into_values().sum(),
                    ),
                )
            })
            .collect();

        chain.tokens.values_mut().for_each(|v| {
            v.0.sort_by_key(|k| k.1);
            v.0.reverse();
            v.0.shrink_to_fit();
        });

        chain
    }

    pub fn generate(&self, length: usize) -> String {
        let mut out: Vec<u32> = Vec::with_capacity(length);
        let mut rng = Xoshiro256Plus::from_rng(&mut rand::rng());

        while out.len() < length {
            let mut next_token = None;

            for cs in (0..=MAX_CONTEXT_SIZE).rev() {
                if cs > out.len() {
                    continue;
                }

                let context: ArrayVec<u32, MAX_CONTEXT_SIZE> =
                    out[(out.len() - cs)..out.len()].try_into().unwrap();

                if let Some(next_tokens) = self.tokens.get(&context) {
                    let mut remaining_distance = rng.random_range::<usize, _>(..next_tokens.1);
                    for (t, c) in &next_tokens.0 {
                        remaining_distance = remaining_distance.saturating_sub(*c);
                        if remaining_distance == 0 {
                            next_token = Some(*t);
                            break;
                        }
                    }
                    break;
                }
            }

            if let Some(next_token) = next_token {
                out.push(next_token);
            } else {
                next_token = Some(0);
                out.push(next_token.unwrap());
            }
        }

        let mut result = String::with_capacity(out.len() * 64); // average word length of largest chain is 44, larger just to be safe
        for &x in &out {
            result.push_str(self.tokenizer.get(x as usize).unwrap());
        }
        result
    }

    fn make_tokenizer(s: &str) -> (Vec<String>, Vec<u32>) {
        // TODO: replace this with nom or something that isnt this complete unreadable mess
        let len = s.len();
        let bytes = s.as_bytes();
        let mut pos = 0;
        let mut tokens: Vec<&str> = Vec::new();

        while pos < len {
            let start = pos;
            let ch = s[pos..].chars().next().unwrap();

            if ch.is_whitespace() {
                pos += ch.len_utf8();
                while pos < len {
                    let c = s[pos..].chars().next().unwrap();
                    if c.is_whitespace() {
                        pos += c.len_utf8();
                    } else {
                        break;
                    }
                }
                tokens.push(&s[start..pos]);
                continue;
            }

            if ch == '<' && bytes.get(pos + 1) == Some(&b'?') {
                pos += 2;
                tokens.push(&s[start..pos]);
                continue;
            }
            if ch == '?' && bytes.get(pos + 1) == Some(&b'>') {
                pos += 2;
                tokens.push(&s[start..pos]);
                continue;
            }

            if (ch == '<' && s[start..].starts_with("<style"))
                || (ch == '<' && s[start..].starts_with("<script"))
            {
                pos += 1;
                while pos < len {
                    let c = s[pos..].chars().next().unwrap();
                    pos += c.len_utf8();
                    if c == '>' {
                        break;
                    }
                }
                while pos < len {
                    if s[pos..].starts_with("</style>") || s[pos..].starts_with("</script>") {
                        pos += if s[pos..].starts_with("</style>") {
                            "</style>".len()
                        } else {
                            "</script>".len()
                        };
                        break;
                    }
                    let c = s[pos..].chars().next().unwrap();
                    pos += c.len_utf8();
                }
                tokens.push(&s[start..pos]);
                continue;
            }

            if ch == '<' {
                pos += ch.len_utf8();
                while pos < len {
                    let c = s[pos..].chars().next().unwrap();
                    pos += c.len_utf8();
                    if c == '>' {
                        break;
                    }
                }
                tokens.push(&s[start..pos]);
                continue;
            }

            if ch == '\'' || ch == '"' {
                pos += ch.len_utf8();
                while pos < len {
                    let c = s[pos..].chars().next().unwrap();
                    if c == '\n' {
                        break;
                    }
                    pos += c.len_utf8();
                    if c == ch {
                        break;
                    }
                }
                tokens.push(&s[start..pos]);
                continue;
            }

            if ch == '/' && bytes.get(pos + 1) == Some(&b'/') {
                while pos < len {
                    let c = s[pos..].chars().next().unwrap();
                    if c == '\n' {
                        break;
                    }
                    pos += c.len_utf8();
                }
                tokens.push(&s[start..pos]);
                continue;
            }

            if ch == '$' {
                pos += ch.len_utf8();
                while pos < len {
                    let c = s[pos..].chars().next().unwrap();
                    if c.is_alphanumeric() || c == '_' {
                        pos += c.len_utf8();
                    } else {
                        break;
                    }
                }
                tokens.push(&s[start..pos]);
                continue;
            }

            if ch == '/' {
                pos += ch.len_utf8();
                while pos < len {
                    let c = s[pos..].chars().next().unwrap();
                    if c.is_alphanumeric() || matches!(c, '/' | '.' | '-' | '_') {
                        pos += c.len_utf8();
                    } else {
                        break;
                    }
                }
                tokens.push(&s[start..pos]);
                continue;
            }

            if ch.is_alphanumeric() || matches!(ch, '#' | '%') {
                pos += ch.len_utf8();
                while pos < len {
                    let c = s[pos..].chars().next().unwrap();
                    if c.is_alphanumeric() || matches!(c, '.' | '-' | '#' | '%') {
                        pos += c.len_utf8();
                    } else {
                        break;
                    }
                }
                tokens.push(&s[start..pos]);
                continue;
            }

            pos += ch.len_utf8();
            tokens.push(&s[start..pos]);
        }

        let mut vocab: FxHashMap<&str, u32> = FxHashMap::default();
        let mut id_to_str: Vec<String> = Vec::default();

        let token_ids: Vec<u32> = tokens
            .iter()
            .map(|&t| {
                *vocab.entry(t).or_insert_with(|| {
                    let id = id_to_str.len() as u32;
                    id_to_str.push(t.to_string());
                    id
                })
            })
            .collect();

        (id_to_str, token_ids)
    }
}
