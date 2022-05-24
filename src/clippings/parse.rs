use std::{io, result::Result as StdResult, str::FromStr};

use chrono::NaiveDateTime;
use custom_error::custom_error;
use lazy_static::lazy_static;
use regex::Regex;

use super::{Clipping, ClippingKind, ClippingLocationOrPage};

use crate::io::LineReader;

pub enum Result {
    Entry(Clipping),
    Eof,
    Err(Error),
}

custom_error! { pub Error
    DateCapture                     = "capturing entry date",
    EndedPrematurely                = "file ended prematurely",
    InfoLineMatch{doc_title:String} = "matching info line (for document \"{doc_title}\")",
    KindCapture                     = "capturing entry kind",
    Other                           = "other error",
}

lazy_static! {
    static ref ENTRY_INFO_RE: Regex =
        Regex::new(r"- Your (?P<kind>\w+).+?(?:location|page) (?P<start>\w+)(?:-(?P<end>\w+))? \| Added on (?P<date>.*)")
            .unwrap();
}

const ENTRY_SEP: &str = "==========\r\n";

pub fn entry<R: io::Read>(mut line_reader: &mut LineReader<R>) -> Result {
    let doc_title = match read_entry_line(&mut line_reader) {
        Some(doc_title) => doc_title,
        _ => return Result::Eof,
    };

    let info_line = match read_entry_line(&mut line_reader) {
        Some(info_line) => info_line,
        // IMPROVEMENT: This happens when the title is multiline. Handle that case.
        _ => {
            move_to_next_entry(&mut line_reader);
            return Result::Err(Error::Other);
        }
    };

    // TODO: Skip "Clip This Article" entries without returning an error
    let info_line_captures = match ENTRY_INFO_RE.captures(&info_line) {
        Some(info_line_captures) => info_line_captures,
        _ => {
            move_to_next_entry(&mut line_reader);
            return Result::Err(Error::InfoLineMatch { doc_title });
        }
    };

    let kind = match info_line_captures.name("kind") {
        Some(kind_capture) => ClippingKind::from_str(kind_capture.as_str()),
        _ => {
            move_to_next_entry(&mut line_reader);
            return Result::Err(Error::KindCapture);
        }
    }
    .unwrap();

    let location_or_page = clipping_location_or_page(&info_line_captures);

    let date = match info_line_captures.name("date") {
        Some(date_capture) => {
            match NaiveDateTime::parse_from_str(date_capture.as_str(), "%A, %e %B %Y %H:%M:%S") {
                Ok(date) => date,
                _ => {
                    return Result::Err(Error::DateCapture);
                }
            }
        }
        _ => return Result::Err(Error::DateCapture),
    };

    let content = match clipping_content(&mut line_reader) {
        Ok(content) => content,
        Err(err) => return Result::Err(err),
    };

    Result::Entry(Clipping {
        doc_title,
        kind,
        location_or_page,
        date,
        content,
    })
}

fn read_entry_line<R: io::Read>(reader: &mut LineReader<R>) -> Option<String> {
    reader.buf.clear();
    let bytes_read = reader.read_line_to_buf().unwrap();
    if bytes_read == 0 {
        return None;
    }
    trim_newline(&mut reader.buf);
    // NOTE: Apparently titles can start with the BOM for whatever reason. Skip it.
    let start_idx = if reader.buf.bytes().next().unwrap() == 0xEF {
        3
    } else {
        0
    };
    Some(reader.buf[start_idx..].trim().to_string())
}

fn clipping_location_or_page(info_line_captures: &regex::Captures) -> ClippingLocationOrPage {
    let start_location_or_page = info_line_captures
        .name("start")
        .unwrap()
        .as_str()
        .parse::<i32>()
        .unwrap_or(0); // FIXME: Handle roman page numbers

    match info_line_captures.name("end") {
        Some(end_location_or_page_capture) => {
            let end_location_or_page = end_location_or_page_capture
                .as_str()
                .parse::<i32>()
                .unwrap();
            if end_location_or_page == start_location_or_page {
                ClippingLocationOrPage::Singular(start_location_or_page)
            } else {
                ClippingLocationOrPage::Ranged {
                    start: start_location_or_page,
                    end: end_location_or_page,
                }
            }
        }
        _ => ClippingLocationOrPage::Singular(start_location_or_page),
    }
}

fn clipping_content<R: io::Read>(reader: &mut LineReader<R>) -> StdResult<String, Error> {
    let mut content = String::new();
    let mut bytes_read;
    loop {
        reader.buf.clear();
        bytes_read = reader.read_line_to_buf().unwrap();
        if bytes_read == 0 {
            return Err(Error::EndedPrematurely);
        }
        match reader.buf.as_str() {
            "\r\n" => continue,
            ENTRY_SEP => break,
            _ => {
                content.push_str(&reader.buf);
            }
        }
    }
    trim_newline(&mut content);
    Ok(content)
}

fn move_to_next_entry<R: io::Read>(line_reader: &mut LineReader<R>) {
    loop {
        line_reader.buf.clear();
        let _ = line_reader.read_line_to_buf();
        match line_reader.buf.as_str() {
            "" | ENTRY_SEP => return,
            _ => (),
        }
    }
}

fn trim_newline(s: &mut String) {
    if s.ends_with('\n') {
        s.pop();
        if s.ends_with('\r') {
            s.pop();
        }
    }
}
