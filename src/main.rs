#![allow(warnings)]

use chrono::{Local, NaiveDate};
use clap::builder::{PossibleValuesParser, Str};
use clap::Parser;
use rspotd::{generate, generate_multiple, seed_to_des};
use serde_json::to_string_pretty;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::str::FromStr;
use std::{path::Path, process::exit};
use std::writeln;

#[derive(Parser)]
#[clap(
    author = "Shea Zerda",
    version,
    about = "ARRIS/Commscope password-of-the-day generator"
)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(
        short = 's',
        long = "seed",
        help = "String of 4-8 characters, used in password generation to mutate output"
    )]
    seed: Option<String>,

    #[arg(
        short = 'd',
        long = "date",
        conflicts_with = "range",
        help = "Generate a password for the given date"
    )]
    date: Option<String>,

    #[arg(
        short = 'D',
        long = "des",
        conflicts_with = "date",
        conflicts_with = "range",
        num_args = 0,
        help = "Output DES representation of seed"
    )]
    des: bool,

    #[arg(
        short = 'f',
        long = "format",
        value_parser = PossibleValuesParser::new(["json", "text"]),
        help="Password output format, either text or json"
    )]
    format: Option<String>,

    #[arg(
        short = 'F',
        long = "date-format",
        help="Format the date string; see date(1) for valid format syntaxw"
    )]
    date_format: Option<String>,

    #[arg(
        short = 'o',
        long = "output",
        help = "Password or list will be written to given filename; existing file will be overwritten"
    )]
    output: Option<String>,

    #[arg(
        short = 'r',
        long = "range",
        conflicts_with = "date",
        num_args = 2,
        value_names = ["START", "END"],
        help="Generate a list of passwords given start and end dates"
    )]
    range: Option<Vec<String>>,

    #[arg(
        short = 'v',
        long = "verbose",
        help = "Print output to console when writing to file"
    )]
    verbose: bool,
}

fn current_date() -> String {
    Local::now().format("%Y-%m-%d").to_string()
}

fn format_potd(format: &str, date: &str, potd: &str) -> String {
    if format == "text" {
        format!("{}: \t{}", date, potd)
    } else {
        let mut potd_map: HashMap<String, String> = HashMap::new();
        potd_map.insert(date.to_string(), potd.to_string());
        let json = serde_json::to_string_pretty(&potd_map);
        if json.is_err() {
            println!("{}", json.unwrap_err());
            exit(1);
        } else {
            json.unwrap()
        }
    }
}

fn format_potd_range(date_format: Option<&str>, format: &str, potd_range: HashMap<String, String>) -> String {
    if format == "text" {
        let mut range: Vec<String> = Vec::new();
        for day in potd_range.to_owned().into_iter() {
            let date_val = format_date(date_format.as_deref(), &day.0).unwrap();
            let full_val = format!("{}:\t{}", date_val, &day.1);
            range.push(full_val);
        }
        range.join("\n")
    } else {
        let potd = to_string_pretty(&potd_range);
        if potd.is_err() {
            println!("{}", potd.unwrap_err());
            exit(1);
        } else {
            potd.unwrap()
        }
    }
}

fn format_date(date_format: Option<&str>, date: &str) -> Option<String> {
    if date_format.is_none() {
        return Some(date.to_string());
    }
    let naive_date = NaiveDate::from_str(date);
    if naive_date.is_err() {
        return None;
    }
    let formatted_date = NaiveDate::from_str(date).unwrap().format(date_format.unwrap());
    return Some(formatted_date.to_string());
}

fn main() {
    use rspotd::vals::DEFAULT_SEED;
    let args = Args::parse();
    let date;
    let begin;
    let end;
    let seed;
    let format;
    let date_format: Option<&str>;
    let mut potd: Option<String> = None;

    // determine output format
    if args.format.is_none() {
        format = "text";
    } else {
        format = args.format.as_ref().unwrap();
    }

    if args.date_format.is_none() {
        date_format = None;
    } else {
        date_format = Some(args.date_format.as_ref().unwrap().as_str());
    }

    // determine seed
    if args.seed.is_none() {
        seed = DEFAULT_SEED;
    } else {
        seed = args.seed.as_ref().unwrap().as_str();
    }

    if args.des {
        let des = seed_to_des(seed);
        if des.is_err() {
            println!("{}", des.unwrap_err());
            exit(1);
        }
        println!("{}", des.unwrap());
        exit(0)
    }

    // date not specified, range not specified, use today
    if args.date.is_none() && args.range.is_none() {
        date = current_date();
        let result = generate(&date, seed);
        if result.is_err() {
            println!("{}", result.unwrap_err());
            exit(1);            
        } else {
            potd = Some(format_potd(format, &date, result.as_ref().unwrap()));
        }
    } else if !args.date.is_none() {
        date = args.date.as_ref().unwrap().to_string();
        let result = generate(date.as_ref(), seed);
        if result.is_err() {
            println!("{}", result.unwrap_err());
            exit(1);
        } else {
            potd = Some(format_potd(format, &date, result.as_ref().unwrap()));
        }
    } else if !args.range.is_none() {
        let range = args.range.as_ref().unwrap();
        begin = &range[0];
        end = &range[1];
        let result = generate_multiple(begin, end, seed);
        if result.is_err() {
            println!("{}", result.unwrap_err());
            exit(1);            
        } else {
            let _potd = format_potd_range(date_format.as_deref(), format, result.unwrap());
            potd = Some(_potd);
        }
    }

    // determine output file, if any
    if args.output.is_none() {
        println!("{}", potd.as_ref().unwrap());
    } else {
        if args.verbose {
            println!("{}", potd.as_mut().unwrap());
        }
        let user_input = args.output.unwrap();
        let path = Path::new(".").join(user_input.to_string());
        let mut file = OpenOptions::new()
            .append(false)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path);
        if file.is_err() {
            println!("Unable to create file '{}', likely due to issue with permissions.", path.display());
            exit(1);
        }
        let mut writer = BufWriter::new(file.as_mut().unwrap());
        // use format here
        if !potd.is_none() {
            writer.write_all(potd.unwrap().as_bytes());
            writer.write_all("\n".as_bytes());
        }
    }
}
