use std::{path::PathBuf, str::FromStr};

use caviar::structs::{ResultStructure, Ruleset, RulesetTag};
use ruler::{enumo::Sexp, halide::Pred, ValidationResult};

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

    #[arg(long, value_name = "FILE")]
    chompy_ruleset_path: Option<PathBuf>,
}

#[derive(Debug)]
struct RulesetComparisonResult {
    expression: String,
    chompy_result: ResultStructure,
    caviar_result: ResultStructure,
    z3_result: ValidationResult,
}

fn main() {
    let args = CLIArgs::parse();
    let mode = EvalMode::from(args.eval_mode);
    match mode {
        EvalMode::CaviarComparison => {
            let dataset_path = args.dataset_path.unwrap();
            let chompy_ruleset_path = args.chompy_ruleset_path.unwrap();
            caviar_comparison(dataset_path, chompy_ruleset_path);
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

fn caviar_comparison(
    expr_path: PathBuf,
    chompy_ruleset_path: PathBuf,
) -> Vec<RulesetComparisonResult> {
    let mut results: Vec<RulesetComparisonResult> = Vec::new();
    let exprs = caviar::io::reader::read_expressions(&expr_path.into());
    let caviar_ruleset = Ruleset::new(RulesetTag::CaviarAll);
    let chompy_ruleset = Ruleset::new(RulesetTag::Custom(
        chompy_ruleset_path.to_str().unwrap().to_string(),
    ));
    let default_limits = (100000, 100000, 3.0);
    for expr_struct in exprs.unwrap().iter() {
        let caviar_res = caviar::trs::prove_pulses_npp(
            expr_struct.index,
            &expr_struct.expression,
            &caviar_ruleset,
            3.0,
            default_limits,
            true,
            false,
        );
        let chompy_res = caviar::trs::prove_pulses_npp(
            expr_struct.index,
            &expr_struct.expression,
            &chompy_ruleset,
            3.0,
            default_limits,
            true,
            false,
        );
        // let z3_res =
        //     match ruler::halide::validate_expression(&Sexp::from_str(&expr_struct.expression).unwrap()) {
        //         ValidationResult::Valid => "false"
        //         ValidationResult::Invalid =>
        //  };

        let res = RulesetComparisonResult {
            expression: expr_struct.expression.clone(),
            chompy_result: chompy_res,
            caviar_result: caviar_res,
            z3_result: ruler::halide::validate_expression(
                &Sexp::from_str(&expr_struct.expression).unwrap(),
            ),
        };
        println!("res {:#?}", res);
        results.push(res);
    }
    results
}
