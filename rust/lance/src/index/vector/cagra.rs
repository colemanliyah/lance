use crate::index::vector::VectorIndexParams;
use crate::Dataset;
use crate::Result;
use crate::index::prefilter::PreFilter;
use crate::index::vector::IvfIndexPartitionStatistics;
use crate::index::vector::IvfIndexStatistics;
use roaring::RoaringBitmap;
use lance_io::local::to_local_path;
use lance_io::traits::Reader;
 use lance_index::vector::Query;
use lance_index::pb;
use lance_index::Index;
use lance_index::pb::VectorIndex;
use lance_index::vector::cagra::CagraBuildParams;
use lance_index::IndexType;
use snafu::location;
use lance_index::vector::hnsw::HNSWIndex;
use lance_index::vector::sq::ScalarQuantizer;
use lance_index::vector::pq::ProductQuantizer;
use lance_index::pb::{Cagra, VectorIndexStage};
use lance_index::pb::vector_index_stage::Stage;
use lance_index::metrics::MetricsCollector;
use lance_index::metrics::NoOpMetricsCollector;
use lance_core::Error;
use lance_linalg::distance::MetricType;
use std::sync::Arc;
use arrow_array::{Array, FixedSizeListArray};
use std::collections::HashMap;
use lance_io::traits::WriteExt;
use async_trait::async_trait;
use std::any::Any;
use deepsize::DeepSizeOf;
use super::pq::PQIndex;
use arrow_array::types::Float16Type;
use arrow_array::RecordBatch;
use arrow_array::types::Float32Type;
use arrow_array::types::Float64Type;
use arrow_array::cast::AsArray;
use arrow_schema::DataType;
use arrow_schema::Schema;
use arrow::datatypes::UInt8Type;
use lance_index::vector::VectorIndex as LanceIndexVectorIndex;
use lance_index::vector::v3::subindex::SubIndexType;
use serde::Serialize;
use tracing::instrument;
use arrow_array::UInt32Array;
use datafusion::execution::SendableRecordBatchStream;
use lance_index::vector::ivf::storage::IvfModel;
use lance_index::vector::quantizer::Quantizer;
use lance_index::vector::quantizer::QuantizationType;

use crate::{
    index::{INDEX_FILE_NAME}
};

#[derive(Debug)]
pub struct CagraIndex {
    uuid: String,

    index: String,
}

impl CagraIndex {
    pub fn new(
        uuid: String,
        index: String,
    ) -> Self {
        Self {
            uuid,
            index
        }
    }
}

#[derive(Serialize)]
pub struct CagraIndexStatistics {
    pub index_type: String,
    // uuid: String,
    // uri: String,
    // metric_type: String,
    // num_partitions: usize,
    // sub_index: serde_json::Value,
    // partitions: Vec<IvfIndexPartitionStatistics>,
    // centroids: Vec<Vec<f32>>,
    // loss: Option<f64>,
}

// Used for calculating heap memory - something like this
impl DeepSizeOf for CagraIndex {
    fn deep_size_of_children(&self, context: &mut deepsize::Context) -> usize {
        self.uuid.deep_size_of_children(context)
            + self.index.deep_size_of_children(context)
        // Skipping session since it is a weak ref
    }
}

#[async_trait]
impl Index for CagraIndex {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_index(self: Arc<Self>) -> Arc<dyn Index> {
        self
    }

    fn as_vector_index(self: Arc<Self>) -> Result<Arc<dyn LanceIndexVectorIndex>> {
        Ok(self)
    }

    fn index_type(&self) -> IndexType {
        IndexType::Cagra
    }

    async fn prewarm(&self) -> Result<()> {
        // TODO: We should prewarm the IVF index by loading the partitions into memory
        Ok(())
    }

    fn statistics(&self) -> Result<serde_json::Value> {
        Ok(serde_json::to_value(CagraIndexStatistics {
            index_type: self.index_type().to_string()
        })?)
    }

    // Collect the ids of data fragments used by an index; Not sure if it's needed yet
    async fn calculate_included_frags(&self) -> Result<RoaringBitmap> {
        let mut frag_ids = RoaringBitmap::default();
        // Options:
        //  1. Compute frag ids from row ids (add row ids as a field in CagraIndex struct)
        //  2. Have frag ids already in the CagraIndex struct and just clone and pass them here
        //  3. Return empty frags if not needed
        Ok(frag_ids)
    }
}

#[async_trait]
impl LanceIndexVectorIndex for CagraIndex {
    #[instrument(level = "debug", skip_all, name = "IVFIndex::search")]
    async fn search(
        &self,
        _query: &Query,
        _pre_filter: Arc<dyn PreFilter>,
        _metrics: &dyn MetricsCollector,
    ) -> Result<RecordBatch> {
        unimplemented!("search is not yet implemented for Cagra");
    }

    // Getting partitions from empty vector since cagra won't have partitions
    fn find_partitions(&self, query: &Query) -> Result<UInt32Array> {
        //Ok(arrow_array::PrimitiveArray::<UInt32Type>::from(vec![]))
        unimplemented!("Not supported for Cagra");
    }

    async fn search_in_partition(
        &self,
        _: usize,
        _: &Query,
        _: Arc<dyn PreFilter>,
        _: &dyn MetricsCollector,
    ) -> Result<RecordBatch> {
        eprintln!("cagar in serach partition");

        let schema = Arc::new(Schema::empty());
        let batch = RecordBatch::new_empty(schema);
        Ok(batch)
    }

    fn total_partitions(&self) -> usize {
        1
    }

    fn is_loadable(&self) -> bool {
        false
    }

    fn use_residual(&self) -> bool {
        false
    }

    async fn load(
        &self,
        _reader: Arc<dyn Reader>,
        _offset: usize,
        _length: usize,
    ) -> Result<Box<dyn LanceIndexVectorIndex>> {
        // Err(Error::Index {
        //     message: "Flat index does not support load".to_string(),
        //     location: location!(),
        // })
        unimplemented!("load is not yet implemented for Cagra")
    }

    async fn to_batch_stream(&self, _with_vector: bool) -> Result<SendableRecordBatchStream> {
        unimplemented!("to_batch_stream is not yet implemented for Cagra")
    }

    fn num_rows(&self) -> u64 {
        0 // TODO: Return actual number of rows indexed
    }

    fn row_ids(&self) -> Box<dyn Iterator<Item = &u64>> {
        todo!("this method is for only IVF_HNSW_* index");
    }

    async fn remap(&mut self, _mapping: &HashMap<u64, Option<u64>>) -> Result<()> {
        // This will be needed if we want to clean up IVF to allow more than just
        // one layer (e.g. IVF -> IVF -> PQ).  We need to pass on the call to
        // remap to the lower layers.

        // Currently, remapping for IVF is implemented in remap_index_file which
        // mirrors some of the other IVF routines like build_ivf_pq_index
        Err(Error::Index {
            message: "Remapping Cagra in this way not supported".to_string(),
            location: location!(),
        })
    }

    fn ivf_model(&self) -> &IvfModel {
        panic!("Cagra does not use an IVF model")
    }

    fn quantizer(&self) -> Quantizer {
        panic!("Cagra does not use a quantizer")
    }

    /// the index type of this vector index.
    fn sub_index_type(&self) -> (SubIndexType, QuantizationType) {
        unimplemented!("Don't think this applys to cagra")
    }

    fn metric_type(&self) -> MetricType {
        unimplemented!("Need to edit cagra index stuct to include this field");
    }


}

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

        let index = metric_conversion.get(idx.metric.as_str()).cloned().expect("Metric must be 'sqeuclidean' or 'inner product' ");

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
    params: &CagraBuildParams,
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
        metric: params.cagra_metric.to_string(), // Change later to not be hardcoded
        algo: params.cagra_build_algo.to_string(), // Change later to not be hardcoded
        dimension: extract_dimension(data)
    };

    let metadata = pb::Index::try_from(&metadata)?;
    let pos = writer.write_protobuf(&metadata).await?;
    //writer.write_magics(pos, 0, 1, MAGIC).await?; //Still not sure about this line

    writer.shutdown().await?;

    Ok(())
}