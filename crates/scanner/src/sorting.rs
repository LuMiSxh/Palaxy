//! Path sorting and parsing.

use lazy_static::lazy_static;
use regex::Regex;
use std::cmp::Ordering;
use std::ffi::OsStr;
use std::path::PathBuf;

lazy_static! {
    /// Regex for extracting numeric values from filenames.
    static ref RE: Regex = Regex::new(r"\d+\.?\d*").unwrap();
}

/// Sorts paths by numeric values in their file stem.
#[inline]
pub fn sort_by_stem_number(a: &PathBuf, b: &PathBuf) -> Ordering {
    fn parse_number(path: &PathBuf) -> Option<usize> {
        path.file_stem()?.to_str()?.parse::<usize>().ok()
    }

    parse_number(a).cmp(&parse_number(b))
}

/// Sorts paths by numeric values found in their names.
#[inline]
pub fn sort_name_by_number(a: &PathBuf, b: &PathBuf) -> Ordering {
    let an = regex_parser(a);
    let bn = regex_parser(b);

    an.partial_cmp(&bn).unwrap_or(Ordering::Equal)
}

/// Sorts paths by volume and chapter numbers in filenames.
///
/// Expects filenames in format "volume-chapter" (e.g., "1-15.jpg").
#[inline]
pub fn sort_by_name_volume_chapter(a: &PathBuf, b: &PathBuf) -> Ordering {
    fn parse_numbers(path: &PathBuf) -> (Option<f64>, Option<f64>) {
        // Early return if no filename
        let file_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(name) => name,
            None => return (None, None),
        };

        // Find the first '-' character to split volume and chapter
        if let Some(dash_pos) = file_name.find('-') {
            let (vol_str, chap_str) = file_name.split_at(dash_pos);
            let chap_str = &chap_str[1..]; // Skip the '-' character

            let first = vol_str.parse::<f64>().ok();
            let last = chap_str.parse::<f64>().ok();

            (first, last)
        } else {
            (None, None)
        }
    }

    let (a_vol, a_chap) = parse_numbers(a);
    let (b_vol, b_chap) = parse_numbers(b);

    match a_vol.partial_cmp(&b_vol) {
        Some(Ordering::Equal) => a_chap.partial_cmp(&b_chap).unwrap_or(Ordering::Equal),
        Some(order) => order,
        None => Ordering::Equal,
    }
}

/// Extracts a numeric value from a path using regex.
#[inline]
fn regex_parser(s: &PathBuf) -> Option<f64> {
    let file_name = s.file_name().unwrap_or_else(|| OsStr::new("")).to_str()?;

    // Find the last numeric match in the filename
    RE.captures_iter(file_name).last().and_then(|cap| {
        let capture = cap.get(0)?.as_str();

        // Fast path: if no leading zeros, parse directly
        if !capture.starts_with('0') || capture.len() == 1 {
            return capture.parse::<f64>().ok();
        }

        // Trim leading zeros for cases like "007.jpg"
        let trimmed = capture.trim_start_matches('0');
        if trimmed.is_empty() {
            Some(0.0)
        } else {
            trimmed.parse::<f64>().ok()
        }
    })
}
