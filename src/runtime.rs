use std::rc::Rc;

use ordered_float::OrderedFloat;

const U16_CAPCITY: usize = u16::MAX as usize + 1;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Error {
  NotYetImplemented,
  CantCastToNum,
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Num {
  Int(i64),
  Float(OrderedFloat<f64>),
}

impl Num {
  pub fn floor(&self) -> i64 {
    match self {
      Num::Int(i) => *i,
      Num::Float(f) => f64::from(*f) as i64,
    }
  }
  pub fn add(a: Num, b: &Num) -> Num {
    match (a, b) {
      (Num::Int(a), Num::Int(b)) => Num::Int(a + b),
      (Num::Float(a), Num::Float(b)) => Num::Float(a + b),
      (Num::Int(a), Num::Float(b)) => {
        Num::Float(OrderedFloat::from(a as f64) + b)
      }
      (Num::Float(a), Num::Int(b)) => Num::Float(a + (*b as f64)),
    }
  }
  pub fn multiply(a: Num, b: &Num) -> Num {
    match (a, b) {
      (Num::Int(a), Num::Int(b)) => Num::Int(a * b),
      (Num::Float(a), Num::Float(b)) => Num::Float(a * b),
      (Num::Int(a), Num::Float(b)) => {
        Num::Float(OrderedFloat::from(a as f64) * b)
      }
      (Num::Float(a), Num::Int(b)) => Num::Float(a * (*b as f64)),
    }
  }
  pub fn min(a: Num, b: &Num) -> Num {
    match (a, b) {
      (Num::Int(a), Num::Int(b)) => Num::Int(a.min(*b)),
      (Num::Float(a), Num::Float(b)) => Num::Float(a.min(*b)),
      (Num::Int(a), Num::Float(b)) => {
        let b_derefed = *b;
        if (a as f64) <= f64::from(b_derefed) {
          Num::Int(a)
        } else {
          Num::Float(b_derefed)
        }
      }
      (Num::Float(a), Num::Int(b)) => {
        let b_derefed = *b;
        if (b_derefed as f64) <= f64::from(a) {
          Num::Int(b_derefed)
        } else {
          Num::Float(a)
        }
      }
    }
  }
  pub fn max(a: Num, b: &Num) -> Num {
    match (a, b) {
      (Num::Int(a), Num::Int(b)) => Num::Int(a.max(*b)),
      (Num::Float(a), Num::Float(b)) => Num::Float(a.max(*b)),
      (Num::Int(a), Num::Float(b)) => {
        let b_derefed = *b;
        if (a as f64) >= f64::from(b_derefed) {
          Num::Int(a)
        } else {
          Num::Float(b_derefed)
        }
      }
      (Num::Float(a), Num::Int(b)) => {
        let b_derefed = *b;
        if (b_derefed as f64) >= f64::from(a) {
          Num::Int(b_derefed)
        } else {
          Num::Float(a)
        }
      }
    }
  }
  pub fn as_float(&self) -> OrderedFloat<f64> {
    match self {
      Num::Int(i) => OrderedFloat::from(*i as f64),
      Num::Float(f) => *f,
    }
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Value {
  Nil,
  Num(Num),
}

impl Value {
  fn as_num(&self) -> Result<Num> {
    match self {
      Value::Num(n) => Ok(n.clone()),
      Value::Nil => Ok(Num::Int(0)),
    }
  }
  fn description(&self) -> String {
    match self {
      Value::Nil => "nil".to_string(),
      Value::Num(n) => match n {
        Num::Int(i) => i.to_string(),
        Num::Float(f) => {
          let mut s = f.to_string();
          if !s.contains('.') {
            s += ".";
          }
          s
        }
      },
    }
  }
}

pub enum Instruction {
  Clear(u16),
  Const(Value, u16),
  Add(u16, u16, u16),
  Multiply(u16, u16, u16),
}

struct EvaluationState {
  memory: Vec<Value>,
  registers: [*const Value; U16_CAPCITY],
}

impl EvaluationState {
  fn new() -> Self {
    Self {
      memory: vec![],
      registers: [std::ptr::null(); U16_CAPCITY],
    }
  }
  fn display_registers(&self) -> String {
    (0..U16_CAPCITY).into_iter().fold(String::new(), |s, i| {
      let register_pointer = self.registers[i];
      if register_pointer.is_null() {
        s
      } else {
        let value = unsafe { &*register_pointer };
        s + &format!("{}: {}\n", i, value.description())
      }
    })
  }
}

pub fn evaluate(instructions: Vec<Instruction>) -> Result<()> {
  let mut state = EvaluationState::new();
  for instruction in instructions {
    match instruction {
      Instruction::Clear(register) => {
        state.registers[register as usize] = std::ptr::null()
      }
      Instruction::Const(value, register) => {
        state.memory.push(value);
        let x = &state.memory[state.memory.len() - 1] as *const Value;
        state.registers[register as usize] = x;
      }
      Instruction::Add(addend_register_1, addend_register_2, sum_register) => unsafe {
        let addend_1 = &*state.registers[addend_register_1 as usize];
        let addend_2 = &*state.registers[addend_register_2 as usize];
        let sum = Num::add(addend_1.as_num()?, &addend_2.as_num()?);
        state.memory.push(Value::Num(sum));
        state.registers[sum_register as usize] =
          &state.memory[state.memory.len() - 1] as *const Value;
      },
      Instruction::Multiply(
        multiplicand_register_1,
        multiplicand_register_2,
        product_register,
      ) => unsafe {
        let multiplicand_1 =
          &*state.registers[multiplicand_register_1 as usize];
        let multiplicand_2 =
          &*state.registers[multiplicand_register_2 as usize];
        let sum =
          Num::multiply(multiplicand_1.as_num()?, &multiplicand_2.as_num()?);
        state.memory.push(Value::Num(sum));
        state.registers[product_register as usize] =
          &state.memory[state.memory.len() - 1] as *const Value;
      },
    }
  }
  println!("registers:\n{}", state.display_registers());
  Ok(())
}
