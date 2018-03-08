struct State {
    stack: Vec<i32>,
    vars: Vec<i32>,
    bytecode: Vec<Opcode>,
    index: usize,
}

impl State {
    fn new(bytecode: Vec<Opcode>) -> Self {
        State {
            stack: Vec::new(),
            vars: Vec::new(),
            bytecode,
            index: 0,
        }
    }

    fn step(&mut self) -> bool {
        let op = self.bytecode[self.index];
        self.index = self.index + 1;
        match op {
            Opcode::Push(val) => self.stack.push(val),
            Pop => {
                self.stack.pop();
            }
        }
        self.index < self.bytecode.len()
    }
}

enum Opcode {
    // Stack and variable manipulation.
    Push(i32), // Push value on stack
    Pop,       // pop value from stack
    Get(u8),   // read variable and push on stack
    Set(u8),   // pop the top value and store in variable
    Copy(u8),  // keep the top value and store in variable
    Dup(u8),   // duplicate value in stack (index is offset from top)
    // Operators pop one or two values and push result
    Add, // addition operator
    Sub, // subtraction operator
    Mul, // multiplication operator
    Div, // integer division operator
    Mod, // modulus operator
    Neg, // Unary negation operator
    And, // logical and
    Or,  // logical or
    Xor, // logical xor
    Not, // Unary logical not
    Eql, // equality test
    Neq, // not equal test
    Gte, // greater than or equal test
    Lte, // less than or equal test
    Gt,  // greater than test
    Lt,  // less than test
    // Control flow
    Jump(i32),  // relative goto
    IJump(i32), // pop value and relative goto if true
}

fn main() {
    use Opcode::*;
    let state = State::new(vec![Push(1), Push(2), Add]);
    println!("Hello, world!");
}
