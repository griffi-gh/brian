use std::io::{self, Write};
use super::{Brainfuck, Opcode, MEMORY_SIZE};

impl Brainfuck {
  ///Run brainfuck program after compilation
  #[inline]
  pub fn run(&mut self) {
    let program_len = self.program.len();
    let program = &self.program[..];
    let memory = &mut self.state.memory;
    let pointer = &mut self.state.pointer;
    let mut program_counter = 0;
    loop {
      if program_counter >= program_len { break }
      let op = &program[program_counter];
      match op {
        Opcode::Increment(rel_pos, rel_val) => {
          let pos = pointer.wrapping_add_signed(*rel_pos);
          memory[pos & MEMORY_SIZE] = memory[pos & MEMORY_SIZE].wrapping_add(*rel_val as u8);
        },
        Opcode::Set(rel_pos, val) => {
          let pos = pointer.wrapping_add_signed(*rel_pos);
          memory[pos & MEMORY_SIZE] = *val;
        },
        Opcode::MovePointer(rel_pos) => {
          *pointer = pointer.wrapping_add_signed(*rel_pos);
        },
        Opcode::LoopStart(end) => {
          if memory[*pointer & MEMORY_SIZE] == 0 {
            program_counter = *end;
          }
        },
        Opcode::LoopEnd(start) => {
          if memory[*pointer & MEMORY_SIZE] != 0 {
            program_counter = *start;
          }
        },
        Opcode::ScanZero(direction) => {
          while memory[*pointer & MEMORY_SIZE] != 0 {
            *pointer = pointer.wrapping_add_signed(*direction);
          }
        }
        Opcode::Output(rel_pos) => {
          let pos = pointer.wrapping_add_signed(*rel_pos);
          io::stdout().write(&[memory[pos & MEMORY_SIZE]]).unwrap();
        },
        Opcode::Input(_) => todo!(),
        Opcode::Eof => break,
      }
      program_counter += 1;
    }
  }
}
