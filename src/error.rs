use std::fmt::Display;

use crate::tokens::Location;

#[derive(Debug)]
pub struct LispErrors {
    errs: Vec<(String, Vec<String>)>,
}

impl Display for LispErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for err in &self.errs {
            write!(f, "{}", err.0)?;
            for note in &err.1 {
                write!(f, "\n\t{}", note)?;
            }
        }
        Ok(())
    }
}

impl LispErrors {
    pub fn new() -> Self {
        Self { errs: Vec::new() }
    }
    pub fn error<T: Display>(mut self, loc: &Location, err: T) -> Self {
        self.errs.push((format!("{loc} - {err}"), Vec::new()));
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
    pub fn extend(&mut self, other: Self) {
        self.errs.extend(other.errs)
    }
}
