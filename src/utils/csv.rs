use std::{fs::File, path::Path};

use csv::{DeserializeRecordsIntoIter, Reader};
use serde::de::DeserializeOwned;

/// Helper struct for iterating over CSV-Rows using `serde`
pub struct CSVIterator<D: DeserializeOwned>(DeserializeRecordsIntoIter<File, D>);

impl<D: DeserializeOwned> CSVIterator<D> {
    #[inline]
    pub fn from_path<P: AsRef<Path>>(path: P) -> Self {
        CSVIterator(
            Reader::from_path(path)
                .expect("Could not open CSV-file!")
                .into_deserialize(),
        )
    }
}

impl<D: DeserializeOwned> Iterator for CSVIterator<D> {
    type Item = D;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|res| res.expect("Could not parse Row!"))
    }
}
