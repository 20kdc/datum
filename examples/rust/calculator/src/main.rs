/*
 * datum-rs - Quick to implement S-expression format
 * Written starting in 2024 by contributors (see CREDITS.txt at repository's root)
 * To the extent possible under law, the author(s) have dedicated all copyright and related and neighboring rights to this software to the public domain worldwide. This software is distributed without any warranty.
 * A copy of the Unlicense should have been supplied as COPYING.txt in this repository. Alternatively, you can find it at <https://unlicense.org/>.
 */

use std::collections::HashMap;

use datum_rs::{
    DatumAtom, DatumErrorKind, DatumMayContainAtom, DatumResult, DatumValue, ViaDatumPipe,
};
use rand::RngCore;
use rustyline::{config::Configurer, validate::ValidationResult};

// ANCHOR: virtual-machine

/// Compiled machine expression.
/// This isn't written for speed, so there's no common subexpression removal/etc.
#[derive(Clone)]
enum CompiledExpr {
    /// Parameter to function, specified by index.
    Arg(usize),
    Const(f64),
    Abs(Box<CompiledExpr>),
    Binop(fn(f64, f64) -> f64, Box<CompiledExpr>, Box<CompiledExpr>),
}

impl CompiledExpr {
    /// Runs the expression with the given arguments.
    fn run(&self, args: &[f64]) -> f64 {
        match self {
            Self::Arg(i) => args[*i],
            Self::Const(f) => *f,
            Self::Abs(e) => e.run(args).abs(),
            Self::Binop(b, s, c) => b(s.run(args), c.run(args)),
        }
    }

    /// Returns a copy with the expression arguments replaced with further expressions.
    fn translate(&self, args: &[CompiledExpr]) -> Self {
        match self {
            Self::Arg(i) => args[*i].clone(),
            Self::Const(f) => Self::Const(*f),
            Self::Abs(e) => Self::Abs(Box::new(e.translate(args))),
            Self::Binop(b, s, c) => {
                Self::Binop(*b, Box::new(s.translate(args)), Box::new(c.translate(args)))
            }
        }
    }
}
// ANCHOR_END: virtual-machine

// ANCHOR: environment
struct Function {
    name: String,
    args: usize,
    expr: CompiledExpr,
}

impl Function {
    /// Creates a new function, auto-determining argument count.
    fn new(name: &str, args: usize, expr: CompiledExpr) -> Function {
        Function {
            name: name.to_string(),
            args,
            expr,
        }
    }
    fn new_binop(name: &str, binop: fn(f64, f64) -> f64) -> Function {
        Function::new(
            name,
            2,
            CompiledExpr::Binop(
                binop,
                Box::new(CompiledExpr::Arg(0)),
                Box::new(CompiledExpr::Arg(1)),
            ),
        )
    }
}

/// The calculator environment itself.
struct Environment {
    /// User-defined functions.
    functions: Vec<Function>,
}

impl Environment {
    fn new() -> Environment {
        Environment {
            functions: vec![
                Function::new_binop("+", |a, b| a + b),
                Function::new_binop("-", |a, b| a - b),
                Function::new_binop("/", |a, b| a / b),
                Function::new_binop("*", |a, b| a * b),
                Function::new_binop("min", |a, b| a.min(b)),
                Function::new_binop("max", |a, b| a.max(b)),
                Function::new("abs", 1, CompiledExpr::Abs(Box::new(CompiledExpr::Arg(0)))),
            ],
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
    /// Compiles an expression.
    fn compile_expr(
        &self,
        args: &HashMap<String, usize>,
        expr: &DatumValue,
    ) -> Result<CompiledExpr, String> {
        match expr {
            DatumValue::Atom(DatumAtom::ID(id)) => {
                if let Some(k) = args.get(id) {
                    Ok(CompiledExpr::Arg(*k))
                } else if let Some(fid) = self.fn_index_for(id, 0) {
                    Ok(CompiledExpr::Const(self.functions[fid].expr.run(&[])))
                } else {
                    Err(format!("No such arg: {}", id))
                }
            }
            DatumValue::Atom(DatumAtom::Float(v)) => Ok(CompiledExpr::Const(*v)),
            DatumValue::Atom(DatumAtom::Integer(v)) => Ok(CompiledExpr::Const(*v as f64)),
            DatumValue::List(list) => {
                if let Some(DatumValue::Atom(DatumAtom::ID(sym))) = &list.get(0) {
                    let call_args = &list[1..];
                    if let Some(fni) = self.fn_index_for(&sym, call_args.len()) {
                        let v = &self.functions[fni];
                        // compile expressions, and if successful, translate them into our args
                        self.compile_exprs(args, call_args)
                            .map(|compiled_call_args| v.expr.translate(&compiled_call_args))
                    } else {
                        Err(format!(
                            "Function not defined: {} #{}",
                            sym,
                            call_args.len()
                        ))
                    }
                } else {
                    Err(format!("Cannot have this kind of list: {:?}", list))
                }
            }
            _ => Err(format!("Not usable here: {:?}", expr)),
        }
    }
    /// Compiles a list of expressions.
    fn compile_exprs(
        &self,
        args: &HashMap<String, usize>,
        exprs: &[DatumValue],
    ) -> Result<Vec<CompiledExpr>, String> {
        exprs
            .iter()
            .map(|arg| self.compile_expr(args, arg))
            .try_fold(Vec::new(), |mut v, e| match e {
                Err(err) => Err(err),
                Ok(elm) => {
                    v.push(elm);
                    Ok(v)
                }
            })
    }
    /// Expression execution.
    fn execute_expr(&mut self, value: &DatumValue) -> Result<(), String> {
        let res = self.compile_expr(&HashMap::new(), value)?.run(&[]);
        println!("= {}", res);
        Ok(())
    }
}
// ANCHOR_END: compiler

// ANCHOR: executor
impl Environment {
    /// High-level execution.
    /// This includes the 'meta' forms (def and minimize).
    fn execute(&mut self, value: &DatumValue) -> Result<(), String> {
        if let Some(list) = value.as_list() {
            match list.get(0) {
                Some(DatumValue::Atom(DatumAtom::ID(syntax_maybe))) => {
                    if syntax_maybe.eq("def") {
                        if list.len() < 3 {
                            return Err(format!("def has to be at least 3 items long"));
                        }
                        let res = list[1].as_id_result(|| format!("def name must be an ID"))?.to_string();
                        let argslice = &list[2..list.len() - 1];
                        let mut argsyms: HashMap<String, usize> = HashMap::new();
                        for (k, v) in argslice.iter().enumerate() {
                            argsyms.insert(v.as_id_result(|| format!("def args must be IDs"))?.to_string(), k);
                        }
                        let compiled = self.compile_expr(&argsyms, &list[list.len() - 1])?;
                        if let Some(fni) = self.fn_index_for(&res, argslice.len()) {
                            // replace (not override due to inlining)
                            self.functions.remove(fni);
                        }
                        self.functions.push(Function {
                            name: res,
                            args: argslice.len(),
                            expr: compiled,
                        });
                        Ok(())
                    } else if syntax_maybe.eq("minimize") {
                        if list.len() < 4 {
                            return Err(format!("minimize has to be at least 4 items long"));
                        }
                        let sym = list[1].as_id_result(|| format!("name must be symbol"))?;
                        let mut magnitude = list[2].as_number_result(|| format!("magnitude must be number"))?;
                        let tolerance = list[3].as_number_result(|| format!("tolerance must be number"))?;
                        let initial_value_exprs =
                            self.compile_exprs(&HashMap::new(), &list[4..])?;
                        let mut values: Vec<f64> = initial_value_exprs
                            .iter()
                            .map(|expr| expr.run(&[]))
                            .collect();
                        let fni =
                            if let Some(fni) = self.fn_index_for(&sym, initial_value_exprs.len()) {
                                fni
                            } else {
                                return Err(format!(
                                    "function {} #{} does not exist",
                                    sym,
                                    initial_value_exprs.len()
                                ));
                            };
                        // This is the actual 'solver'. I am aware you are probably not supposed to solve equations this way, but it works!
                        let mut values_score = self.functions[fni].expr.run(&values).abs();
                        let mut at_this_score_half_magnitude = values_score / 2.0;
                        let mut values_test = values.clone();
                        println!("initial score {}", values_score);
                        let mut rng = rand::thread_rng();
                        while values_score >= tolerance {
                            for (k, v) in values_test.iter_mut().enumerate() {
                                let ofs = (((rng.next_u32() as f64) / (u32::max_value() as f64))
                                    - 0.5)
                                    * 2.0
                                    * magnitude;
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
                        Ok(())
                    } else {
                        self.execute_expr(value)
                    }
                }
                _ => self.execute_expr(value),
            }
        } else {
            self.execute_expr(value)
        }
    }
}
// ANCHOR_END: executor

// ANCHOR: main
struct DatumParseHelper;

impl rustyline::validate::Validator for DatumParseHelper {
    fn validate(
        &self,
        ctx: &mut rustyline::validate::ValidationContext,
    ) -> rustyline::Result<rustyline::validate::ValidationResult> {
        // Parse the line to see if it's invalid.
        let res: DatumResult<()> = ctx
            .input()
            .chars()
            .via_datum_pipe(datum_rs::datum_char_to_value_pipeline())
            .try_fold((), |_, r| match r {
                Err(e) => Err(e),
                Ok(_) => Ok(()),
            });
        match res {
            Ok(_) => Ok(ValidationResult::Valid(None)),
            Err(err) => {
                if err.kind == DatumErrorKind::Interrupted {
                    Ok(ValidationResult::Incomplete)
                } else {
                    Ok(ValidationResult::Invalid(Some(format!(" {:?}", err))))
                }
            }
        }
    }
    fn validate_while_typing(&self) -> bool {
        true
    }
}
impl rustyline::highlight::Highlighter for DatumParseHelper {}
impl rustyline::hint::Hinter for DatumParseHelper {
    type Hint = String;
}
impl rustyline::completion::Completer for DatumParseHelper {
    type Candidate = String;
}
impl rustyline::Helper for DatumParseHelper {}

fn main() {
    let mut rl: rustyline::Editor<DatumParseHelper, rustyline::history::DefaultHistory> =
        rustyline::Editor::new().expect("rustyline expected to initialize");
    rl.set_auto_add_history(true);
    println!(
        "
Desk Calculator
A datum-rs Example Program
Basic operations are (+ X Y) (- X Y) (/ X Y) (* X Y) (min X Y) (max X Y) (abs X)
Functions can be defined with (def name args... expr)
 different arg counts count as different functions
Solve a function with (minimize name magnitude tolerance initial...)
 this will attempt to bring (abs (name initial...)) to under tolerance
Example: (def problem x (- (- (* x 2) 5) 9)) (minimize problem 1 0.001 0)
"
    );
    let mut env = Environment::new();
    loop {
        rl.set_helper(Some(DatumParseHelper));
        let line = rl.readline(&"> ");
        match line {
            Ok(line) => {
                for v in line
                    .chars()
                    .via_datum_pipe(datum_rs::datum_char_to_value_pipeline())
                {
                    match v {
                        Err(err) => {
                            println!("Parse error: {:?}", err);
                            break;
                        }
                        Ok(v) => {
                            if let Err(err) = env.execute(&v) {
                                println!("Run error: {}", err);
                                break;
                            }
                        }
                    }
                }
            }
            Err(e) => match e {
                rustyline::error::ReadlineError::Eof => break,
                rustyline::error::ReadlineError::Interrupted => break,
                _ => (),
            },
        }
    }
}
// ANCHOR_END: main
