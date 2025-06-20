use crate::Result;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::types::PyList;
use snafu::location;
use crate::Error;
use pyo3::impl_::pymethods::IterBaseKind;
use arrow_array::ArrayRef;
use arrow::array::Int32Array;
use arrow::datatypes::Int32Type;
use std::sync::Arc;

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

// #[pyclass]
// #[derive(Debug, Clone)]
// pub struct pythonCagraParams {
//     pub data: &ArrayRef,
//     pub cagra_params: Vec<String>,
// }

// #[pymethods]
// impl pythonCagraParams {
//     fn displayData() -> &ArrayRef {
//         self.data.clone()
//     }
// }

fn arrayref_to_pylist(py: Python<'_>, array: &ArrayRef) -> PyResult<PyObject> {
    if let Some(int_arr) = array.as_any().downcast_ref::<Int32Array>() {
        let py_values: Vec<i32> = (0..int_arr.len()).map(|i| int_arr.value(i)).collect();

        let py_list: Bound<'_, PyList> = PyList::new(py, py_values)?;
        Ok(py_list.into_py(py))
    }else{
        Err(pyo3::exceptions::PyTypeError::new_err("Unsupported array type"))
    }
}

pub async fn build_cagra_index(
    data: &ArrayRef,
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
        let d_result = arrayref_to_pylist(py, data);
        let d = match d_result {
            Ok(obj) => obj,
            Err(e) => return Err(Error::Index {
                message: format!("Failed to call inner function: {}", e),
                location: location!(),
            }),
        };
        let mut cagra_params_vec = cagra_params.iter();
        // for item in cagra_params.iter() {
        //     eprintln!("{}", item);
        // }
        // let pythonParams = pythonCagraParams {
        //     data:data,
        //     cagra_params:cagra_params_vec
        // }

        function.call1((d, cagra_params_vec, )).map_err(|e| Error::Index {
            message: format!("Failed to call function {}", e),
            location: location!(),
        })?;

        Ok(())
    })
}