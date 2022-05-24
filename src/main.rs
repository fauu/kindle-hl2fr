mod clippings;
mod io;
mod org;

use std::{collections::HashSet, env, fs::File, io::BufReader, process};

use itertools::Itertools;
use serde_json::json;
use strum::EnumString;

use io::LineReader;
use org::Display;

enum ProgramMode<'a> {
    PrintDocTitles,
    PrintClippings {
        doc_titles: Vec<&'a str>,
        format: ClippingPrintFormat,
    },
}

#[derive(EnumString)]
#[strum(serialize_all = "snake_case")]
enum ClippingPrintFormat {
    Json,
    Org,
}

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 && args.len() != 4 {
        println!("Usage: kindle-hl2fr <My Clippings file> {{org|json}} [Document titles]");
        process::exit(0);
    }

    let program_mode = match args.get(3) {
        Some(doc_titles_str) => ProgramMode::PrintClippings {
            doc_titles: doc_titles_str.lines().collect_vec().to_owned(),
            // CODE: This feels hacky
            format: match args.get(2).unwrap_or(&"_".to_string()).as_str() {
                "json" => ClippingPrintFormat::Json,
                _ => ClippingPrintFormat::Org,
            },
        },
        _ => ProgramMode::PrintDocTitles,
    };

    let clippings_path = &args[1];
    if let Ok(file) = File::open(clippings_path) {
        run(program_mode, &file);
    } else {
        eprintln!("Error opening clippings file: '{}'", clippings_path);
        process::exit(1);
    }
}

fn run(mode: ProgramMode, file: &File) {
    let mut doc_titles = HashSet::new();
    let mut entries = HashSet::new();

    let mut line_reader = LineReader::new(BufReader::new(file));
    loop {
        use clippings::parse::*;
        match entry(&mut line_reader) {
            Result::Eof => break,

            Result::Entry(entry) => match &mode {
                ProgramMode::PrintDocTitles => {
                    doc_titles.insert(entry.doc_title);
                }
                ProgramMode::PrintClippings { doc_titles, .. } => {
                    if doc_titles.contains(&entry.doc_title.as_str()) {
                        entries.insert(entry);
                    }
                }
            },

            // TODO: Verify that reported line number is correct
            Result::Err(err) => {
                eprintln!(
                    "Error parsing clippings entry at line {}: {}. Skipping.",
                    line_reader.current_line,
                    err.to_string()
                );
            }
        };
    }

    match mode {
        ProgramMode::PrintDocTitles => doc_titles
            .iter()
            .collect::<Vec<&String>>()
            .iter()
            .sorted()
            .for_each(|t| println!("{}", t)),

        ProgramMode::PrintClippings {
            doc_titles: _,
            format,
        } => {
            use clippings::ClippingKind;
            let entries_by_doc_title = entries
                .iter()
                .filter(|e| [ClippingKind::Highlight, ClippingKind::Note].contains(&e.kind))
                .sorted_by_key(|e| &e.date)
                .sorted_by_key(|e| &e.location_or_page)
                .map(|e| (&e.doc_title, e))
                .into_group_map();

            let out_str = match format {
                ClippingPrintFormat::Json => {
                    let json = entries_by_doc_title
                        .iter()
                        .map(|(doc_title, entries)| {
                            json!({
                            "documentTitle": doc_title,
                            "clippings": entries
                            })
                        })
                        .collect::<serde_json::Value>();
                    serde_json::to_string_pretty(&json).unwrap()
                }

                ClippingPrintFormat::Org => entries_by_doc_title
                    .iter()
                    .map(|(doc_title, entries)| {
                        // PERF: Should another way of building a string be used?
                        let header_str = format!("DOCUMENT: {}", doc_title);
                        let entries_str = entries
                            .iter()
                            .map(|e| e.to_org_text())
                            .collect::<Vec<String>>()
                            .join("\n");
                        format!("{}\n{}\n", header_str, entries_str)
                    })
                    .collect::<Vec<String>>()
                    .join("\n"),
            };

            println!("{}", out_str);
        }
    }
}
