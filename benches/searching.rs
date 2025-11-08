use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rusty_files::{MatchMode, Query, SearchEngine};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

fn setup_indexed_engine(file_count: usize) -> (TempDir, SearchEngine) {
    let temp_dir = TempDir::new().unwrap();
    let data_dir = temp_dir.path().join("data");
    fs::create_dir(&data_dir).unwrap();

    for i in 0..file_count {
        let ext = match i % 4 {
            0 => "rs",
            1 => "txt",
            2 => "md",
            _ => "log",
        };
        let file_path = data_dir.join(format!("file_{}.{}", i, ext));
        fs::write(file_path, format!("content number {}", i)).unwrap();
    }

    let index_path = temp_dir.path().join("index.db");
    let engine = SearchEngine::new(&index_path).unwrap();
    engine.index_directory(&data_dir, None).unwrap();

    (temp_dir, engine)
}

fn benchmark_simple_search(c: &mut Criterion) {
    let (_temp_dir, engine) = setup_indexed_engine(1000);

    c.bench_function("search_simple", |b| {
        b.iter(|| {
            black_box(engine.search("file").unwrap());
        });
    });
}

fn benchmark_pattern_search(c: &mut Criterion) {
    let (_temp_dir, engine) = setup_indexed_engine(1000);

    c.bench_function("search_pattern", |b| {
        b.iter(|| {
            black_box(engine.search("*.rs").unwrap());
        });
    });
}

fn benchmark_fuzzy_search(c: &mut Criterion) {
    let (_temp_dir, engine) = setup_indexed_engine(1000);

    c.bench_function("search_fuzzy", |b| {
        b.iter(|| {
            let query = Query::new("fle".to_string())
                .with_match_mode(MatchMode::Fuzzy);

            black_box(engine.search_with_query(&query).unwrap());
        });
    });
}

fn benchmark_filtered_search(c: &mut Criterion) {
    let (_temp_dir, engine) = setup_indexed_engine(1000);

    c.bench_function("search_filtered", |b| {
        b.iter(|| {
            black_box(engine.search("file ext:rs").unwrap());
        });
    });
}

fn benchmark_complex_query(c: &mut Criterion) {
    let (_temp_dir, engine) = setup_indexed_engine(1000);

    c.bench_function("search_complex", |b| {
        b.iter(|| {
            black_box(
                engine
                    .search("file ext:rs,txt size:>0 mode:fuzzy limit:50")
                    .unwrap(),
            );
        });
    });
}

criterion_group!(
    benches,
    benchmark_simple_search,
    benchmark_pattern_search,
    benchmark_fuzzy_search,
    benchmark_filtered_search,
    benchmark_complex_query
);
criterion_main!(benches);
