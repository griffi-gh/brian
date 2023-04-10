use std::collections::HashMap;

use arrayvec::ArrayVec;
use itertools::Itertools;
#[cfg(feature = "serde")]
use serde::{Serialize, Deserialize};

const MEMORY_SIZE: usize = 0xffff;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Opcode {
  IncrementRelative(isize, isize),
  SetRelative(isize, u8),
  MovePointer(isize),
  LoopStart,
  LoopEnd,
  LinkedLoopStart(usize),
  LinkedLoopEnd(usize),
  Output,
  Input,
  Move(usize, ArrayVec::<usize, 16>),
  Eof,
}
impl From<Token> for Opcode {
  fn from(value: Token) -> Self {
    match value {
      Token::Increment => Self::IncrementRelative(0, 1),
      Token::Decrement => Self::IncrementRelative(0, -1),
      Token::MovePointerLeft => Self::MovePointer(-1),
      Token::MovePointerRight => Self::MovePointer(1),
      Token::LoopStart => Self::LoopStart,
      Token::LoopEnd => Self::LoopEnd,
      Token::Output => Self::Output,
      Token::Input => Self::Input,
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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
    {
      #[derive(Clone, Copy, PartialEq, Eq)]
      enum BlockEffect {
        Increment(isize),
        Set(u8)
      }
      let mut ptr_offset: isize = 0;
      let mut block_effects: HashMap<isize, BlockEffect> = HashMap::new();
      //TODO: unlink loops
      //TODO: check for eof token and add it
      //TODO: recursive block compilation
      //Optimize increments/ptr movements
      let mut index = 0;
      while index < ops.len() {
        let op = &ops[index];
        index += 1;
        match op {
          Opcode::IncrementRelative(offset, increment) => {
            let existing_effect = block_effects.get_mut(&ptr_offset);
            match existing_effect {
              Some(BlockEffect::Increment(effect)) => {
                *effect += increment;
              },
              Some(BlockEffect::Set(effect)) => {
                //TODO casting isize to i8 can cause unexpected behaviour here!
                *effect = effect.wrapping_add_signed(*increment as i8);
              },
              None => {
                block_effects.insert(offset + ptr_offset, BlockEffect::Increment(*increment));
              }
            }
            //block_effects.insert(offset + ptr_offset, existing_value + increment);
          }
          Opcode::MovePointer(diff) => {
            ptr_offset += diff;
          },
          Opcode::Eof | Opcode::Input | Opcode::Output | Opcode::LoopStart | Opcode::LoopEnd => {
            //Detect [-]/[+] loops (TODO compute block effects instead!)
            if let Opcode::LoopStart = op {
              if let Opcode::LoopEnd = ops[index + 1] {
                if let Opcode::IncrementRelative(pos, value) = ops[index] {
                  if pos == 0 && value.abs() % 2 == 1 {
                    index += 2;
                    block_effects.insert(ptr_offset, BlockEffect::Set(0));
                    continue
                  }
                }
              }
            }
            //commit increments and pointer movements
            for (relative_pos, effect) in block_effects.iter().sorted_by_key(|x| *x.0) {
              output_ops.push(match effect {
                BlockEffect::Increment(increment) => {
                  if *increment == 0 { continue }
                  Opcode::IncrementRelative(*relative_pos, *increment)
                },
                BlockEffect::Set(value) => Opcode::SetRelative(*relative_pos, *value),
              })
            }
            if ptr_offset > 0 {
              output_ops.push(Opcode::MovePointer(ptr_offset));
            }
            //TODO: figure out a way to avoid clone (low prio; opt times arent that important)
            output_ops.push(op.clone()); 
            block_effects.clear();
          },
          _ => todo!()
        }
      }
    }

    //Link loops
    {
      let mut stack: Vec<usize> = Vec::new();
      for index in 0..output_ops.len() {
        //This is very hacky
        let (output_ops_before, op) = output_ops.split_at_mut(index);
        let op = &mut op[0];
        match op {
          Opcode::LoopStart => {
            stack.push(index);
          },
          Opcode::LoopEnd => {
            let start_index = stack.pop().expect("Unexpected loop end");
            output_ops_before[start_index] = Opcode::LinkedLoopStart(index);
            *op = Opcode::LinkedLoopEnd(start_index);
          }
          _ => ()
        }
      }
      assert!(stack.is_empty(), "Unclosed loop");
    }

    output_ops
  }

  /// Compile a brainfuck source
  pub fn compile(&mut self, code: &str) {
    let mut ops: Vec<Opcode> = brainfuck_tokens(code).map(Opcode::from).collect();
    ops.push(Opcode::Eof);
    self.program = Self::optimize(ops);
    println!("{:?}", &self.program);
  }

  ///Run optimized bf source
  pub fn run(&mut self) {
    todo!();
  }
}
impl Default for Brainfuck {
  fn default() -> Self {
    Self::new()
  }
}
