use std::{path::PathBuf, str::FromStr};

use caviar::structs::{ResultStructure, Ruleset, RulesetTag};
use ruler::{
    enumo::{self, Rule, Sexp, Workload},
    halide::Pred,
    SynthLanguage, ValidationResult,
};

use clap::Parser;

enum EvalMode {
    CaviarComparison,
    DerivabilityComparison,
    Eggsplain,
    Verify,
}

impl From<String> for EvalMode {
    fn from(s: String) -> Self {
        match s.as_str() {
            "caviar" => EvalMode::CaviarComparison,
            "derivability" => EvalMode::DerivabilityComparison,
            "verify" => EvalMode::Verify,
            "eggsplain" => EvalMode::Eggsplain,
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

    #[arg(long, value_name = "FILE")]
    other_ruleset_path: Option<PathBuf>,

    #[arg(long, value_name = "FILE")]
    ruleset_comparison_output_path: Option<PathBuf>,

    #[arg(long, value_name = "FILE")]
    explanation_output_path: Option<PathBuf>,

    #[arg(long, value_name = "FILE")]
    derivability_output_path: Option<PathBuf>,
}

#[derive(Debug)]
struct RulesetComparisonResult {
    expression: String,
    chompy_result: ResultStructure,
    caviar_result: ResultStructure,
    z3_result: ValidationResult,
}

#[derive(Debug)]
struct ExplanationResult {
    expression: String,
    chompy_result: caviar_new::structs::ResultStructure,
    caviar_result: caviar_new::structs::ResultStructure,
    chompy_explanation: String,
    caviar_explanation: String,
    z3_result: ValidationResult,
}

#[derive(Debug)]
struct DerivabilityResult {
    can_len: usize,
    cannot_len: usize,
    can: Vec<String>,
    cannot: Vec<String>,
}

fn main() {
    let args = CLIArgs::parse();
    let mode = EvalMode::from(args.eval_mode);
    match mode {
        EvalMode::CaviarComparison => {
            let dataset_path = args.dataset_path.unwrap();
            let chompy_ruleset_path = args.chompy_ruleset_path.unwrap();
            let output_path = args.ruleset_comparison_output_path.unwrap();
            let chompy_ruleset = Ruleset::new(RulesetTag::Custom(
                chompy_ruleset_path.to_str().unwrap().to_string(),
            ));
            let results = caviar_comparison(dataset_path, &chompy_ruleset);
            write_chompy_caviar_results_to_json(output_path, &chompy_ruleset, &results);
        }
        EvalMode::DerivabilityComparison => {
            let ruleset_path = args.chompy_ruleset_path.unwrap();
            let against_path = args.other_ruleset_path.unwrap();
            let output_path = args.derivability_output_path.unwrap();
            let first_direction = derivability_check(ruleset_path.clone(), against_path.clone());
            let second_direction = derivability_check(against_path, ruleset_path);
            write_derivability_results_to_json(&output_path, &first_direction, &second_direction);
        }
        EvalMode::Eggsplain => {
            let dataset_path = args.dataset_path.unwrap();
            let chompy_ruleset_path = args.chompy_ruleset_path.unwrap();
            let output_path = args.explanation_output_path.unwrap();
            let chompy_ruleset =
                caviar_new::structs::Ruleset::new(caviar_new::structs::RulesetTag::Custom(
                    chompy_ruleset_path.to_str().unwrap().to_string(),
                ));
            let results = eggsplanations(dataset_path, &chompy_ruleset);
            write_eggsplanation_results_to_json(&output_path, &chompy_ruleset, &results);
        }
        EvalMode::Verify => {
            let dataset_path = args.dataset_path.unwrap();
            verify_expressions(dataset_path);
        }
    }
}

fn derivability_check(ruleset_path: PathBuf, against_path: PathBuf) -> DerivabilityResult {
    fn read_ruleset_from_file(path: PathBuf) -> enumo::Ruleset<Pred> {
        let lines = std::fs::read_to_string(path).unwrap();
        let mut result: enumo::Ruleset<Pred> = enumo::Ruleset::default();
        for r in lines.split('\n') {
            if r.is_empty() {
                continue;
            }
            let (fw, bw) = Rule::from_string(r).unwrap();
            result.add(fw);
            if let Some(bw) = bw {
                result.add(bw);
            }
        }
        result
    }

    fn get_conditions(ruleset: &enumo::Ruleset<Pred>) -> Vec<String> {
        ruleset
            .iter()
            .filter_map(|r| r.cond.clone().map(|c| c.to_string()))
            .collect()
    }

    println!("ruleset: {:?}", ruleset_path);
    let ruleset = read_ruleset_from_file(ruleset_path);
    println!("against: {:?}", against_path);
    let against = read_ruleset_from_file(against_path);

    let mut conditions = get_conditions(&ruleset);
    conditions.extend(get_conditions(&against));

    conditions.dedup();

    let cond_wkld = Workload::new(&conditions);
    println!("conditions: {:#?}", conditions.len());

    let conditional_prop_rules = ruler::halide::Pred::get_condition_propogation_rules(&cond_wkld);
    println!("made it here");
    let (can, cannot) = ruleset.derive(
        ruler::DeriveType::LhsAndRhs,
        &against,
        ruler::Limits::deriving(),
        &Some(conditional_prop_rules),
    );

    let can: Vec<String> = can.into_iter().map(|r| r.0.to_string()).collect();
    let cannot: Vec<String> = cannot.into_iter().map(|r| r.0.to_string()).collect();

    DerivabilityResult {
        can_len: can.len(),
        cannot_len: cannot.len(),
        can,
        cannot,
    }
}

fn verify_expressions(path: PathBuf) -> Vec<ruler::ValidationResult> {
    let mut results = Vec::new();
    let expressions = caviar::io::reader::read_expressions(&path.into());
    for r in expressions.unwrap().iter() {
        let halide_expr = r.expression.clone();
        let res = ruler::halide::validate_expression(&Sexp::from_str(&halide_expr).unwrap());
        println!("Validation result for {}: {:?}", halide_expr, res);
        results.push(res);
    }
    results
}

fn caviar_comparison(expr_path: PathBuf, chompy_ruleset: &Ruleset) -> Vec<RulesetComparisonResult> {
    let mut results: Vec<RulesetComparisonResult> = Vec::new();
    let exprs = caviar::io::reader::read_expressions(&expr_path.into());
    let caviar_ruleset = Ruleset::new(RulesetTag::CaviarAll);
    let default_limits = (100000, 100000, 60.0);
    for expr_struct in exprs.unwrap().iter() {
        let caviar_res = caviar::trs::prove_pulses_npp(
            expr_struct.index,
            &expr_struct.expression,
            &caviar_ruleset,
            0.01,
            default_limits,
            true,
            false,
        );
        let chompy_res = caviar::trs::prove_pulses_npp(
            expr_struct.index,
            &expr_struct.expression,
            chompy_ruleset,
            0.01,
            default_limits,
            true,
            false,
        );

        let res = RulesetComparisonResult {
            expression: expr_struct.expression.clone(),
            chompy_result: chompy_res,
            caviar_result: caviar_res,
            z3_result: ruler::halide::validate_expression(
                &Sexp::from_str(&expr_struct.expression).unwrap(),
            ),
        };
        results.push(res);
    }
    results
}

fn write_eggsplanation_results_to_json(
    output_path: &PathBuf,
    chompy_ruleset: &caviar_new::structs::Ruleset,
    results: &[ExplanationResult],
) {
    let validation_result_to_string = |res: &ValidationResult| match res {
        ValidationResult::Valid => "valid",
        ValidationResult::Invalid => "invalid",
        ValidationResult::Unknown => "unknown",
    };
    let ruleset_strings: Vec<String> = chompy_ruleset
        .rules()
        .iter()
        .map(|r| r.name.to_string())
        .collect();
    let ruleset_json = serde_json::json!(ruleset_strings);
    let results_json = results
        .iter()
        .map(|r| {
            serde_json::json!({
                "expression": r.expression,
                "chompy_result": r.chompy_result,
                "caviar_result": r.caviar_result,
                "chompy_explanation": r.chompy_explanation,
                "caviar_explanation": r.caviar_explanation,
                "z3_result": validation_result_to_string(&r.z3_result),
            })
        })
        .collect::<Vec<serde_json::Value>>();
    let json = serde_json::json!({
        "ruleset": ruleset_json,
        "results": results_json,
    });
    std::fs::write(output_path, json.to_string()).unwrap();
}

fn write_chompy_caviar_results_to_json(
    output_path: PathBuf,
    chompy_ruleset: &Ruleset,
    results: &[RulesetComparisonResult],
) {
    let validation_result_to_string = |res: &ValidationResult| match res {
        ValidationResult::Valid => "valid",
        ValidationResult::Invalid => "invalid",
        ValidationResult::Unknown => "unknown",
    };
    let ruleset_strings: Vec<String> = chompy_ruleset
        .rules()
        .iter()
        .map(|r| r.name().to_string())
        .collect();
    let ruleset_json = serde_json::json!(ruleset_strings);
    let results_json = results
        .iter()
        .map(|r| {
            serde_json::json!({
                "expression": r.expression,
                "chompy_result": r.chompy_result,
                "caviar_result": r.caviar_result,
                "z3_result": validation_result_to_string(&r.z3_result),
            })
        })
        .collect::<Vec<serde_json::Value>>();
    let json = serde_json::json!({
        "ruleset": ruleset_json,
        "results": results_json,
    });
    std::fs::write(output_path, json.to_string()).unwrap();
}

fn eggsplanations(
    expr_path: PathBuf,
    chompy_ruleset: &caviar_new::structs::Ruleset,
) -> Vec<ExplanationResult> {
    let mut results: Vec<ExplanationResult> = Vec::new();
    let exprs = caviar_new::io::reader::read_expressions(&expr_path.into());
    let caviar_ruleset =
        caviar_new::structs::Ruleset::new(caviar_new::structs::RulesetTag::CaviarAll);
    let default_limits = (100000, 100000, 3.0);
    for expr_struct in exprs.unwrap().iter() {
        let (caviar_res, caviar_explanation) = caviar_new::trs::prove_with_explanation(
            expr_struct.index,
            &expr_struct.expression,
            &caviar_ruleset,
            default_limits,
            false,
            false,
        );
        let (chompy_res, chompy_explanation) = caviar_new::trs::prove_with_explanation(
            expr_struct.index,
            &expr_struct.expression,
            chompy_ruleset,
            default_limits,
            false,
            false,
        );

        let res = ExplanationResult {
            expression: expr_struct.expression.clone(),
            chompy_result: chompy_res,
            caviar_result: caviar_res,
            chompy_explanation,
            caviar_explanation,
            z3_result: ruler::halide::validate_expression(
                &Sexp::from_str(&expr_struct.expression).unwrap(),
            ),
        };
        results.push(res);
    }
    results
}

fn write_derivability_results_to_json(
    output_path: &PathBuf,
    forwards_result: &DerivabilityResult,
    backwards_result: &DerivabilityResult,
) {
    let to_json = |result: &DerivabilityResult| {
        serde_json::json!({
            "can_len": result.can_len,
            "cannot_len": result.cannot_len,
            "can": result.can,
            "cannot": result.cannot,
        })
    };
    let json = serde_json::json!({
        "forwards": to_json(forwards_result),
        "backwards": to_json(backwards_result),
    });
    std::fs::write(output_path, json.to_string()).unwrap();
}
