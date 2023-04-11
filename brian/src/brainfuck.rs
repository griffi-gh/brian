use std::{io::{self, Write}, collections::HashMap};

//Warning: please only use values that can be used as bitmasks!
const MEMORY_SIZE: usize = 0xffff;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Token {
  Increment,
  Decrement,
  MovePointerLeft,
  MovePointerRight,
  LoopStart,
  LoopEnd,
  Output,
  Input
}

#[repr(u8)]
#[derive(Clone, Debug)]
pub enum Opcode {
  Increment(isize, isize),
  Set(isize, u8),
  MovePointer(isize),
  LoopStart(usize),
  LoopEnd(usize),
  Output(isize),
  Input(isize),
  ScanZero(isize),
  //Move(usize, ArrayVec::<usize, 16>),
  Eof,
}
impl From<Token> for Opcode {
  fn from(value: Token) -> Self {
    match value {
      Token::Increment => Self::Increment(0, 1),
      Token::Decrement => Self::Increment(0, -1),
      Token::MovePointerLeft => Self::MovePointer(-1),
      Token::MovePointerRight => Self::MovePointer(1),
      Token::LoopStart => Self::LoopStart(0),
      Token::LoopEnd => Self::LoopEnd(0),
      Token::Output => Self::Output(0),
      Token::Input => Self::Input(0),
    }
  }
}

fn brainfuck_tokens(code: &str) -> impl Iterator<Item=Token> + '_ {
  code.chars().filter_map(|x| match x {
    '+' => Some(Token::Increment),
    '-' => Some(Token::Decrement),
    '<' => Some(Token::MovePointerLeft),
    '>' => Some(Token::MovePointerRight),
    '[' => Some(Token::LoopStart),
    ']' => Some(Token::LoopEnd),
    '.' => Some(Token::Output),
    ',' => Some(Token::Input),
    _ => None,
  })
}

#[derive(Clone, Copy)]
pub struct BrainfuckState {
  pub memory: [u8; MEMORY_SIZE],
  pub pointer: usize,
}
impl BrainfuckState {
  pub fn new() -> Self {
    Self {
      memory: [0; MEMORY_SIZE],
      pointer: 0,
    }
  }
}
impl Default for BrainfuckState {
  fn default() -> Self {
    Self::new()
  }
}

/// Brainfuck interpreter
#[derive(Clone)]
pub struct Brainfuck {
  state: BrainfuckState,
  program: Vec<Opcode>
}
impl Brainfuck {
  /// Create a new brainfuck interpreter
  #[inline]
  pub fn new() -> Self {
    Self {
      state: BrainfuckState::new(),
      program: Vec::new(),
    }
  }

  /// Create  a new brainfuck interpreter using existing state
  #[inline]
  pub fn new_with_state(state: BrainfuckState) -> Self {
    Self {
      state: state,
      program: Vec::new(),
    }
  }

  /// Get an immutable reference to the interpreter state
  #[inline(always)]
  pub fn state(&self) -> &BrainfuckState {
    &self.state
  }

  /// Get a mutable reference to the interpreter state
  #[inline(always)]
  pub fn state_mut(&mut self) -> &mut BrainfuckState {
    &mut self.state
  }

  fn optimize(ops: Vec<Opcode>) -> Vec<Opcode> {
    let mut output_ops: Vec<Opcode> = Vec::new();

    //TODO: check for eof token and add it
    //TODO: recursive block compilation

    //Optimize increments/ptr movements
    {
      #[derive(Clone, Copy, PartialEq, Eq)]
      enum BlockEffect {
        Increment(isize),
        Set(u8),
      }
      trait BlockCommit {
        fn opcode(&self) -> Option<Opcode>;
        fn commit(&self, ops: &mut Vec<Opcode>);
      }
      impl BlockCommit for (&isize, &BlockEffect) {
        fn opcode(&self) -> Option<Opcode> {
          match self.1 {
            BlockEffect::Increment(increment) => {
              if *increment == 0 { return None }
              Some(Opcode::Increment(*self.0, *increment))
            },
            BlockEffect::Set(value) => {
              Some(Opcode::Set(*self.0, *value))
            },
          }
        }
        fn commit(&self, ops: &mut Vec<Opcode>) {
          if let Some(op) = self.opcode() {
            ops.push(op);
          }
        }
      }
      let mut block_effects: HashMap<isize, BlockEffect> = HashMap::new();
      let mut ptr_offset: isize = 0;
      let mut index = 0;

      'opt: while index < ops.len() {
        let op = &ops[index];
        index += 1;
        match op {
          Opcode::Increment(offset, increment) => {
            let existing_effect = block_effects.get_mut(&ptr_offset);
            match existing_effect {
              Some(BlockEffect::Increment(effect)) => {
                *effect += increment;
              },
              Some(BlockEffect::Set(effect)) => {
                //TODO: casting isize to i8 can cause unexpected behaviour here! (but it's not likely to break)
                *effect = effect.wrapping_add_signed(*increment as i8);
              },
              None => {
                block_effects.insert(offset + ptr_offset, BlockEffect::Increment(*increment));
              }
            }
            //block_effects.insert(offset + ptr_offset, existing_value + increment);
          }
          Opcode::MovePointer(diff) => {
            ptr_offset += *diff;
          },
          Opcode::Output(out_offset) | Opcode::Input(out_offset) => {
            //THIS IS EXPERIMENTAL!
            //Partial commit: commit only operations related to the current cell
            //TODO: maybe do not remove the effect if its "Set"? (probably special value should be used to indicate that the set is already committed?)
            let relative_pos = &(ptr_offset + out_offset);
            if let Some(ref effect) = block_effects.remove(relative_pos) {
              (relative_pos, effect).commit(&mut output_ops);
            }
            output_ops.push(match op {
              Opcode::Output(_) => Opcode::Output(*relative_pos),
              Opcode::Input(_) => Opcode::Input(*relative_pos),
              _ => unreachable!()
            })
          }
          Opcode::Eof | Opcode::LoopStart(_) | Opcode::LoopEnd(_) => {
            //Detect [-]/[+] loops 
            //TODO: compute block effects instead!
            //TODO: at least compute pointer movement? (to allow [<->]) (recursive comp is preferable)
            if let Opcode::LoopStart(_) = op {
              if let Opcode::LoopEnd(_) = ops[index + 1] {
                if let Opcode::Increment(pos, value) = ops[index] {
                  if pos == 0 && value.abs() % 2 == 1 {
                    index += 2;
                    block_effects.insert(ptr_offset, BlockEffect::Set(0));
                    continue
                  }
                }
              }
            }
            //commit increments and pointer movements
            for effect in &block_effects {
              effect.commit(&mut output_ops);
            }
            block_effects.clear();
            //commit pointer movements
            if ptr_offset != 0 {
              output_ops.push(Opcode::MovePointer(ptr_offset));
              ptr_offset = 0;
            }

            //Detect zero-scan loops 
            'outer: {
              if let Opcode::LoopStart(end) = op {
                let mut mov_sum = 0;
                for op in &ops[index..*end] {
                  match op {
                    Opcode::MovePointer(mov) => {
                      mov_sum += *mov;
                    },
                    _ => break 'outer
                  }
                }
                if mov_sum == 0 {
                  break 'outer
                }
                output_ops.push(Opcode::ScanZero(mov_sum)); 
                index = end + 1;
                continue 'opt
              }
            }

            //Push original opcode
            output_ops.push(op.clone()); 
          },
          _ => todo!()
        }
      }
    }

    output_ops
  }

  fn link_loops(ops: &mut Vec<Opcode>) {
    let mut stack: Vec<usize> = Vec::new();
    for index in 0..ops.len() {
      //This is very hacky
      let (output_ops_before, op) = ops.split_at_mut(index);
      let op = &mut op[0];
      match op {
        Opcode::LoopStart(_) => {
          stack.push(index);
        },
        Opcode::LoopEnd(_) => {
          let start_index = stack.pop().expect("Unexpected loop end");
          output_ops_before[start_index] = Opcode::LoopStart(index);
          *op = Opcode::LoopEnd(start_index);
        }
        _ => ()
      }
    }
    assert!(stack.is_empty(), "Unclosed loop");
  }

  fn parse(code: &str) -> Vec<Opcode> {
    let mut ops: Vec<Opcode> = brainfuck_tokens(code).map(Opcode::from).collect();
    ops.push(Opcode::Eof);
    ops
  }

  /// Compile brainfuck source code
  pub fn compile(&mut self, code: &str) {
    let mut ops = Self::parse(code);
    Self::link_loops(&mut ops);
    let mut ops = Self::optimize(ops);
    Self::link_loops(&mut ops);
    self.program = ops;
  }

  /// Compile brainfuck source code without applying any optimizations
  pub fn compile_without_optimizations(&mut self, code: &str) {
    let mut ops = Self::parse(code);
    Self::link_loops(&mut ops);
    self.program = ops;
  }

  pub fn _debug(&self) {
    println!("{:?}", &self.program);
  }

  ///Run brainfuck program after compilation
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
impl Default for Brainfuck {
  fn default() -> Self {
    Self::new()
  }
}
