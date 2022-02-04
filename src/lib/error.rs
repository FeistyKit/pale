use std::fmt::Display;

use crate::tokens::Location;

pub struct LispErrors {
    errs: Vec<(String, Vec<String>)>,
}

impl LispErrors {
    pub fn new() -> Self {
        Self { errs: Vec::new() }
    }
    pub fn error<T: Display>(mut self, loc: &Location, err: T) -> Self {
        self.errs.push((format!("{loc} - {err}!"), Vec::new()));
        self
    }
    pub fn note<'a, T: Display, L: Into<Option<&'a Location>>>(mut self, loc: L, err: T) -> Self {
        let loc: Option<&Location> = loc.into();
        if let Some((_, notes)) = self.errs.last_mut() {
            let msg = if let Some(l) = loc {
                format!("NOTE: {l} - {err}")
            } else {
                format!("NOTE: {err}")
            };
            notes.push(msg);
        }
        self
    }
}
