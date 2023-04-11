use brian::Brainfuck;

fn main() {
  let mut bf = Brainfuck::new();
  bf.compile(include_str!("../../malderbrot.b.txt"));
  bf._debug();
  bf.run();
}
