use grainpit::image::gen_image;

use criterion::{Criterion, criterion_group, criterion_main};

fn bench(c: &mut Criterion) {
    c.bench_function("gen_image", |b| b.iter(|| gen_image()));
}

criterion_group!(benches, bench);
criterion_main!(benches);
