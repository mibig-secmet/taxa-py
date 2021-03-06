use std::error;
use std::fmt;

use mibig_taxa::NcbiTaxEntry;
use pyo3::exceptions::{PyOSError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyLong, PyUnicode};

use mibig_taxa::{MibigTaxonError, TaxonCache};

#[derive(Debug)]
enum PyMibigTaxonError {
    MibigError(MibigTaxonError),
    NotFound(i64),
    InvalidAntismashTaxon(String),
}

impl error::Error for PyMibigTaxonError {}

impl fmt::Display for PyMibigTaxonError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PyMibigTaxonError::MibigError(e) => write!(f, "{}", e),
            PyMibigTaxonError::NotFound(id) => write!(f, "ID {} not found", id),
            PyMibigTaxonError::InvalidAntismashTaxon(tax) => {
                write!(f, "Can't map taxon {} to an antiSMASH taxon", tax)
            }
        }
    }
}

impl std::convert::From<MibigTaxonError> for PyMibigTaxonError {
    fn from(err: MibigTaxonError) -> PyMibigTaxonError {
        PyMibigTaxonError::MibigError(err)
    }
}

impl std::convert::From<PyMibigTaxonError> for PyErr {
    fn from(err: PyMibigTaxonError) -> PyErr {
        match err {
            PyMibigTaxonError::MibigError(_) => PyOSError::new_err(err.to_string()),
            PyMibigTaxonError::NotFound(_) | PyMibigTaxonError::InvalidAntismashTaxon(_) => {
                PyValueError::new_err(err.to_string())
            }
        }
    }
}

/// Python version of NcbiTaxEntry
#[pyclass(name = "TaxonEntry", module = "mibig_taxa")]
struct PyTaxonEntry {
    #[pyo3(get)]
    tax_id: i64,
    #[pyo3(get)]
    name: String,
    #[pyo3(get)]
    species: String,
    #[pyo3(get)]
    genus: String,
    #[pyo3(get)]
    family: String,
    #[pyo3(get)]
    order: String,
    #[pyo3(get)]
    class: String,
    #[pyo3(get)]
    phylum: String,
    #[pyo3(get)]
    kingdom: String,
    #[pyo3(get)]
    superkingdom: String,
}

#[pymethods]
impl PyTaxonEntry {
    pub fn __str__(&self) -> String {
        format!("{} ({})", self.name, self.tax_id)
    }

    pub fn __repr__(&self) -> String {
        format!("{} ({})", self.name, self.tax_id)
    }

    pub fn get_antismash_taxon(&self) -> PyResult<String> {
        let ncbi_entry: NcbiTaxEntry = self.into();
        get_taxon_from_entry(&ncbi_entry)
    }
}

impl std::convert::From<&NcbiTaxEntry> for PyTaxonEntry {
    fn from(entry: &NcbiTaxEntry) -> Self {
        PyTaxonEntry {
            tax_id: entry.tax_id,
            name: entry.name.to_string(),
            species: entry.species.to_string(),
            genus: entry.genus.to_string(),
            family: entry.family.to_string(),
            order: entry.order.to_string(),
            class: entry.class.to_string(),
            phylum: entry.phylum.to_string(),
            kingdom: entry.kingdom.to_string(),
            superkingdom: entry.superkingdom.to_string(),
        }
    }
}

impl std::convert::From<&PyTaxonEntry> for NcbiTaxEntry {
    fn from(entry: &PyTaxonEntry) -> Self {
        NcbiTaxEntry {
            tax_id: entry.tax_id,
            name: entry.name.to_string(),
            species: entry.species.to_string(),
            genus: entry.genus.to_string(),
            family: entry.family.to_string(),
            order: entry.order.to_string(),
            class: entry.class.to_string(),
            phylum: entry.phylum.to_string(),
            kingdom: entry.kingdom.to_string(),
            superkingdom: entry.superkingdom.to_string(),
        }
    }
}

/// Python version of the TaxonCache
#[pyclass(name = "TaxonCache", module = "mibig_taxa")]
struct PyTaxonCache {
    cache: TaxonCache,
}

#[pymethods]
impl PyTaxonCache {
    #[new]
    fn new(cachefile: Option<&PyUnicode>) -> PyResult<Self> {
        let mut cache = PyTaxonCache {
            cache: TaxonCache::new(),
        };

        if let Some(filename) = cachefile {
            cache.load(filename)?;
        }
        Ok(cache)
    }

    pub fn initialise(
        &mut self,
        taxdump: &PyUnicode,
        merged_id_dump: &PyUnicode,
        datadir: &PyUnicode,
    ) -> PyResult<()> {
        self.cache
            .initialise_from_paths(
                taxdump.extract()?,
                merged_id_dump.extract()?,
                datadir.extract()?,
            )
            .map_err(PyMibigTaxonError::from)?;
        Ok(())
    }

    pub fn load(&mut self, cachefile: &PyUnicode) -> PyResult<usize> {
        let size = self
            .cache
            .load_path(&cachefile.extract()?)
            .map_err(PyMibigTaxonError::from)?;
        Ok(size)
    }

    pub fn save(&self, cachefile: &PyUnicode) -> PyResult<usize> {
        let size = self
            .cache
            .save_path(&cachefile.extract()?)
            .map_err(PyMibigTaxonError::from)?;
        Ok(size)
    }

    #[args(allow_deprecated = "false")]
    pub fn get_name_by_id(&self, id: &PyLong, allow_deprecated: bool) -> PyResult<String> {
        let tax_id: i64 = id.extract()?;

        if let Some(entry) = self.cache.mappings.get(&tax_id) {
            return Ok(entry.name.clone());
        } else {
            if !allow_deprecated {
                let err = PyMibigTaxonError::NotFound(tax_id);
                return Err(PyErr::from(err));
            }
            if let Some(new_id) = self.cache.deprecated_ids.get(&tax_id) {
                if let Some(entry) = self.cache.mappings.get(&new_id) {
                    return Ok(entry.name.clone());
                }
            }
        }
        let err = PyMibigTaxonError::NotFound(tax_id);
        Err(PyErr::from(err))
    }

    #[args(allow_deprecated = "false")]
    pub fn get_antismash_taxon(&self, id: &PyLong, allow_deprecated: bool) -> PyResult<String> {
        let tax_id: i64 = id.extract()?;

        if let Some(entry) = self.cache.mappings.get(&tax_id) {
            return get_taxon_from_entry(entry);
        } else {
            if !allow_deprecated {
                let err = PyMibigTaxonError::NotFound(tax_id);
                return Err(PyErr::from(err));
            }
            if let Some(new_id) = self.cache.deprecated_ids.get(&tax_id) {
                if let Some(entry) = self.cache.mappings.get(&new_id) {
                    return get_taxon_from_entry(entry);
                }
            }
        }
        let err = PyMibigTaxonError::NotFound(tax_id);
        Err(PyErr::from(err))
    }

    #[args(allow_deprecated = "false")]
    pub fn get(&self, id: &PyLong, allow_deprecated: bool) -> PyResult<PyTaxonEntry> {
        let tax_id: i64 = id.extract()?;

        if let Some(entry) = self.cache.mappings.get(&tax_id) {
            return Ok(PyTaxonEntry::from(entry));
        } else {
            if !allow_deprecated {
                let err = PyMibigTaxonError::NotFound(tax_id);
                return Err(PyErr::from(err));
            }
            if let Some(new_id) = self.cache.deprecated_ids.get(&tax_id) {
                if let Some(entry) = self.cache.mappings.get(&new_id) {
                    return Ok(PyTaxonEntry::from(entry));
                }
            }
        }
        let err = PyMibigTaxonError::NotFound(tax_id);
        Err(PyErr::from(err))
    }
}

fn get_taxon_from_entry(entry: &NcbiTaxEntry) -> PyResult<String> {
    match entry.superkingdom.as_str() {
        "Archaea" | "Bacteria" => return Ok("bacteria".to_string()),
        "Eukaryota" => match entry.kingdom.as_str() {
            "Fungi" => return Ok("fungi".to_string()),
            "Viridiplantae" => return Ok("plants".to_string()),
            "Unknown" => match entry.phylum.as_str() {
                "Rhodophyta" | "Bacillariophyta" => return Ok("plants".to_string()),
                "Unknown" => match entry.class.as_str() {
                    "Dinophyceae" => return Ok("plants".to_string()),
                    _ => {
                        let err = PyMibigTaxonError::InvalidAntismashTaxon(entry.class.clone());
                        return Err(PyErr::from(err));
                    }
                },
                _ => {
                    let err = PyMibigTaxonError::InvalidAntismashTaxon(entry.phylum.clone());
                    return Err(PyErr::from(err));
                }
            },
            _ => {
                let err = PyMibigTaxonError::InvalidAntismashTaxon(entry.kingdom.clone());
                return Err(PyErr::from(err));
            }
        },
        // Many metagenomes are superkingdom "Unknown" but still bacterial
        _ => return Ok("bacteria".to_string()),
    }
}

#[pymodule]
fn mibig_taxa(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyTaxonCache>()?;
    m.add_class::<PyTaxonEntry>()?;
    Ok(())
}
