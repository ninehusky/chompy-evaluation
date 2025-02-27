use std::{path::PathBuf, str::FromStr};

use caviar::structs::{Ruleset, RulesetTag};
use ruler::{enumo::Sexp, halide::Pred};

use clap::Parser;

enum EvalMode {
    CaviarComparison,
    RulesetComparison,
    Verify,
}

impl From<String> for EvalMode {
    fn from(s: String) -> Self {
        match s.as_str() {
            "caviar" => EvalMode::CaviarComparison,
            "ruleset" => EvalMode::RulesetComparison,
            "verify" => EvalMode::Verify,
            _ => panic!("Invalid mode: {}", s),
        }
    }
}

#[derive(Parser, Debug)]
struct CLIArgs {
    #[arg(short, long)]
    eval_mode: String,

    #[arg(long, value_name = "FILE")]
    dataset_path: Option<PathBuf>,
}

fn main() {
    let args = CLIArgs::parse();
    let mode = EvalMode::from(args.eval_mode);
    match mode {
        EvalMode::CaviarComparison => {
            let dataset_path = args.dataset_path.unwrap();
            caviar_comparison(dataset_path);
        }
        EvalMode::RulesetComparison => {
            println!("Ruleset comparison");
        }
        EvalMode::Verify => {
            let dataset_path = args.dataset_path.unwrap();
            verify_expressions(dataset_path);
        }
    }
}

fn verify_expressions(path: PathBuf) -> Vec<ruler::ValidationResult> {
    let mut results = Vec::new();
    let expressions = caviar::io::reader::read_expressions(&path.into());
    for r in expressions.unwrap().iter() {
        let halide_expr = r.expression.clone();
        let res = ruler::halide::validate_expression(&Sexp::from_str(&halide_expr).unwrap());
        println!("Validation result: {:?}", res);
        results.push(res);
    }
    results
}

fn caviar_comparison(path: PathBuf) {
    // get the expression first.
    let results = caviar::io::reader::read_expressions(&path.into());
    for res in results.unwrap().iter() {
        println!("consider expression: {:?}", res);
        // let res = caviar::trs::prove_expression
        // let res = caviar::prove_expressions_pulses_npp_paper(exprs_vect, ruleset, threshold, params, use_iteration_check, report)
    }
}
