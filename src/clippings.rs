pub mod parse;

use chrono::NaiveDateTime;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

use serde::{ser::SerializeTuple, Serialize};
use strum::{AsRefStr, EnumString};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Clipping {
    #[serde(skip_serializing)]
    pub doc_title: String,

    pub kind: ClippingKind,

    pub location_or_page: ClippingLocationOrPage,

    #[serde(skip_serializing)]
    pub date: NaiveDateTime,

    pub content: String,
}

// NOTE: Implemented PartialEq, Eq and Hash manually to treat otherwise identical Clippings with
//       different dates as duplicates
// IMRPOVEMENT: More ideallly, we should probably derive them and use a custom hasher only where we
//              need them deduplicated disregarding the dates
impl PartialEq for Clipping {
    fn eq(&self, other: &Self) -> bool {
        self.doc_title == other.doc_title
            && self.kind == other.kind
            && self.location_or_page == other.location_or_page
            && self.content == other.content
    }
}

impl Eq for Clipping {}

impl Hash for Clipping {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.doc_title.hash(state);
        self.kind.hash(state);
        self.location_or_page.hash(state);
        self.content.hash(state);
    }
}

impl Clipping {
    // TODO: Strip trailing commas and semicolons.
    pub fn context_aware_formatted_content(&self) -> String {
        let s = self.content.clone();
        if s.is_empty() {
            return s;
        }

        let s = s.trim();

        let s = s
            .replace("—", " — ") // Em dash
            .replace(" - ", " — ") // Hyphen
            .replace(" – ", " — ") // En dash
            .replace("---", " — ")
            .replace("--", " — ")
            .replace("\u{00A0}", " ") // Non-breaking space
            .replace("  ", " ");

        let first_char = s.chars().next().unwrap();
        // NOTE: Always treats strings starting with a non-Unicode Scalar Value as closed at
        //       the start, which is incorrect.
        // FIXME: Catches numbers and likely other characters that are neither case.
        //        For example "(".
        let is_start_open = s.is_char_boundary(1) && !first_char.is_uppercase();

        let last_char = s.chars().last().unwrap();
        // FIXME: Considers American style period inside apostrophes as open.
        let is_end_open = !['.', '?', '!'].contains(&last_char);

        match (is_start_open, is_end_open) {
            (true, true) => format!("[…] {} […]", s),
            (true, false) => format!("[{}]{}", first_char.to_uppercase(), &s[1..]),
            (false, true) => format!("{} […].", s),
            (false, false) => s,
        }
    }
}

#[derive(AsRefStr, Debug, EnumString, Eq, PartialEq, Hash)]
pub enum ClippingKind {
    Bookmark,
    Highlight,
    Note,
}

impl serde::ser::Serialize for ClippingKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(&self.as_ref().to_lowercase())
    }
}

#[derive(Eq, Debug, PartialEq, Hash)]
pub enum ClippingLocationOrPage {
    Singular(i32),
    Ranged { start: i32, end: i32 },
}

impl Ord for ClippingLocationOrPage {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Singular(self_lop), Self::Singular(oth_lop)) => self_lop.cmp(oth_lop),

            (Self::Singular(self_lop), Self::Ranged { start, end: _ }) => {
                if self_lop <= start {
                    Ordering::Less
                } else {
                    Ordering::Greater
                }
            }

            (Self::Ranged { .. }, Self::Singular(_)) => other.cmp(self).reverse(),

            (
                Self::Ranged {
                    start: self_start,
                    end: self_end,
                },
                Self::Ranged {
                    start: other_start,
                    end: other_end,
                },
            ) => {
                let starts_cmp = self_start.cmp(other_start);
                if starts_cmp == Ordering::Equal {
                    self_end.cmp(other_end)
                } else {
                    starts_cmp
                }
            }
        }
    }
}

impl PartialOrd for ClippingLocationOrPage {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl serde::ser::Serialize for ClippingLocationOrPage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        match self {
            ClippingLocationOrPage::Singular(location_or_page) => {
                serializer.serialize_i32(*location_or_page)
            }

            ClippingLocationOrPage::Ranged { start, end } => {
                let mut tup = serializer.serialize_tuple(2)?;
                tup.serialize_element(start)?;
                tup.serialize_element(end)?;
                tup.end()
            }
        }
    }
}
