use std::fmt;

use pyo3::exceptions::PyOSError;
use pyo3::prelude::*;
use pyo3::types::PyUnicode;

use mibig_taxa::{MibigTaxonError, TaxonCache};

enum PyMibigTaxonError {
    MibigError(MibigTaxonError),
}

impl fmt::Display for PyMibigTaxonError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl From<MibigTaxonError> for PyMibigTaxonError {
    fn from(err: MibigTaxonError) -> PyMibigTaxonError {
        PyMibigTaxonError::MibigError(err)
    }
}

impl From<PyMibigTaxonError> for PyErr {
    fn from(err: PyMibigTaxonError) -> PyErr {
        PyOSError::new_err(err.to_string())
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
}

#[pymodule]
fn mibig_taxa(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyTaxonCache>()?;
    Ok(())
}
