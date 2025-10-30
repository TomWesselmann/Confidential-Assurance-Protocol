/// Registry Performance Benchmarks
///
/// Measures performance of JSON vs SQLite backends for various operations:
/// - insert: Adding entries to registry
/// - load: Loading entire registry from disk
/// - find: Finding entries by hash
/// - list: Listing all entries

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use cap_agent::registry::{RegistryEntry, RegistryBackend, open_store};
use std::path::Path;
use std::fs;

/// Helper: Create mock registry entry
fn create_mock_entry(id: usize) -> RegistryEntry {
    RegistryEntry {
        id: format!("proof_{:05}", id),
        manifest_hash: format!("0x{:064x}", id),
        proof_hash: format!("0x{:064x}", id + 1000000),
        timestamp_file: if id % 3 == 0 {
            Some(format!("timestamp_{}.tsr", id))
        } else {
            None
        },
        registered_at: chrono::Utc::now().to_rfc3339(),
        signature: None,
        public_key: None,
    }
}

/// Helper: Setup temporary registry with entries
fn setup_registry(backend: RegistryBackend, num_entries: usize, path: &str) {
    // Clean up if exists
    let _ = fs::remove_file(path);

    let store = open_store(backend, Path::new(path)).unwrap();

    for i in 0..num_entries {
        let entry = create_mock_entry(i);
        store.add_entry(entry).unwrap();
    }
}

/// Benchmark: Insert entries
fn bench_registry_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("registry_insert");

    for size in [100, 1000].iter() {
        // JSON backend
        group.bench_with_input(BenchmarkId::new("json", size), size, |b, &size| {
            b.iter(|| {
                let path = "bench_insert_json.json";
                let _ = fs::remove_file(path);
                setup_registry(black_box(RegistryBackend::Json), black_box(size), path);
                let _ = fs::remove_file(path);
            });
        });

        // SQLite backend
        group.bench_with_input(BenchmarkId::new("sqlite", size), size, |b, &size| {
            b.iter(|| {
                let path = "bench_insert_sqlite.db";
                let _ = fs::remove_file(path);
                let _ = fs::remove_file(format!("{}-wal", path));
                let _ = fs::remove_file(format!("{}-shm", path));
                setup_registry(black_box(RegistryBackend::Sqlite), black_box(size), path);
                let _ = fs::remove_file(path);
                let _ = fs::remove_file(format!("{}-wal", path));
                let _ = fs::remove_file(format!("{}-shm", path));
            });
        });
    }

    group.finish();
}

/// Benchmark: Load entire registry
fn bench_registry_load(c: &mut Criterion) {
    let mut group = c.benchmark_group("registry_load");

    for size in [100, 1000].iter() {
        // Setup JSON registry
        let json_path = "bench_load_json.json";
        setup_registry(RegistryBackend::Json, *size, json_path);

        group.bench_with_input(BenchmarkId::new("json", size), size, |b, _size| {
            b.iter(|| {
                let store = open_store(black_box(RegistryBackend::Json), Path::new(json_path)).unwrap();
                let _reg = store.load().unwrap();
            });
        });

        fs::remove_file(json_path).ok();

        // Setup SQLite registry
        let sqlite_path = "bench_load_sqlite.db";
        setup_registry(RegistryBackend::Sqlite, *size, sqlite_path);

        group.bench_with_input(BenchmarkId::new("sqlite", size), size, |b, _size| {
            b.iter(|| {
                let store = open_store(black_box(RegistryBackend::Sqlite), Path::new(sqlite_path)).unwrap();
                let _reg = store.load().unwrap();
            });
        });

        fs::remove_file(sqlite_path).ok();
        fs::remove_file(format!("{}-wal", sqlite_path)).ok();
        fs::remove_file(format!("{}-shm", sqlite_path)).ok();
    }

    group.finish();
}

/// Benchmark: Find entry by hash
fn bench_registry_find(c: &mut Criterion) {
    let mut group = c.benchmark_group("registry_find");

    let size = 1000;

    // Setup JSON registry
    let json_path = "bench_find_json.json";
    setup_registry(RegistryBackend::Json, size, json_path);
    let json_store = open_store(RegistryBackend::Json, Path::new(json_path)).unwrap();

    // Middle entry
    let middle_entry = create_mock_entry(size / 2);

    group.bench_function("json", |b| {
        b.iter(|| {
            let _result = json_store.find_by_hashes(
                black_box(&middle_entry.manifest_hash),
                black_box(&middle_entry.proof_hash)
            ).unwrap();
        });
    });

    fs::remove_file(json_path).ok();

    // Setup SQLite registry
    let sqlite_path = "bench_find_sqlite.db";
    setup_registry(RegistryBackend::Sqlite, size, sqlite_path);
    let sqlite_store = open_store(RegistryBackend::Sqlite, Path::new(sqlite_path)).unwrap();

    group.bench_function("sqlite", |b| {
        b.iter(|| {
            let _result = sqlite_store.find_by_hashes(
                black_box(&middle_entry.manifest_hash),
                black_box(&middle_entry.proof_hash)
            ).unwrap();
        });
    });

    fs::remove_file(sqlite_path).ok();
    fs::remove_file(format!("{}-wal", sqlite_path)).ok();
    fs::remove_file(format!("{}-shm", sqlite_path)).ok();

    group.finish();
}

/// Benchmark: List all entries
fn bench_registry_list(c: &mut Criterion) {
    let mut group = c.benchmark_group("registry_list");

    for size in [100, 1000].iter() {
        // Setup JSON registry
        let json_path = "bench_list_json.json";
        setup_registry(RegistryBackend::Json, *size, json_path);
        let json_store = open_store(RegistryBackend::Json, Path::new(json_path)).unwrap();

        group.bench_with_input(BenchmarkId::new("json", size), size, |b, _size| {
            b.iter(|| {
                let _list = json_store.list().unwrap();
            });
        });

        fs::remove_file(json_path).ok();

        // Setup SQLite registry
        let sqlite_path = "bench_list_sqlite.db";
        setup_registry(RegistryBackend::Sqlite, *size, sqlite_path);
        let sqlite_store = open_store(RegistryBackend::Sqlite, Path::new(sqlite_path)).unwrap();

        group.bench_with_input(BenchmarkId::new("sqlite", size), size, |b, _size| {
            b.iter(|| {
                let _list = sqlite_store.list().unwrap();
            });
        });

        fs::remove_file(sqlite_path).ok();
        fs::remove_file(format!("{}-wal", sqlite_path)).ok();
        fs::remove_file(format!("{}-shm", sqlite_path)).ok();
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_registry_insert,
    bench_registry_load,
    bench_registry_find,
    bench_registry_list
);
criterion_main!(benches);
