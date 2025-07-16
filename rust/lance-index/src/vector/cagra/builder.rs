use crate::Result;
use pyo3::prelude::*;
use pyo3::types::PyList;
use pyo3::types::PyDict;
use snafu::location;
use crate::Error;
use arrow_array::{Array, PrimitiveArray, FixedSizeListArray};
use std::sync::Arc;
use arrow::datatypes::Float32Type;
use pyo3::types::IntoPyDict;
use arrow_array::Int64Array;
use arrow_array::StringArray;
use pyo3::Py;

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
    fn iter(&self, py: Python<'_>) -> Py<PyDict> {
        let param_vec = vec![
            ("metric", self.cagra_metric.to_string()),
            ("intermediate_graph_degree", self.cagra_intermediate_graph_degree.to_string()),
            ("graph_degree", self.cagra_graph_degree.to_string()),
            ("algo", self.cagra_build_algo.to_string()),
        ];

        return param_vec.into_py_dict(py).expect("Failed to convert to PyDict").into();
    }
}

fn print_type<T>(_: &T) { 
    println!("{:?}", std::any::type_name::<T>());
}

fn data_array_to_pylist(py: Python<'_>, array: &Arc<dyn arrow_array::Array>) -> Py<PyList> {
    //eprintln!("{:?}", print_type(&array));

    if let Some(list_array) = array.as_any().downcast_ref::<FixedSizeListArray>() {
        let py_outer_list = PyList::empty(py);

        for i in 0..list_array.len() {
            let subarray = list_array.value(i);

            if let Some(float_subarray) = subarray.as_any().downcast_ref::<PrimitiveArray<Float32Type>>(){
                let py_inner_list = PyList::new(py, float_subarray).unwrap();
                py_outer_list.append(py_inner_list).unwrap();
            } else {
                py_outer_list.append(PyList::empty(py)).unwrap();
            }
        }
        return py_outer_list.into();
    } else {
        return PyList::empty(py).into();
    }
}

fn id_array_to_pylist(py: Python<'_>, array: &Arc<dyn arrow_array::Array>) -> Py<PyList> {
    let mut id_values = PyList::empty(py);
    if let Some(string_array) = array.as_any().downcast_ref::<StringArray>() {
        for i in 0..string_array.len() {
            let val = string_array.value(i);
            match val.parse::<i64>() {
                Ok(parsed) =>  id_values.append(parsed).unwrap(),
                Err(err) => {
                    eprintln!("Failed to parse '{}' as i64 at index {}: {}", val, i, err);
                    continue;
                }
            }
        }
    }
    return id_values.into();
}

pub async fn build_cagra_index(
    data: &Arc<dyn arrow_array::Array>,
    ids: &Arc<dyn arrow_array::Array>,
    cagra_params: &CagraBuildParams
) -> Result<()> {
    Python::with_gil(|py| {
        let module = PyModule::import(py, "lance.cagra").map_err(|e| Error::Index {
            message: format!("Failed to import lance.cagra: {}", e),
            location: location!(),
        })?;

        let function = module.getattr("build_cagra_index").map_err(|e| Error::Index {
            message: format!("Failed to get attribute {}", e),
            location: location!(),
        })?;

        let data_result = data_array_to_pylist(py, data);
        // if(data_result.is_empty()) {
        //     eprintln!("Data not transformed properly");
        // }
        // let data_list = match data_result {
        //     Ok(obj) => obj,
        //     Err(e) => return Err(Error::Index {
        //         message: format!("Failed to call inner function: {}", e),
        //         location: location!(),
        //     }),
        // };

        let cagra_params_vec = cagra_params.iter(py);
        let ids_pylist = id_array_to_pylist(py, ids);

        eprintln!("ids pylist is {:?}", ids_pylist);

        function.call1((data_result, ids_pylist, cagra_params_vec)).map_err(|e| Error::Index {
            message: format!("Failed to call function {}", e),
            location: location!(),
        })?;

        Ok(())
    })

}