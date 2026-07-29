use grainpit::markov::Markov;

use criterion::{Criterion, Throughput, criterion_group, criterion_main};

fn bench(c: &mut Criterion) {
    let markov = Markov::new();

    c.bench_function("gen_html", |b| b.iter(|| markov.gen_html()));

    let mut group = c.benchmark_group("generate");
    let length = 2048;
    group.throughput(Throughput::Elements(length as u64));
    group.bench_function("html-chain", |b| {
        b.iter(|| markov.html_chain.generate(length))
    });
    group.bench_function("config-chain", |b| {
        b.iter(|| markov.config_chain.generate(length))
    });
    group.bench_function("url-chain", |b| {
        b.iter(|| markov.url_chain.generate(length))
    });
    group.bench_function("url-name-chain", |b| {
        b.iter(|| markov.url_name_chain.generate(length))
    });
    group.finish();
}

criterion_group!(benches, bench);
criterion_main!(benches);
