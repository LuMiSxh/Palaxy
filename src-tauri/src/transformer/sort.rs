// This file contains the trait and impls for different sorting methods for Vec<PathBuf>.
// This should be an improved version of the sorting functions in collector.rs.

use lazy_static::lazy_static;
use regex::Regex;
use std::cmp::Ordering;
use std::ffi::OsStr;
use std::path::PathBuf;

lazy_static! {
    static ref REG: Regex = Regex::new(r"\d+\.?\d*").unwrap();
}

pub trait Sort {
    fn sort(&self, a: &PathBuf, b: &PathBuf) -> Ordering;

    fn is_sortable(&self, path: &PathBuf) -> bool;
}

pub struct SortByStemNumber;

impl Sort for SortByStemNumber {
    fn sort(&self, a: &PathBuf, b: &PathBuf) -> Ordering {
        let parse_stem = |path: &PathBuf| {
            path.file_stem()
                .and_then(|stem| stem.to_str())
                .and_then(|s| s.parse::<usize>().ok())
        };

        parse_stem(a).cmp(&parse_stem(b))
    }

    fn is_sortable(&self, path: &PathBuf) -> bool {
        path.file_stem()
            .and_then(|stem| stem.to_str())
            .and_then(|s| s.parse::<usize>().ok())
            .is_some()
    }
}

pub struct SortByStemVolumeChapter;

impl Sort for SortByStemVolumeChapter {
    fn sort(&self, a: &PathBuf, b: &PathBuf) -> Ordering {
        // This closure will extract the volume and chapter number from the file name
        let num = |path: &PathBuf, first: bool| -> Option<f64> {
            if first {
                path.file_name()?
                    .to_str()?
                    .split("-")
                    .next()?
                    .parse::<f64>()
                    .ok()
            } else {
                path.file_name()?
                    .to_str()?
                    .split("-")
                    .last()?
                    .parse::<f64>()
                    .ok()
            }
        };

        let an = (num(a, true), num(a, false));
        let bn = (num(b, true), num(b, false));

        // This will compare the volume number first and then the chapter number
        if an.0 == bn.0 {
            an.1.partial_cmp(&bn.1).unwrap()
        } else {
            an.0.partial_cmp(&bn.0).unwrap()
        }
    }

    fn is_sortable(&self, path: &PathBuf) -> bool {
        let num = |path: &PathBuf, first: bool| -> Option<f64> {
            if first {
                path.file_name()?
                    .to_str()?
                    .split("-")
                    .next()?
                    .parse::<f64>()
                    .ok()
            } else {
                path.file_name()?
                    .to_str()?
                    .split("-")
                    .last()?
                    .parse::<f64>()
                    .ok()
            }
        };

        num(path, true).is_some() && num(path, false).is_some()
    }
}

pub struct SortByRegex;

impl SortByRegex {
    fn regex_parser(s: &PathBuf) -> Option<f64> {
        let (capture, []) = REG
            .captures_iter(
                s.file_name()
                    .unwrap_or(OsStr::new(""))
                    .to_str()
                    .unwrap_or(""),
            )
            .last()?
            .extract();

        Some(capture.trim_start_matches("0").trim().parse::<f64>().ok()?)
    }
}

impl Sort for SortByRegex {
    fn sort(&self, a: &PathBuf, b: &PathBuf) -> Ordering {
        let an = SortByRegex::regex_parser(a);
        let bn = SortByRegex::regex_parser(b);

        an.partial_cmp(&bn).unwrap()
    }

    fn is_sortable(&self, path: &PathBuf) -> bool {
        SortByRegex::regex_parser(path).is_some()
    }
}
