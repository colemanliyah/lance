use crate::Dataset;
use crate::Result;
use lance_index::pb;
use snafu::location;
use lance_index::pb::{Cagra, VectorIndexStage};
use lance_index::pb::vector_index_stage::Stage;
use lance_core::Error;
use lance_linalg::distance::MetricType;
use std::sync::Arc;
use arrow_array::{Array, FixedSizeListArray};
use std::collections::HashMap;
use lance_io::traits::WriteExt;

use crate::{
    index::{INDEX_FILE_NAME}
};

#[derive(Debug)]
pub struct CagraIndexMetaData {
    // Index name
    name: String,

    // Column we built the index for
    column: String,

    // Version of dataset where index was built
    dataset_version: u64,

    // Metric used the build the algorithm
    metric: String,

    // Algorithm used to build index
    algo: String,

    // Dimension of one embedding
    dimension: u32

}

impl CagraIndexMetaData {
    pub fn new(
        name: String,
        column: String,
        dataset_version: u64,
        metric: String,
        algo: String,
        dimension: u32
    ) -> Self {
        Self {
            name,
            column,
            dataset_version,
            metric,
            algo,
            dimension
        }
    }
}

/// Convert a CagraIndexMetaData to protobuf payload
impl TryFrom<&CagraIndexMetaData> for pb::Index {
    type Error = Error;

    fn try_from(idx: &CagraIndexMetaData) -> Result<Self> {
        let cagra_stage = VectorIndexStage {
            stage: Some(Stage::Cagra(Cagra {
                build_algo: idx.algo.clone(),
                metric: idx.metric.clone()
            }))
        };

        let metric_conversion = HashMap::from([
            ("sqeuclidean".to_string(), MetricType::L2),
            ("inner_product".to_string(), MetricType::Dot)

        ]);

        let index = metric_conversion.get(idx.metric.as_str()).cloned().expect("Metric must be 'sqeuclidea' or 'inner product' ");

        Ok(Self {
            name: idx.name.clone(),
            columns: vec![idx.column.clone()],
            dataset_version: idx.dataset_version,
            index_type: pb::IndexType::Vector.into(),
            implementation: Some(pb::index::Implementation::VectorIndex(pb::VectorIndex {
                spec_version: 1,
                dimension: idx.dimension, //array.shape()[1]
                stages: vec![cagra_stage],
                metric_type: match index {
                    MetricType::L2 => pb::VectorMetricType::L2.into(), // Mimic euclidean, closest for sequclidean
                    MetricType::Dot => pb::VectorMetricType::Dot.into(), // Mimic inner_product
                    _ => {
                        return Err(Error::Index { 
                            message: "unsupported cagra distance metric".to_string(),
                            location: location!(),
                        })
                    }
                },
            })),
        })
    }
}

pub fn extract_dimension(data: &Arc<dyn arrow_array::Array>) -> u32 {
    if let Some(list_array) = data.as_any().downcast_ref::<FixedSizeListArray>() {
        let dim = list_array.len();
        return dim as u32;
    } else {
        return 0
    }
}

pub async fn save_cagra_index(
    dataset: &Dataset,
    data: &Arc<dyn arrow_array::Array>,
    column: &str,
    index_name: &str,
    uuid: &str,
) -> Result<()> {
    eprintln!("Maade i in save cagra index");
    let object_store = dataset.object_store();
    let index_dir = dataset.indices_dir();
    let dataset_version = dataset.version().version;

    let path = index_dir.child(uuid).child(INDEX_FILE_NAME);
    let mut writer = object_store.create(&path).await?;

    let mut file = tokio::fs::File::open("/workspace/cagra_index.bin").await?; //eventually this should not be hardcoded
    tokio::io::copy(&mut file, &mut writer).await?;


    let metadata = CagraIndexMetaData {
        name: index_name.to_string(),
        column: column.to_string(),
        dataset_version,
        metric: "sqeuclidean".to_string(), // Change later to not be hardcoded
        algo: "ivf_pq".to_string(), // Change later to not be hardcoded
        dimension: extract_dimension(data)
    };

    let metadata = pb::Index::try_from(&metadata)?;
    let pos = writer.write_protobuf(&metadata).await?;
    //writer.write_magics(pos, 0, 1, MAGIC).await?; //Still not sure about this line

    writer.shutdown().await?;

    Ok(())
}