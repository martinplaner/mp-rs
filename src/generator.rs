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
            let line = line?.trim().to_owned();

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

        let words = term
            .to_uppercase()
            .chars()
            .filter_map(|c| {
                self.dict.get(&c).and_then(|words| {
                    let i = rng.gen_range(0..words.len());
                    words.get(i).map(|s| s.to_owned())
                })
            })
            .collect::<Vec<String>>();

        if words.is_empty() {
            None
        } else {
            Some(words.join(&self.delimiter))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::generator::{CompoundGenerator, Generator};
    use std::io::BufReader;

    #[test]
    fn generate_some() {
        let dict = "#comment\nAlpha\n# comment\n   Bravo".as_bytes();
        let generator = CompoundGenerator::new(dict, "-").unwrap();

        assert_eq!(generator.generate("ab"), Some("Alpha-Bravo".to_string()));
    }

    #[test]
    fn generate_none() {
        let reader = BufReader::new(std::io::empty());
        let generator = CompoundGenerator::new(reader, "-").unwrap();

        assert_eq!(generator.generate("ab"), None);
        assert_eq!(generator.generate(""), None);
    }
}
