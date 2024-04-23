#![allow(warnings)]

use chrono::Local;
use clap::builder::{PossibleValuesParser, Str};
use clap::Parser;
use rspotd::{generate, generate_multiple, seed_to_des};
use serde_json::to_string_pretty;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
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
        short = 'o',
        long = "output",
        help = "Password or list will be written to given filename"
    )]
    output: Option<String>,

    #[arg(
        short = 'r',
        long = "range",
        conflicts_with = "date",
        num_args = 2, value_names = ["START", "END"],
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
        let mut potd_vec: Vec<String> = Vec::new();
        potd_vec.push(date.to_string());
        potd_vec.push(potd.to_string());
        serde_json::to_string_pretty(&potd_vec).unwrap()
    }
}

fn format_potd_range(format: &str, potd_range: HashMap<String, String>) -> String {
    if format == "text" {
        // iterate
        let formatted_range: String = potd_range.iter().map(|(x, y)| {
            format!("{x}:\t{y}").to_string()
        }).collect();
        println!("{}", &formatted_range);
        formatted_range
    } else {
        to_string_pretty(&potd_range).unwrap()
    }
}

fn main() {
    use rspotd::vals::DEFAULT_SEED;
    let args = Args::parse();
    let date;
    let begin;
    let end;
    let seed;
    let format;
    let mut potd: Option<String> = None;
    let mut file;

    // determine output format
    if args.format.is_none() {
        format = "text";
    } else {
        format = args.format.as_ref().unwrap();
    }

    // determine seed
    if args.seed.is_none() {
        seed = DEFAULT_SEED;
    } else {
        seed = args.seed.as_ref().unwrap().as_str();
    }

    // date not specified, range not specified, use today
    if args.date.is_none() && args.range.is_none() {
        date = current_date();
        let result = generate(&date, seed);
        if result.is_err() {
            println!("{}", result.unwrap_err());
            exit(1);            
        } else {
            potd = Some(result.as_ref().unwrap().to_string());
        }
    } else if !args.date.is_none() {
        date = args.date.as_ref().unwrap().to_string();
        let result = generate(date.as_ref(), seed);
        if result.is_err() {
            println!("{}", result.unwrap_err());
            exit(1);
        } else {
            potd = Some(result.as_ref().unwrap().to_string());
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
            let _potd = serde_json::to_string_pretty(result.as_ref().unwrap());
            if _potd.is_err() {
                println!("{}", _potd.as_ref().unwrap_err());
                exit(1)                
            } else {
                potd = Some(_potd.unwrap());
            }
        }
    }

    // determine output file, if any
    if args.output.is_none() {
        println!("{}", potd.unwrap());
    } else {
        if args.verbose {
            println!("{}", potd.as_ref().unwrap());
        }
        let user_input = args.output.unwrap();
        let path = Path::new(".").join(user_input.to_string());
        file = OpenOptions::new()
            .write(true)
            .append(false)
            .open(&path);
        // file does not already exist, try to create it
        if file.is_err() {
            file = OpenOptions::new()
                .write(true)
                .append(false)
                .create_new(true)
                .open(&path);
            // if this fails, most likely permission denied, nothing we can do
            if file.is_err() {
                println!("Unable to create file '{}', likely due to issue with permissions.", path.display());
                exit(1);
            }
        }
        let mut writer = BufWriter::new(file.as_mut().unwrap());
        // use format here
        writer.write_all(potd.unwrap().as_bytes());
        writer.write_all("\n".as_bytes());
    }

    // TODO:
    // - implement format
    // - output to file
    // - verbose (print even when output to file)
    // - add date formatting
    //   - default format
}
