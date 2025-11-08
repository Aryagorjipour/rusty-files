use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use rusty_files::SearchEngine;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

fn create_test_files(dir: &PathBuf, count: usize) {
    for i in 0..count {
        let file_path = dir.join(format!("test_file_{}.txt", i));
        fs::write(file_path, format!("content {}", i)).unwrap();
    }

    for i in 0..count / 10 {
        let subdir = dir.join(format!("subdir_{}", i));
        fs::create_dir(&subdir).unwrap();

        for j in 0..10 {
            let file_path = subdir.join(format!("file_{}_{}.txt", i, j));
            fs::write(file_path, format!("content {} {}", i, j)).unwrap();
        }
    }
}

fn benchmark_indexing(c: &mut Criterion) {
    let mut group = c.benchmark_group("indexing");

    for size in [100, 500, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let temp_dir = TempDir::new().unwrap();
                let data_dir = temp_dir.path().join("data");
                fs::create_dir(&data_dir).unwrap();

                create_test_files(&data_dir, size);

                let index_path = temp_dir.path().join("index.db");
                let engine = SearchEngine::new(&index_path).unwrap();

                black_box(engine.index_directory(&data_dir, None).unwrap());
            });
        });
    }

    group.finish();
}

fn benchmark_incremental_update(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let data_dir = temp_dir.path().join("data");
    fs::create_dir(&data_dir).unwrap();

    create_test_files(&data_dir, 500);

    let index_path = temp_dir.path().join("index.db");
    let engine = SearchEngine::new(&index_path).unwrap();
    engine.index_directory(&data_dir, None).unwrap();

    c.bench_function("incremental_update", |b| {
        b.iter(|| {
            fs::write(data_dir.join("new_file.txt"), "new content").unwrap();

            black_box(engine.update_index(&data_dir, None).unwrap());

            fs::remove_file(data_dir.join("new_file.txt")).ok();
        });
    });
}

criterion_group!(benches, benchmark_indexing, benchmark_incremental_update);
criterion_main!(benches);
