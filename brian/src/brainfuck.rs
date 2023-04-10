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
  SetRelative(isize, usize),
  MovePointer(isize),
  LoopStart,
  LoopEnd,
  LinkedLoopStart(usize),
  LinkedLoopEnd(usize),
  Output,
  Input,
  Move(usize, ArrayVec::<usize, 16>),
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


  /// Compile a brainfuck source
  pub fn compile(&mut self, code: &str) {
    let ops: Vec<Opcode> = brainfuck_tokens(code).map(Opcode::from).collect();
    let mut ptr_offset = 0;
    let mut prev_op = None;
    for op in ops {
      prev_op = Some(op);
    }
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
