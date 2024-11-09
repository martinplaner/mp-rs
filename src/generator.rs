use rand::{thread_rng, Rng};
use std::collections::HashMap;
use std::io::{BufRead, Error};

pub trait Generator {
    fn generate(&self, term: &str) -> Option<String>;
}

pub struct CompoundGenerator {
    dict: HashMap<char, Vec<String>>,
    delimiter: String,
}

impl CompoundGenerator {
    pub fn new(reader: impl BufRead, delimiter: &str) -> Result<CompoundGenerator, Error> {
        let mut dict: HashMap<char, Vec<String>> = HashMap::new();
        for line in reader.lines() {
            let line = line?;

            if line.starts_with('#') {
                continue;
            }

            if let Some(mut first_char) = line.chars().next() {
                first_char = first_char.to_uppercase().next().unwrap_or(first_char);
                dict.entry(first_char).or_default().push(line);
            }
        }

        Ok(CompoundGenerator {
            dict,
            delimiter: delimiter.to_string(),
        })
    }
}

impl Generator for CompoundGenerator {
    fn generate(&self, term: &str) -> Option<String> {
        let mut rng = thread_rng();

        term.to_uppercase()
            .chars()
            .map(|c| {
                self.dict.get(&c).and_then(|words| {
                    let i = rng.gen_range(0..words.len());
                    words.get(i).map(|s| s.to_owned())
                })
            })
            .collect::<Option<Vec<String>>>()
            .map(|words| words.join(&self.delimiter))
    }
}
