use brian::Brainfuck;
use criterion::{Criterion, Bencher, criterion_group, criterion_main, black_box};

const BENCH_SRC: &str = include_str!("./../../malderbrot.b.txt");

fn bench_bf_compile(b: &mut Bencher) {
  let mut bf = Brainfuck::new();
  b.iter(|| {
    black_box(bf.compile(BENCH_SRC));
  });
}

// fn bench_bf_run(b: &mut Bencher) {
//   let mut bf = Brainfuck::new();
//   bf.compile(code);
// }

fn criterion_benchmark(c: &mut Criterion) {
  c.bench_function("bench_bf_compile", bench_bf_compile);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
