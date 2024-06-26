#![allow(warnings)]

use chrono::{
    format::{DelayedFormat, StrftimeItems},
    Local, NaiveDate, ParseError,
};
use clap::{
    builder::{PossibleValuesParser, Str},
    Parser,
};
use rspotd::{generate, generate_multiple, seed_to_des};
use serde_json::to_string_pretty;
use std::{
    borrow::{Borrow, BorrowMut}, collections::{BTreeMap, HashMap}, error::Error, fs::{File, OpenOptions}, io::{BufWriter, Write}, path::{Path, PathBuf}, process::exit, str::FromStr, writeln
};

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
        help = "Password output format, either text or json"
    )]
    format: Option<String>,

    #[arg(
        short = 'F',
        long = "date-format",
        help = "Format the date string; see date(1) for valid format syntax"
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

fn format_potd(date_format: &str, format: &str, date: &str, potd: &str) -> String {
    if format == "text" {
        format!("{}: \t{}", date, potd)
    } else {
        let mut potd_map: HashMap<String, String> = HashMap::new();
        let formatted_date = format_date(date_format, date);
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

fn format_potd_range(
    date_format: &str,
    format: &str,
    potd_range: BTreeMap<String, String>,
) -> String {
    let mut range: Vec<String> = Vec::new();
    for day in &potd_range {
        let date_val = format_date(date_format, &day.0);
        let potd_val = &day.1;
        let full_val = format!("{}: {}", date_val, &potd_val);
        range.push(full_val);
    }
    if format == "text" {
        range.join("\n")
    } else {
        let potd = to_string_pretty(&range);
        if potd.is_err() {
            println!("{}", potd.unwrap_err());
            exit(1);
        } else {
            potd.unwrap()
        }
    }
}

fn format_date(date_format: &str, date: &str) -> String {
    use std::fmt::Write;
    let split: Vec<i32>= date.split("-").map(|part| part.parse::<i32>().unwrap()).collect();
    let naive_date: Option<NaiveDate> = NaiveDate::from_ymd_opt(split[0] as i32, split[1] as u32, split[2] as u32);
    if naive_date.is_some() {
        let formatted_date = naive_date.unwrap().format(date_format).to_string();
        return formatted_date;
    } else {
        println!("Unable to parse date '{}'. Year, month or day value out of range.", &date);
        exit(1);
    }

}

fn unwrap_date_result(result: Result<String, Box<dyn Error>>) -> String {
    if result.is_err() {
        println!("{}", result.unwrap_err());
        exit(1);
    } else {
        result.unwrap()
    }
}

fn unwrap_range_result(
    result: Result<BTreeMap<String, String>, Box<dyn Error>>
) -> BTreeMap<String, String> {
    if result.is_err() {
        println!("{}", result.unwrap_err());
        exit(1);
    } else {
        result.unwrap()
    }
}

fn write_to_file(potd: &str, path: &Path) {
    let mut file = OpenOptions::new()
        .append(false)
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path);
    if file.is_err() {
        println!(
            "Unable to create file '{}', likely due to issue with permissions.",
            path.display()
        );
        exit(1);
    }
    let mut writer = BufWriter::new(file.as_mut().unwrap());
    writer.write_all(potd.as_bytes());
    writer.write_all("\n".as_bytes());
}

fn main() {
    use rspotd::vals::DEFAULT_SEED;
    let args = Args::parse();

    // determine output format
    let format: String;
    if args.format.is_none() {
        format = String::from("text");
    } else {
        format = args.format.unwrap();
    }

    let date_format: String;
    if args.date_format.is_none() {
        date_format = String::from("%Y-%m-%d");
    } else {
        date_format = args.date_format.unwrap().to_string();
    }

    // determine seed
    let seed: String;
    if args.seed.is_none() {
        seed = DEFAULT_SEED.to_string();
    } else {
        seed = args.seed.unwrap();
    }

    if args.des {
        let des = seed_to_des(&seed);
        if des.is_err() {
            println!("{}", des.unwrap_err());
            exit(1);
        }
        println!("{}", des.unwrap());
        exit(0)
    }

    // determine whether date or range and set potd value
    let potd;
    if args.date.is_none() && args.range.is_none() {
        let date = current_date();
        let formatted_date = format_date(&date_format, &date);
        let date_result = unwrap_date_result(generate(&date, &seed));
        potd = format_potd(&date_format, &format, &formatted_date, &date_result);
    } else if !args.date.is_none() {
        let date = args.date.as_ref().unwrap().to_string();
        let formatted_date = format_date(&date_format, &date);
        let date_result = unwrap_date_result(generate(&date, &seed));
        potd = format_potd(&date_format, &format, &formatted_date, &date_result);
    } else if !args.range.is_none() {
        let range = args.range.unwrap();
        let begin = &range[0];
        let end = &range[1];
        let _range_result = unwrap_range_result(generate_multiple(begin, end, &seed));
        potd = format_potd_range(&date_format, &format, _range_result);
    } else {
        // empty string initialization to keep the compiler happy
        // and give us something to reference later for a potd value
        potd = String::from("");
    }

    // determine output file, if any
    if args.output.is_none() {
        println!("{}", potd);
    } else {
        if args.verbose {
            println!("{}", potd);
        }
        let user_input = args.output.unwrap();
        let path = Path::new(".").join(user_input.to_string());
        write_to_file(&potd, &path);
    }
}
