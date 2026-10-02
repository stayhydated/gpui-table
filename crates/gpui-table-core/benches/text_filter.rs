use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use gpui_table_core::filter::TextValue;
use std::hint::black_box;

fn text_filter(c: &mut Criterion) {
    let mut group = c.benchmark_group("text_filter/scan");

    for row_count in [100, 1_000, 10_000] {
        for (corpus, pattern, matching, missing) in [
            ("ascii", "AbCdEfGh", "cDe", "xyz"),
            ("unicode", "É東京", "é東京", "不存在"),
        ] {
            // Both patterns are 8 bytes: fixtures have exactly 128 bytes per row.
            let rows: Vec<_> = (0..row_count).map(|_| pattern.repeat(16)).collect();
            assert!(rows.iter().all(|row| row.len() == 128));
            group.throughput(Throughput::Bytes((row_count * 128) as u64));

            for (query_name, query) in [("match", matching), ("miss", missing)] {
                let filter = TextValue::from(query);
                assert_eq!(
                    rows.iter().filter(|row| filter.matches(row)).count(),
                    if query_name == "match" { row_count } else { 0 },
                );
                group.bench_with_input(
                    BenchmarkId::new(format!("{corpus}/{query_name}"), row_count),
                    &rows,
                    |b, rows| {
                        b.iter(|| {
                            let matches = black_box(rows)
                                .iter()
                                .filter(|row| black_box(&filter).matches(black_box(row)))
                                .count();
                            black_box(matches)
                        });
                    },
                );
            }
        }
    }

    group.finish();
}

criterion_group!(benches, text_filter);
criterion_main!(benches);
