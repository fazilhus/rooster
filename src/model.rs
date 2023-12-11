use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub type TermFreq = HashMap::<String, usize>;
pub type TermFreqIndex = HashMap::<PathBuf, TermFreq>;

pub struct Lexer<'a> {
    content: &'a [char],
}

impl<'a> Lexer<'a> {
    pub fn new(content: &'a [char]) -> Self {
        Self { content }
    }

    fn trim_left(&mut self) -> &'a [char] {
        while !self.content.is_empty() && self.content[0].is_ascii_whitespace() {
            self.content = &self.content[1..];
        }

        self.content
    }

    fn strip_left(&mut self, n: usize) -> &'a [char] {
        let token = &self.content[0..n];
        self.content = &self.content[n..];
        token
    }

    fn strip_left_while<P>(&mut self, mut predicate: P) -> &'a [char] where P: FnMut(&char) -> bool {
        let mut i = 0;
        while i < self.content.len() && predicate(&self.content[i]) {
            i += 1;
        }
        return self.strip_left(i);
    }

    pub fn next_token(&mut self) -> Option<String> {
        self.trim_left();

        if self.content.is_empty() {
            return None;
        }

        if self.content[0].is_numeric() {
            return Some(self
                .strip_left_while(|e| e.is_numeric() || e.is_ascii_punctuation())
                .iter().collect());
        }

        if self.content[0].is_alphabetic() {
            return Some(self
                .strip_left_while(|&e| e.is_alphanumeric())
                .iter()
                .map(|e| e.to_ascii_uppercase())
                .collect());
        }

        return Some(self
            .strip_left(1)
            .iter().collect());
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}

pub fn tf(term: &str, map: &TermFreq) -> f32 {
    let a = map.get(term).cloned().unwrap_or(0) as f32;
    let b = map.iter().fold(0, |acc, (_, v)| acc + v) as f32;
    a / b
}

pub fn idf(term: &str, map: &TermFreqIndex) -> f32 {
    let n = map.len() as f32;
    let m = map.values().filter(|tf| tf.contains_key(term)).count().max(1) as f32;
    (n / m).log10()
}

pub fn search_query<'a>(query: &'a [char], tfi: &'a TermFreqIndex) -> Vec<(&'a Path, f32)> {
    let mut result = Vec::<(&Path, f32)>::new();
    let tokens = Lexer::new(&query).collect::<Vec<_>>();
    let cached_idf = tokens.iter().map(|t| idf(&t, &tfi)).collect::<Vec<f32>>();
    for (path, map) in tfi {
        let mut rank = 0f32;
        for (i, token) in tokens.iter().enumerate() {
            rank += tf(&token, &map) * cached_idf[i]
        }
        result.push((path, rank));
    }

    result.sort_by(|(_, rank1), (_, rank2)| rank2.total_cmp(rank1));
    result
}