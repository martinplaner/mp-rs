# mp-rs

[![License](https://img.shields.io/badge/license-Apache%202.0-blue?style=flat-square)](LICENSE-APACHE)
[![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](LICENSE-MIT)

Highly sophisticated [initialism](https://en.wikipedia.org/wiki/Acronym#Nomenclature) to
hyphenated [compound words](https://en.wikipedia.org/wiki/Compound_(linguistics)) generator.

This is a [Rust](https://www.rust-lang.org/) rewrite of https://github.com/martinplaner/mp.

## Example

*MP* -> *Melanchton-Paralogismus*

## Disclaimer

All this effort for a stupid inside joke. Don't worry if you don't get it, it's not that funny anyway...

## Usage

```
Usage: mp-rs.exe [OPTIONS]

Options:
  -f, --file <FILE>                    Path to word list (one word per line) [default: words.txt]
  -l, --listen <LISTEN>                TCP address for the server to listen on, in the form 'host:port' [default: 0.0.0.0:8080]
  -d, --default-query <DEFAULT_QUERY>  Default fallback query term, if not provided [default: MP]
  -o, --once <ONCE>
  -h, --help                           Print help
  -V, --version                        Print version
```

## Licenses

### Source code

Dual-licensed under [Apache 2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT).

### Word list (words_de.txt)

Korpusbasierte Wortgrundformenliste DEREWO, v-ww-bll-320000g-2012-12-31-1.0, mit
Benutzerdokumentation, http://www.ids-mannheim.de/derewo, © Institut für Deutsche Sprache, Programmbereich
Korpuslinguistik, Mannheim, Deutschland, 2013.

### Twemoji (Favicon)

Copyright 2019 Twitter, Inc and other contributors.

Code licensed under the MIT License: <http://opensource.org/licenses/MIT>

Graphics licensed under CC-BY 4.0: <https://creativecommons.org/licenses/by/4.0/>
