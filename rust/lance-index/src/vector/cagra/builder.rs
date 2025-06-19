use crate::Result;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use snafu::location;
use crate::Error;
use pyo3::impl_::pymethods::IterBaseKind;


// Parameters for building product quantizer.
#[derive(Debug, Clone)]
pub struct CagraBuildParams {
    /// cagra build algorithm
    pub cagra_metric: String,
    pub cagra_intermediate_graph_degree: u32,
    pub cagra_graph_degree: u32,
    pub cagra_build_algo: String,

}

impl Default for CagraBuildParams {
    fn default() -> Self {
        Self {
            cagra_metric: "sqeuclidean".to_string(),
            cagra_intermediate_graph_degree: 128,
            cagra_graph_degree: 64,
            cagra_build_algo: "ivf_pq".to_string(),
        }
    }
}

impl CagraBuildParams {
    fn iter(&self) -> Vec<String> {
        vec![
            self.cagra_metric.to_string(),
            self.cagra_intermediate_graph_degree.to_string(),
            self.cagra_graph_degree.to_string(),
            self.cagra_build_algo.to_string(),
        ]
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

        // let mut array = ArrayVec::<_, 4>::new();
        // for param in cagra_params.iter_tag(){
        //     array.push(param);
        // }

        let mut cagra_params_vec = cagra_params.iter();
        for item in cagra_params.iter() {
            eprintln!("{}", item);
        }

        function.call1((cagra_params_vec,)).map_err(|e| Error::Index {
            message: format!("Failed to call function {}", e),
            location: location!(),
        })?;

        Ok(())
    })
}