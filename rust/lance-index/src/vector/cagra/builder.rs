use crate::Result;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use snafu::location;
use crate::Error;

// Parameters for building product quantizer.
#[derive(Debug, Clone)]
pub struct CagraBuildParams {
    /// cagra build algorithm
    pub cagra_build_algo: String,

}

impl Default for CagraBuildParams {
    fn default() -> Self {
        Self {
            cagra_build_algo: "nn_descent".to_string(),
        }
    }
}

pub async fn build_cagra_index(
    cagra_params: &CagraBuildParams
) -> Result<()> {
    eprintln!("Cagra logic to go here");

    Python::with_gil(|py| {
        let module = PyModule::import(py, "lance.cagra").map_err(|e| Error::Index {
            message: format!("Failed to import lance.cagra: {}", e),
            location: location!(),
        })?;

        let function = module.getattr("build_cagra_index").map_err(|e| Error::Index {
            message: format!("Failed to get attribute {}", e),
            location: location!(),
        })?;

        function.call1((cagra_params.cagra_build_algo.clone(),)).map_err(|e| Error::Index {
            message: format!("Failed to call function {}", e),
            location: location!(),
        })?;

        Ok(())
    })
}