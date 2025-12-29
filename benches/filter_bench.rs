use criterion::{black_box, BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use simple_filter_box::binary_search::redb::RedbFilter;
use simple_filter_box::bloom_filter::bloom_filter::BloomFilterBox;
use simple_filter_box::core::{FilterBox, bytes::Long};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use std::{io, path::Path};
use redb::{Database, TableDefinition};

// const PRELOAD_COUNTS: &[usize] = &[1, 10, 100, 1_000];//&[100, 1_000, 10_000, 100_000, 1_000_000];
// const PRELOAD_COUNTS: &[usize] = &[100, 1_000, 10_000, 100_000, 1_000_000];
const PRELOAD_COUNTS: &[usize] = &[100_000, 200_000, 300_000, 400_000, 500_000, 600_000, 700_000, 800_000, 900_000];
const TOKEN_TABLE: TableDefinition<&[u8], u8> = TableDefinition::new("dedup_tokens");
const SAMPLE_SIZE: usize = 100;

fn make_tokens(count: usize, seed: u64) -> Vec<Long> {
    let mut tokens = Vec::with_capacity(count);
    let mut state = seed;
    for _ in 0..count {
        let mut token = [0u8; 32];
        for chunk in token.chunks_exact_mut(8) {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1);
            chunk.copy_from_slice(&state.to_le_bytes());
        }
        tokens.push(token);
    }
    tokens
}

fn temp_db_path(label: &str) -> PathBuf {
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    let mut path = std::env::temp_dir();
    let id = COUNTER.fetch_add(1, Ordering::Relaxed);
    path.push(format!(
        "simple_filter_box_bench_{}_{}_{}.redb",
        std::process::id(),
        label,
        id
    ));
    path
}

fn open_database(path: &Path) -> io::Result<Database> {
    let result = if path.exists() {
        Database::open(path)
    } else {
        Database::create(path)
    };
    result.map_err(|err| io::Error::new(io::ErrorKind::Other, err))
}

fn insert_batch(path: &Path, tokens: &[Long]) -> io::Result<()> {
    let db = open_database(path)?;
    let write_txn = db
        .begin_write()
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
    let mut table = write_txn
        .open_table(TOKEN_TABLE)
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;

    for token in tokens {
        table
            .insert(&token[..], &1u8)
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
    }

    drop(table);
    write_txn
        .commit()
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
    Ok(())
}

fn bench_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("insert");
    group.sample_size(SAMPLE_SIZE);
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(3));

    for &count in PRELOAD_COUNTS {
        let preload = make_tokens(count, 0x5EED_u64);
        let insert_tokens = make_tokens(1, 0xBEE5_u64);

        group.bench_with_input(BenchmarkId::new("bloom", count), &count, |b, &_count| {
            b.iter_batched(
                || {
                    let mut filter = BloomFilterBox::with_rate(count + 1, 0.01);
                    for token in &preload {
                        filter.insert(token).unwrap();
                    }
                    filter
                },
                |mut filter| {
                    let token = &insert_tokens[0];
                    filter.insert(black_box(token)).unwrap();
                },
                BatchSize::SmallInput,
            );
        });

        group.bench_with_input(BenchmarkId::new("redb", count), &count, |b, &_count| {
            b.iter_batched(
                || {
                    let path = temp_db_path("insert");
                    insert_batch(&path, &preload).unwrap();
                    let mut filter = RedbFilter::open(&path).expect("open redb");
                    (filter, path)
                },
                |(mut filter, path)| {
                    let token = &insert_tokens[0];
                    filter.insert(black_box(token)).unwrap();
                    drop(filter);
                    let _ = std::fs::remove_file(path);
                },
                BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

fn bench_test(c: &mut Criterion) {
    let mut group = c.benchmark_group("test");
    group.sample_size(SAMPLE_SIZE);
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(3));

    for &count in PRELOAD_COUNTS {
        let existing = make_tokens(count, 0xBADC0DE_u64);
        let missing = make_tokens(1, 0xD15EA5E_u64);

        group.bench_with_input(BenchmarkId::new("bloom_miss", count), &count, |b, &_count| {
            let mut filter = BloomFilterBox::with_rate(count + 1, 0.01);
            for token in &existing {
                filter.insert(token).unwrap();
            }
            b.iter(|| {
                let token = &missing[0];
                black_box(filter.test(black_box(token)));
            });
        });

        group.bench_with_input(BenchmarkId::new("redb_miss", count), &count, |b, &_count| {
            let path = temp_db_path("test");
            insert_batch(&path, &existing).unwrap();
            let filter = RedbFilter::open(&path).expect("open redb");
            b.iter(|| {
                let token = &missing[0];
                black_box(filter.test(black_box(token)));
            });
            drop(filter);
            let _ = std::fs::remove_file(path);
        });
    }

    group.finish();
}

#[cfg(feature = "bench-insert-and-test")]
fn bench_insert_and_test(c: &mut Criterion) {
    let mut group = c.benchmark_group("insert_and_test");
    group.sample_size(SAMPLE_SIZE);
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(3));

    for &count in PRELOAD_COUNTS {
        let preload = make_tokens(count, 0xABAD1DEA_u64);
        let insert_tokens = make_tokens(1, 0x1CEB00DA_u64);

        group.bench_with_input(BenchmarkId::new("bloom", count), &count, |b, &_count| {
            b.iter_batched(
                || {
                    let mut filter = BloomFilterBox::with_rate(count + 1, 0.01);
                    for token in &preload {
                        filter.insert(token).unwrap();
                    }
                    filter
                },
                |mut filter| {
                    let token = &insert_tokens[0];
                    filter.insert(black_box(token)).unwrap();
                    black_box(filter.test(black_box(token)));
                },
                BatchSize::SmallInput,
            );
        });

        group.bench_with_input(BenchmarkId::new("redb", count), &count, |b, &_count| {
            b.iter_batched(
                || {
                    let path = temp_db_path("insert_test");
                    insert_batch(&path, &preload).unwrap();
                    let mut filter = RedbFilter::open(&path).expect("open redb");
                    (filter, path)
                },
                |(mut filter, path)| {
                    let token = &insert_tokens[0];
                    filter.insert(black_box(token)).unwrap();
                    black_box(filter.test(black_box(token)));
                    drop(filter);
                    let _ = std::fs::remove_file(path);
                },
                BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

#[cfg(feature = "bench-insert-and-test")]
criterion_group!(benches, bench_insert, bench_test, bench_insert_and_test);
#[cfg(not(feature = "bench-insert-and-test"))]
criterion_group!(benches, bench_insert, bench_test);
criterion_main!(benches);
