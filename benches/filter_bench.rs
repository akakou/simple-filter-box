use criterion::{black_box, BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use simple_filter_box::binary_search::redb::RedbFilterBox;
use simple_filter_box::bloom_filter::bloom_filter::BloomFilterBox;
use simple_filter_box::core::{FilterBox, bytes::Long};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

const INSERT_COUNTS: &[usize] = &[100, 1_000];

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

fn bench_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("insert");
    group.sample_size(10);
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(3));

    for &count in INSERT_COUNTS {
        let tokens = make_tokens(count, 0x5EED_u64);

        group.bench_with_input(BenchmarkId::new("bloom", count), &tokens, |b, tokens| {
            b.iter_batched(
                || BloomFilterBox::with_rate(tokens.len(), 0.01),
                |mut filter| {
                    for token in tokens {
                        filter.insert(black_box(token)).unwrap();
                    }
                },
                BatchSize::SmallInput,
            );
        });

        group.bench_with_input(BenchmarkId::new("redb", count), &tokens, |b, tokens| {
            b.iter_batched(
                || {
                    let path = temp_db_path("insert");
                    let filter = RedbFilterBox::open(&path).expect("open redb");
                    (filter, path)
                },
                |(mut filter, path)| {
                    for token in tokens {
                        filter.insert(black_box(token)).unwrap();
                    }
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
    group.sample_size(10);
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(3));

    for &count in INSERT_COUNTS {
        let existing = make_tokens(count, 0xBADC0DE_u64);
        let missing = make_tokens(count, 0xD15EA5E_u64);

        group.bench_with_input(BenchmarkId::new("bloom_miss", count), &missing, |b, tokens| {
            let mut filter = BloomFilterBox::with_rate(tokens.len(), 0.01);
            for token in &existing {
                filter.insert(token).unwrap();
            }
            b.iter(|| {
                for token in tokens {
                    black_box(filter.test(black_box(token)));
                }
            });
        });

        group.bench_with_input(BenchmarkId::new("redb_miss", count), &missing, |b, tokens| {
            let path = temp_db_path("test");
            let mut filter = RedbFilterBox::open(&path).expect("open redb");
            for token in &existing {
                filter.insert(token).unwrap();
            }
            b.iter(|| {
                for token in tokens {
                    black_box(filter.test(black_box(token)));
                }
            });
            drop(filter);
            let _ = std::fs::remove_file(path);
        });
    }

    group.finish();
}

criterion_group!(benches, bench_insert, bench_test);
criterion_main!(benches);
