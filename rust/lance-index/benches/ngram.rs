// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: Copyright The Lance Authors

use std::{sync::Arc, time::Duration};

use arrow::array::AsArray;
use arrow_array::{RecordBatch, StringArray, UInt64Array};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use datafusion::physical_plan::stream::RecordBatchStreamAdapter;
use futures::stream;
use itertools::Itertools;
use lance_core::cache::FileMetadataCache;
use lance_core::ROW_ID;
use lance_index::metrics::NoOpMetricsCollector;
use lance_index::scalar::lance_format::LanceIndexStore;
use lance_index::scalar::ngram::{NGramIndex, NGramIndexBuilder, NGramIndexBuilderOptions};
use lance_index::scalar::{ScalarIndex, TextQuery};
use lance_io::object_store::ObjectStore;
use object_store::path::Path;
#[cfg(target_os = "linux")]
use pprof::criterion::{Output, PProfProfiler};

fn bench_ngram(c: &mut Criterion) {
    const TOTAL: usize = 1_000_000;

    env_logger::init();

    let rt = tokio::runtime::Builder::new_multi_thread().build().unwrap();

    let tempdir = tempfile::tempdir().unwrap();
    let index_dir = Path::from_filesystem_path(tempdir.path()).unwrap();
    let store = rt.block_on(async {
        Arc::new(LanceIndexStore::new(
            Arc::new(ObjectStore::local()),
            index_dir,
            FileMetadataCache::no_cache(),
        ))
    });

    // generate 2000 different tokens
    let tokens = random_word::all(random_word::Lang::En);
    let row_id_col = Arc::new(UInt64Array::from(
        (0..TOTAL).map(|i| i as u64).collect_vec(),
    ));
    let docs = (0..TOTAL)
        .map(|_| {
            let num_words = rand::random::<usize>() % 30 + 1;
            let doc = (0..num_words)
                .map(|_| tokens[rand::random::<usize>() % tokens.len()])
                .collect::<Vec<_>>();
            doc.join(" ")
        })
        .collect_vec();
    let doc_col = Arc::new(StringArray::from(docs));
    let batch = RecordBatch::try_new(
        arrow_schema::Schema::new(vec![
            arrow_schema::Field::new("doc", arrow_schema::DataType::Utf8, false),
            arrow_schema::Field::new(ROW_ID, arrow_schema::DataType::UInt64, false),
        ])
        .into(),
        vec![doc_col, row_id_col],
    )
    .unwrap();

    let batches = (0..1000).map(|i| batch.slice(i * 1000, 1000)).collect_vec();

    let mut group = c.benchmark_group("train");

    group.sample_size(10);
    group.bench_function(format!("ngram_train({TOTAL})").as_str(), |b| {
        b.to_async(&rt).iter(|| async {
            let stream = RecordBatchStreamAdapter::new(
                batch.schema(),
                stream::iter(batches.clone().into_iter().map(Ok)),
            );
            let stream = Box::pin(stream);
            let mut builder =
                NGramIndexBuilder::try_new(NGramIndexBuilderOptions::default()).unwrap();
            let num_spill_files = builder.train(stream).await.unwrap();

            builder
                .write_index(store.as_ref(), num_spill_files, None)
                .await
                .unwrap();
        })
    });

    drop(group);

    let mut group = c.benchmark_group("search");

    group
        .sample_size(10)
        .measurement_time(Duration::from_secs(10));
    let index = rt.block_on(NGramIndex::load(store, None)).unwrap();
    group.bench_function(format!("ngram_search({TOTAL})").as_str(), |b| {
        b.to_async(&rt).iter(|| async {
            let sample_idx = rand::random::<usize>() % batch.num_rows();
            let sample = batch
                .column(0)
                .as_string::<i32>()
                .value(sample_idx)
                .to_string();
            black_box(
                index
                    .search(&TextQuery::StringContains(sample), &NoOpMetricsCollector)
                    .await
                    .unwrap(),
            );
        })
    });
}

#[cfg(target_os = "linux")]
criterion_group!(
    name=benches;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(10))
        .sample_size(10)
        .with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = bench_ngram);

// Non-linux version does not support pprof.
#[cfg(not(target_os = "linux"))]
criterion_group!(
    name=benches;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(10))
        .sample_size(10);
    targets = bench_ngram);

criterion_main!(benches);
