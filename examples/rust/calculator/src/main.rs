/*
 * datum-rs - Quick to implement S-expression format
 * Written starting in 2024 by contributors (see CREDITS.txt at repository's root)
 * To the extent possible under law, the author(s) have dedicated all copyright and related and neighboring rights to this software to the public domain worldwide. This software is distributed without any warranty.
 * A copy of the Unlicense should have been supplied as COPYING.txt in this repository. Alternatively, you can find it at <https://unlicense.org/>.
 */

use std::collections::HashMap;

use datum_rs::{DatumAtom, DatumErrorKind, DatumMayContainAtom, DatumValue, ViaDatumPipe};
use rand::RngCore;
use rustyline::config::Configurer;

// ANCHOR: virtual-machine
/// Operation between two numbers.
#[derive(Clone, Copy)]
enum Binop {
    Add,
    Sub,
    Div,
    Mul,
    Min,
    Max
}

impl Binop {
    fn run(&self, a: f64, b: f64) -> f64 {
        match self {
            Self::Add => a + b,
            Self::Sub => a - b,
            Self::Div => a / b,
            Self::Mul => a * b,
            Self::Min => a.min(b),
            Self::Max => a.max(b),
        }
    }
}

/// Compiled machine expression.
/// This isn't written for speed, so there's no common subexpression removal/etc.
#[derive(Clone)]
enum CompiledExpr {
    /// Parameter to function, specified by index.
    Arg(usize),
    Const(f64),
    Abs(Box<CompiledExpr>),
    Binop(Binop, Box<CompiledExpr>, Box<CompiledExpr>),
}

impl CompiledExpr {
    /// Runs the expression with the given arguments.
    fn run(&self, args: &[f64]) -> f64 {
        match self {
            Self::Arg(i) => args[*i],
            Self::Const(f) => *f,
            Self::Abs(e) => e.run(args).abs(),
            Self::Binop(b, s, c) => b.run(s.run(args), c.run(args))
        }
    }

    /// Returns a copy with the expression arguments replaced with further expressions.
    fn translate(&self, args: &[CompiledExpr]) -> Self {
        match self {
            Self::Arg(i) => args[*i].clone(),
            Self::Const(f) => Self::Const(*f),
            Self::Abs(e) => Self::Abs(Box::new(e.translate(args))),
            Self::Binop(b, s, c) => Self::Binop(*b, Box::new(s.translate(args)), Box::new(c.translate(args))),
        }
    }
}
// ANCHOR_END: virtual-machine

// ANCHOR: environment
struct Function {
    name: String,
    args: usize,
    expr: CompiledExpr
}

/// The calculator environment itself.
struct Environment {
    /// User-defined functions.
    functions: Vec<Function>
}

impl Environment {
    fn new() -> Environment {
        Environment {
            functions: vec![
                Function { name: ("+").to_string(),   args: 2, expr: CompiledExpr::Binop(Binop::Add, Box::new(CompiledExpr::Arg(0)), Box::new(CompiledExpr::Arg(1))) },
                Function { name: ("-").to_string(),   args: 2, expr: CompiledExpr::Binop(Binop::Sub, Box::new(CompiledExpr::Arg(0)), Box::new(CompiledExpr::Arg(1))) },
                Function { name: ("/").to_string(),   args: 2, expr: CompiledExpr::Binop(Binop::Div, Box::new(CompiledExpr::Arg(0)), Box::new(CompiledExpr::Arg(1))) },
                Function { name: ("*").to_string(),   args: 2, expr: CompiledExpr::Binop(Binop::Mul, Box::new(CompiledExpr::Arg(0)), Box::new(CompiledExpr::Arg(1))) },
                Function { name: ("min").to_string(), args: 2, expr: CompiledExpr::Binop(Binop::Min, Box::new(CompiledExpr::Arg(0)), Box::new(CompiledExpr::Arg(1))) },
                Function { name: ("max").to_string(), args: 2, expr: CompiledExpr::Binop(Binop::Max, Box::new(CompiledExpr::Arg(0)), Box::new(CompiledExpr::Arg(1))) },
                Function { name: ("abs").to_string(), args: 1, expr: CompiledExpr::Abs(Box::new(CompiledExpr::Arg(0))) },
            ]
        }
    }
    /// Get function index of existing function
    fn fn_index_for(&self, name: &str, args: usize) -> Option<usize> {
        for (k, v) in self.functions.iter().enumerate() {
            if v.name.eq(name) && v.args == args {
                return Some(k);
            }
        }
        None
    }
}
// ANCHOR_END: environment

// ANCHOR: compiler
impl Environment {
    /// Compiles an expression (or returns None, failing).
    fn compile_expr(&self, args: &HashMap<String, usize>, expr: &DatumValue) -> Option<CompiledExpr> {
        match expr {
            DatumValue::Atom(DatumAtom::ID(id)) => {
                if let Some(k) = args.get(id) {
                    Some(CompiledExpr::Arg(*k))
                } else if let Some(fid) = self.fn_index_for(id, 0) {
                    Some(CompiledExpr::Const(self.functions[fid].expr.run(&[])))
                } else {
                    println!("No such arg: {}", id);
                    None
                }
            },
            DatumValue::Atom(DatumAtom::Float(v)) => Some(CompiledExpr::Const(*v)),
            DatumValue::Atom(DatumAtom::Integer(v)) => Some(CompiledExpr::Const(*v as f64)),
            DatumValue::List(list) => {
                if let Some(DatumValue::Atom(DatumAtom::ID(sym))) = &list.get(0) {
                    let call_args = &list[1..];
                    if let Some(fni) = self.fn_index_for(&sym, call_args.len()) {
                        let v = &self.functions[fni];
                        // compile expressions, and if successful, translate them into our args
                        self.compile_exprs(args, call_args).map(|compiled_call_args| v.expr.translate(&compiled_call_args))
                    } else {
                        println!("Function not defined: {} #{}", sym, call_args.len());
                        None
                    }
                } else {
                    println!("Cannot have this kind of list: {:?}", list);
                    None
                }
            },
            _ => {
                println!("Not usable here: {:?}", expr);
                None
            }
        }
    }
    /// Compiles a list of expressions (or returns None, failing).
    fn compile_exprs(&self, args: &HashMap<String, usize>, exprs: &[DatumValue]) -> Option<Vec<CompiledExpr>> {
        let mut stop_now = false;
        let exprs_compiled: Vec<CompiledExpr> = exprs.iter().map(|arg| self.compile_expr(args, arg)).inspect(|v| stop_now |= v.is_none()).map(|v| v.unwrap_or(CompiledExpr::Const(0.0))).collect();
        if stop_now {
            None
        } else {
            Some(exprs_compiled)
        }
    }
    /// Expression execution. Returns false on failure.
    fn execute_expr(&mut self, value: &DatumValue) -> bool {
        match self.compile_expr(&HashMap::new(), value) {
            Some(expr) => {
                println!("= {}", expr.run(&[]));
                true
            },
            None => false
        }
    }
}
// ANCHOR_END: compiler

// ANCHOR: executor
impl Environment {
    /// High-level execution.
    /// This includes the 'meta' forms (def and minimize).
    /// Returns false on failure.
    fn execute(&mut self, value: &DatumValue) -> bool {
        if let Some(list) = value.as_list() {
            match list.get(0) {
                Some(DatumValue::Atom(DatumAtom::ID(syntax_maybe))) => {
                    if syntax_maybe.eq("def") {
                        if list.len() < 3 {
                            println!("def has to be at least 3 items long");
                            return false;
                        }
                        let res = if let Some(sym) = list[1].as_id() {
                            sym.to_string()
                        } else {
                            println!("def name must be an ID");
                            return false;
                        };
                        let argslice = &list[2..list.len() - 1];
                        let mut argsyms: HashMap<String, usize> = HashMap::new();
                        for (k, v) in argslice.iter().enumerate() {
                            if let Some(sym) = v.as_id() {
                                argsyms.insert(sym.to_string(), k);
                            } else {
                                println!("def args must be IDs");
                                return false;
                            }
                        }
                        let compiled = self.compile_expr(&argsyms, &list[list.len() - 1]);
                        if let Some(success) = compiled {
                            if let Some(fni) = self.fn_index_for(&res, argslice.len()) {
                                // replace (not override due to inlining)
                                self.functions.remove(fni);
                            }
                            self.functions.push(Function {
                                name: res,
                                args: argslice.len(),
                                expr: success
                            });
                            true
                        } else {
                            false
                        }
                    } else if syntax_maybe.eq("minimize") {
                        if list.len() < 4 {
                            println!("minimize has to be at least 4 items long");
                            return false;
                        }
                        let sym = if let Some(sym) = list[1].as_id() {
                            sym
                        } else {
                            println!("name must be symbol");
                            return false;
                        };
                        let mut magnitude = if let Some(magnitude) = list[2].as_number() {
                            magnitude
                        } else {
                            println!("magnitude must be number");
                            return false;
                        };
                        let tolerance = if let Some(tolerance) = list[3].as_number() {
                            tolerance
                        } else {
                            println!("tolerance must be number");
                            return false;
                        };
                        let initial_value_exprs = if let Some(ive) = self.compile_exprs(&HashMap::new(), &list[4..]) {
                            ive
                        } else {
                            return false;
                        };
                        let mut values: Vec<f64> = initial_value_exprs.iter().map(|expr| expr.run(&[])).collect();
                        let fni = if let Some(fni) = self.fn_index_for(&sym, initial_value_exprs.len()) {
                            fni
                        } else {
                            println!("function {} #{} does not exist", sym, initial_value_exprs.len());
                            return false;
                        };
                        // This is the actual 'solver'. I am aware you are probably not supposed to solve equations this way, but it works!
                        let mut values_score = self.functions[fni].expr.run(&values).abs();
                        let mut at_this_score_half_magnitude = values_score / 2.0;
                        let mut values_test = values.clone();
                        println!("initial score {}", values_score);
                        let mut rng = rand::thread_rng();
                        while values_score >= tolerance {
                            for (k, v) in values_test.iter_mut().enumerate() {
                                let ofs = (((rng.next_u32() as f64) / (u32::max_value() as f64)) - 0.5) * 2.0 * magnitude;
                                *v = values[k] + ofs;
                            }
                            let new_score = self.functions[fni].expr.run(&values_test).abs();
                            if new_score < values_score {
                                values.copy_from_slice(&values_test);
                                values_score = new_score;
                                println!("@ score {}", values_score);
                                while values_score < at_this_score_half_magnitude {
                                    magnitude /= 2.0;
                                    at_this_score_half_magnitude /= 2.0;
                                }
                            }
                        }
                        for value in values {
                            println!("= {}", value);
                        }
                        true
                    } else {
                        self.execute_expr(value)
                    }
                }
                _ => self.execute_expr(value)
            }
        } else {
            self.execute_expr(value)
        }
    }
}
// ANCHOR_END: executor

// ANCHOR: main
fn main() {
    let mut rl = rustyline::DefaultEditor::new().expect("rustyline expected to initialize");
    rl.set_auto_add_history(true);
    println!("Desk Calculator");
    println!("A datum-rs Example Program");
    println!("Basic operations are (+ X Y) (- X Y) (/ X Y) (* X Y) (min X Y) (max X Y) (abs X)");
    println!("Functions can be defined with (def name args... expr)");
    println!(" different arg counts count as different functions");
    println!("Solve a function with (minimize name magnitude tolerance initial...)");
    println!(" this will attempt to bring (abs (name initial...)) to under tolerance");
    println!("Example: (def problem x (- (- (* x 2) 5) 9)) (minimize problem 1 0.001 0)");
    let mut combo_buffer = String::new();
    let mut env = Environment::new();
    loop {
        let mut should_clear_combo_buffer = true;
        let mut prompt: &'static str = &"> ";
        if !combo_buffer.is_empty() {
            prompt = &"... ";
        }
        match rl.readline(prompt) {
            Ok(line) => {
                combo_buffer += &line;
                for v in combo_buffer.chars().via_datum_pipe(datum_rs::datum_char_to_value_pipeline()) {
                    match v {
                        Err(err) => {
                            if err.kind == DatumErrorKind::Interrupted {
                                should_clear_combo_buffer = false;
                                combo_buffer += &"\n";
                            } else {
                                println!("Parse error: {:?}", err);
                            }
                            break;
                        },
                        Ok(v) => {
                            if !env.execute(&v) {
                                break;
                            }
                        }
                    }
                }
            },
            Err(e) => match e {
                rustyline::error::ReadlineError::Eof => { break; }
                rustyline::error::ReadlineError::Interrupted => { break; }
                _ => {
                    // retry
                }
            }
        }
        if should_clear_combo_buffer {
            combo_buffer = String::new();
        }
    }
}
// ANCHOR_END: main
