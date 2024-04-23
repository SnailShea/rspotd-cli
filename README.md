# rspotd-cli

Generate ARRIS/Commscope password-of-the-day for modems using [rspotd](https://crates.io/crates/rspotd) library

```
Usage: rspotd-cli [OPTIONS]

Options:
  -s, --seed <SEED>                String of 4-8 characters, used in password generation to mutate output
  -d, --date <DATE>                Generate a password for the given date
  -D, --des                        Output DES representation of seed
  -f, --format <FORMAT>            Password output format, either text or json [possible values: json, text]
  -F, --date-format <DATE_FORMAT>  Format the date string; see date(1) for valid format syntaxw
  -o, --output <OUTPUT>            Password or list will be written to given filename; existing file will be overwritten
  -r, --range <START> <END>        Generate a list of passwords given start and end dates
  -v, --verbose                    Print output to console when writing to file
  -h, --help                       Print help
  -V, --version                    Print version
```