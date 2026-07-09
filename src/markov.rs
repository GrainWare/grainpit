//! A generic [Markov chain](https://en.wikipedia.org/wiki/Markov_chain) for almost any type.
//! In particular, elements of the chain must be `Eq`, `Hash`, and `Clone`.
//!
//! # Examples
//!
//! ```
//! use markov::Chain;
//!
//! let mut chain = Chain::new();
//! chain.feed_str("I like cats and I like dogs.");
//! println!("{}", chain.generate_str());
//! ```
//!
//! ```
//! use markov::Chain;
//!
//! let mut chain = Chain::new();
//! chain.feed(vec![1u8, 2, 3, 5]).feed([3u8, 9, 2]);
//! println!("{:?}", chain.generate());
//! ```
#![warn(missing_docs)]

use std::borrow::ToOwned;
use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::fs::File;
use std::hash::Hash;
use std::io::prelude::*;
use std::io::{BufReader, Result};
use std::path::Path;

use rand::{RngExt, rng};

/// The definition of all types that can be used in a `Chain`.
pub trait Chainable: Eq + Hash + Clone {}
impl<T> Chainable for T where T: Eq + Hash + Clone {}

type Token<T> = Option<T>;

/// A generic [Markov chain](https://en.wikipedia.org/wiki/Markov_chain) for almost any type.
/// In particular, elements of the chain must be `Eq`, `Hash`, and `Clone`.
#[derive(Clone, PartialEq, Debug)]
pub struct Chain<T>
where
    T: Chainable,
{
    map: HashMap<Vec<Token<T>>, HashMap<Token<T>, usize>>,
    order: usize,
}

impl<T> Default for Chain<T>
where
    T: Chainable,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Chain<T>
where
    T: Chainable,
{
    /// Constructs a new Markov chain.
    pub fn new() -> Chain<T> {
        Self::of_order(1)
    }

    /// Creates a new Markov chain of the specified order. The order is the number of previous
    /// tokens to use for each mapping in the chain. Higher orders mean that the generated text
    /// will more closely resemble the training set. Increasing the order can yield more realistic
    /// output, but typically at the cost of requiring more training data.
    pub fn of_order(order: usize) -> Chain<T> {
        assert!(order != 0);
        Chain {
            map: {
                let mut map = HashMap::new();
                map.insert(vec![None; order], HashMap::new());
                map
            },
            order,
        }
    }

    /// Determines whether or not the chain is empty. A chain is considered empty if nothing has
    /// been fed into it.
    pub fn is_empty(&self) -> bool {
        self.map[&vec![None; self.order]].is_empty()
    }

    /// Feeds the chain a collection of tokens. This operation is `O(n)` where `n` is the number of
    /// tokens to be fed into the chain.
    pub fn feed<S: AsRef<[T]>>(&mut self, tokens: S) -> &mut Chain<T> {
        let tokens = tokens.as_ref();
        if tokens.is_empty() {
            return self;
        }
        let mut toks = vec![None; self.order];
        toks.extend(tokens.iter().map(|token| Some(token.clone())));
        toks.push(None);
        for p in toks.windows(self.order + 1) {
            self.map
                .entry(p[0..self.order].to_vec())
                .or_default();
            self.map
                .get_mut(&p[0..self.order])
                .unwrap()
                .add(p[self.order].clone(), 1);
        }
        self
    }

    /// Generates a collection of tokens from the chain. This operation is `O(mn)` where `m` is the
    /// length of the generated collection, and `n` is the number of possible states from a given
    /// state.
    pub fn generate(&self) -> Vec<T> {
        let mut ret = Vec::new();
        let mut curs = vec![None; self.order];
        loop {
            let next = self.map[&curs].next();
            curs = curs[1..self.order].to_vec();
            curs.push(next.clone());
            if let Some(next) = next {
                ret.push(next)
            };
            if curs[self.order - 1].is_none() {
                break;
            }
        }
        ret
    }

    /// Generates a collection of tokens from the chain, starting with the given token. This
    /// operation is O(mn) where m is the length of the generated collection, and n is the number
    /// of possible states from a given state. This returns an empty vector if the token is not
    /// found.
    pub fn generate_from_token(&self, token: T) -> Vec<T> {
        let mut curs = vec![None; self.order - 1];
        curs.push(Some(token.clone()));
        if !self.map.contains_key(&curs) {
            return Vec::new();
        }
        let mut ret = vec![token];
        loop {
            let next = self.map[&curs].next();
            curs = curs[1..self.order].to_vec();
            curs.push(next.clone());
            if let Some(next) = next {
                ret.push(next)
            };
            if curs[self.order - 1].is_none() {
                break;
            }
        }
        ret
    }

    /// Merges 2 chains (self and other) into self, consuming the other one. Both chains must be of
    /// the same order. This method is useful when you want to speed up chain building - chains
    /// built independently (e.g. in parallel with rayon) can be merged into a final one.
    pub fn merge(&mut self, other: Chain<T>) -> &Chain<T> {
        assert!(self.order == other.order);

        for (tokens, next) in other.map {
            let states = self.map.entry(tokens).or_default();

            for (token, count) in next {
                states.add(token, count);
            }
        }

        self
    }
}

impl Chain<String> {
    /// Feeds a string of text into the chain.
    pub fn feed_str(&mut self, string: &str) -> &mut Chain<String> {
        self.feed(string.split(' ').map(|s| s.to_owned()).collect::<Vec<_>>())
    }

    /// Feeds a properly formatted file into the chain. This file should be formatted such that
    /// each line is a new sentence. Punctuation may be included if it is desired.
    pub fn feed_file<P: AsRef<Path>>(&mut self, path: P) -> Result<&mut Chain<String>> {
        let reader = BufReader::new(File::open(path)?);
        for line in reader.lines() {
            let line = line?;
            let words = line
                .split_whitespace()
                .filter(|word| !word.is_empty())
                .map(|s| s.to_owned())
                .collect::<Vec<_>>();
            self.feed(&words);
        }
        Ok(self)
    }

    /// Converts the output of `generate(...)` on a String chain to a single String.
    fn vec_to_string(vec: Vec<String>) -> String {
        let mut ret = String::new();
        for s in &vec {
            ret.push_str(s);
            ret.push(' ');
        }
        let len = ret.len();
        if len > 0 {
            ret.truncate(len - 1);
        }
        ret
    }

    /// Generates a random string of text.
    pub fn generate_str(&self) -> String {
        Chain::vec_to_string(self.generate())
    }

    /// Generates a random string of text starting with the desired token. This returns an empty
    /// string if the token is not found.
    pub fn generate_str_from_token(&self, string: &str) -> String {
        Chain::vec_to_string(self.generate_from_token(string.to_owned()))
    }
}

/// A collection of states for the Markov chain.
trait States<T: PartialEq> {
    /// Adds a state to this states collection.
    fn add(&mut self, token: Token<T>, count: usize);
    /// Gets the next state from this collection of states.
    fn next(&self) -> Token<T>;
}

impl<T> States<T> for HashMap<Token<T>, usize>
where
    T: Chainable,
{
    fn add(&mut self, token: Token<T>, count: usize) {
        match self.entry(token) {
            Occupied(mut e) => *e.get_mut() += count,
            Vacant(e) => {
                e.insert(count);
            }
        }
    }

    fn next(&self) -> Token<T> {
        let mut sum = 0;
        for &value in self.values() {
            sum += value;
        }
        let mut rng = rng();
        let cap = rng.random_range(0..sum);
        sum = 0;
        for (key, &value) in self.iter() {
            sum += value;
            if sum > cap {
                return key.clone();
            }
        }
        unreachable!("The random number generator failed.")
    }
}

#[cfg(test)]
mod test {
    use super::Chain;

    #[test]
    fn new() {
        Chain::<u8>::new();
        Chain::<String>::new();
    }

    #[test]
    fn is_empty() {
        let mut chain = Chain::new();
        assert!(chain.is_empty());
        chain.feed(vec![1u8, 2, 3]);
        assert!(!chain.is_empty());
    }

    #[test]
    fn feed() {
        let mut chain = Chain::new();
        chain.feed(vec![3, 5, 10]).feed(vec![5, 12]);
    }

    #[test]
    fn generate() {
        let mut chain = Chain::new();
        chain.feed(vec![3u8, 5, 10]).feed(vec![5, 12]);
        let v = chain.generate();
        assert!([vec![3, 5, 10], vec![3, 5, 12], vec![5, 10], vec![5, 12]].contains(&v));
    }

    #[test]
    fn generate_for_higher_order() {
        let mut chain = Chain::of_order(2);
        chain.feed(vec![3u8, 5, 10]).feed(vec![2, 3, 5, 12]);
        let v = chain.generate();
        assert!(
            [
                vec![3, 5, 10],
                vec![3, 5, 12],
                vec![2, 3, 5, 10],
                vec![2, 3, 5, 12]
            ]
            .contains(&v)
        );
    }

    #[test]
    fn generate_from_token() {
        let mut chain = Chain::new();
        chain.feed(vec![3u8, 5, 10]).feed(vec![5, 12]);
        let v = chain.generate_from_token(5);
        assert!([vec![5, 10], vec![5, 12]].contains(&v));
    }

    #[test]
    fn generate_from_unfound_token() {
        let mut chain = Chain::new();
        chain.feed(vec![3u8, 5, 10]).feed(vec![5, 12]);
        let v: Vec<_> = chain.generate_from_token(9);
        assert!(v.is_empty());
    }

    #[test]
    fn feed_str() {
        let mut chain = Chain::new();
        chain.feed_str("I like cats and dogs");
    }

    #[test]
    fn generate_str() {
        let mut chain = Chain::new();
        chain.feed_str("I like cats").feed_str("I hate cats");
        assert!(["I like cats", "I hate cats"].contains(&&chain.generate_str()[..]));
    }

    #[test]
    fn generate_str_from_token() {
        let mut chain = Chain::new();
        chain.feed_str("I like cats").feed_str("cats are cute");
        assert!(["cats", "cats are cute"].contains(&&chain.generate_str_from_token("cats")[..]));
    }

    #[test]
    fn generate_str_from_token_higher_order() {
        let mut chain = Chain::of_order(2);
        chain.feed_str("I like cats").feed_str("cats are cute");
        println!("{:?}", chain.generate_str_from_token("cats"));
        assert!(["cats", "cats are cute"].contains(&&chain.generate_str_from_token("cats")[..]));
    }

    #[test]
    fn generate_str_from_unfound_token() {
        let mut chain = Chain::new();
        chain.feed_str("I like cats").feed_str("cats are cute");
        assert_eq!(chain.generate_str_from_token("test"), "");
    }

    #[test]
    fn merge() {
        let mut chain = Chain::of_order(2);
        chain.feed_str("I like cats and I like dogs");
        chain.feed_str("I like puzzles and I don't like dogs");
        chain.feed_str("I don't like puzzles and I like dogs");

        let mut new_chain = Chain::of_order(2);
        new_chain.feed_str("I like cats and I like dogs");

        let mut another_chain = Chain::of_order(2);
        another_chain.feed_str("I like puzzles and I don't like dogs");
        another_chain.feed_str("I don't like puzzles and I like dogs");

        new_chain.merge(another_chain);
        assert_eq!(chain, new_chain);
    }
}
