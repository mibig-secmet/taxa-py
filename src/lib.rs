use std::error;
use std::fmt;

use pyo3::exceptions::{PyOSError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyLong, PyUnicode};

use mibig_taxa::{MibigTaxonError, TaxonCache};

#[derive(Debug)]
enum PyMibigTaxonError {
    MibigError(MibigTaxonError),
    NotFound(i64),
}

impl error::Error for PyMibigTaxonError {}

impl fmt::Display for PyMibigTaxonError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PyMibigTaxonError::MibigError(e) => write!(f, "{}", e),
            PyMibigTaxonError::NotFound(id) => write!(f, "ID {} not found", id),
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
            PyMibigTaxonError::NotFound(_) => PyValueError::new_err(err.to_string()),
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
}

#[pymodule]
fn mibig_taxa(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyTaxonCache>()?;
    Ok(())
}
