use brian::Brainfuck;

fn main() {
  let mut bf = Brainfuck::new();
  bf.compile("[-]++>>+>+.<+");
}
