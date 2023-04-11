use brian::Brainfuck;
use std::time::Instant;

fn main() {
  let mut bf = Brainfuck::new();
  bf.compile(include_str!("../../malderbrot.b.txt"));
  bf._debug();
  let start = Instant::now();
  bf.run();
  let elapsed_ms = start.elapsed().as_secs_f64();
  println!("Took {} seconds", elapsed_ms);
}
