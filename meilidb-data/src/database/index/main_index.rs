use std::sync::Arc;
use std::convert::TryInto;

use meilidb_schema::Schema;

use crate::ranked_map::RankedMap;
use crate::database::Error;

const SCHEMA_KEY:              &str = "schema";
const WORDS_KEY:               &str = "words";
const SYNONYMS_KEY:            &str = "synonyms";
const RANKED_MAP_KEY:          &str = "ranked-map";
const NUMBER_OF_DOCUMENTS_KEY: &str = "number-of-documents";

#[derive(Clone)]
pub struct MainIndex(pub(crate) crate::CfTree);

impl MainIndex {
    pub fn schema(&self) -> Result<Option<Schema>, Error> {
        match self.0.get(SCHEMA_KEY)? {
            Some(bytes) => {
                let schema = bincode::deserialize_from(bytes.as_ref())?;
                Ok(Some(schema))
            },
            None => Ok(None),
        }
    }

    pub fn set_schema(&self, schema: &Schema) -> Result<(), Error> {
        let bytes = bincode::serialize(schema)?;
        self.0.insert(SCHEMA_KEY, bytes)?;
        Ok(())
    }

    pub fn words_set(&self) -> Result<Option<fst::Set>, Error> {
        match self.0.get(WORDS_KEY)? {
            Some(bytes) => {
                let len = bytes.len();
                let value = Arc::from(bytes.as_ref());
                let fst = fst::raw::Fst::from_shared_bytes(value, 0, len)?;
                Ok(Some(fst::Set::from(fst)))
            },
            None => Ok(None),
        }
    }

    pub fn set_words_set(&self, value: &fst::Set) -> Result<(), Error> {
        self.0.insert(WORDS_KEY, value.as_fst().as_bytes()).map(drop).map_err(Into::into)
    }

    pub fn synonyms_set(&self) -> Result<Option<fst::Set>, Error> {
        match self.0.get(SYNONYMS_KEY)? {
            Some(bytes) => {
                let len = bytes.len();
                let value = Arc::from(bytes.as_ref());
                let fst = fst::raw::Fst::from_shared_bytes(value, 0, len)?;
                Ok(Some(fst::Set::from(fst)))
            },
            None => Ok(None),
        }
    }

    pub fn set_synonyms_set(&self, value: &fst::Set) -> Result<(), Error> {
        self.0.insert(SYNONYMS_KEY, value.as_fst().as_bytes()).map(drop).map_err(Into::into)
    }

    pub fn ranked_map(&self) -> Result<Option<RankedMap>, Error> {
        match self.0.get(RANKED_MAP_KEY)? {
            Some(bytes) => {
                let ranked_map = RankedMap::read_from_bin(bytes.as_ref())?;
                Ok(Some(ranked_map))
            },
            None => Ok(None),
        }
    }

    pub fn set_ranked_map(&self, value: &RankedMap) -> Result<(), Error> {
        let mut bytes = Vec::new();
        value.write_to_bin(&mut bytes)?;
        self.0.insert(RANKED_MAP_KEY, bytes)?;
        Ok(())
    }

    pub fn number_of_documents(&self) -> Result<u64, Error> {
        match self.0.get(NUMBER_OF_DOCUMENTS_KEY)? {
            Some(bytes) => {
                let array = (*bytes).try_into().unwrap();
                Ok(u64::from_be_bytes(array))
            },
            None => Ok(0),
        }
    }

    pub fn set_number_of_documents<F>(&self, f: F) -> Result<u64, Error>
    where F: FnOnce(u64) -> u64,
    {
        let new = self.number_of_documents().map(f)?;
        self.0.insert(NUMBER_OF_DOCUMENTS_KEY, new.to_be_bytes())?;
        Ok(new)
    }
}
