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
    PruneHalide,
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

    #[arg(long)]
    prune_rules: bool,
}

#[derive(Debug)]
struct RulesetComparisonResult {
    expression: String,
    other_result: ResultStructure,
    caviar_result: ResultStructure,
    z3_result: ValidationResult,
}

#[derive(Debug)]
struct ExplanationResult {
    expression: String,
    caviar_result: caviar_new::structs::ResultStructure,
    other_result: caviar_new::structs::ResultStructure,
    other_explanation: String,
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
            let chompy_ruleset_path = args.chompy_ruleset_path;
            let output_path = args.ruleset_comparison_output_path.unwrap();

            // None means that the comparison should be internal to Caviar
            let other_ruleset = match chompy_ruleset_path {
                Some(path) => Ruleset::new(RulesetTag::Custom(path.to_str().unwrap().to_string())),
                None => Ruleset::new(RulesetTag::CaviarOnlyTotal),
            };

            let results = caviar_comparison(dataset_path, &other_ruleset);
            write_chompy_caviar_results_to_json(output_path, &other_ruleset, &results);
        }
        EvalMode::DerivabilityComparison => {
            // returns a file containing the ruleset with the rules that are not pruned.
            fn prune_rules(path: PathBuf) -> PathBuf {
                let mut temp_path = path.clone();
                temp_path.set_extension("tmp");
                let mut temp_file = std::fs::File::create(temp_path.clone()).unwrap();
                let ruleset = prune_halide_rules(path.clone());
                ruleset.to_file(temp_path.to_str().unwrap());
                temp_path
            }

            let [ruleset_path, against_path] = if args.prune_rules {
                let ruleset_path = args.chompy_ruleset_path.unwrap();
                let against_path = args.other_ruleset_path.unwrap();
                let pruned_ruleset_path = prune_rules(ruleset_path);
                let pruned_against_path = prune_rules(against_path);
                [pruned_ruleset_path, pruned_against_path]
            } else {
                [
                    args.chompy_ruleset_path.unwrap(),
                    args.other_ruleset_path.unwrap(),
                ]
            };

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
        EvalMode::PruneHalide => {
            let ruleset_path = args.chompy_ruleset_path.unwrap();
        }
    }
}


pub fn check_valid_ops(s: &str) -> bool {
    // an "op" is just the first token that follows a "(" in the string.
    // split on whitespace
    let mut tokens = s.split_whitespace();
    let mut ops = vec![];
    for t in tokens {
        if t.starts_with('(') {
            let op = t[1..].split_whitespace().next().unwrap();
            ops.push(op);
        }
    }

    let allowed_ops = vec![
        "abs",
        "<",
        "<=",
        ">" ,
        ">=",
        "==",
        "!=",
        "->",
        "!",
        "-",
        "&&",
        "||",
        "^",
        "+",
        "-",
        "*",
        "/",
        "\"%\"",
        "min",
        "max",
        "select",
        "istrue",
    ];


    for op in ops {
        if !allowed_ops.contains(&op) {
            println!("Invalid op: {}", op);
            return false;
        }
    }
    true
}

fn prune_halide_rules(ruleset_path: PathBuf) -> enumo::Ruleset<Pred> {
    let lines = std::fs::read_to_string(ruleset_path).unwrap();
    let mut kept: enumo::Ruleset<Pred> = enumo::Ruleset::default();
    for r in lines.split('\n') {


        if r.is_empty() {
            continue;
        }

        if !check_valid_ops(r) {
            continue;
        }


        let result = Rule::from_string(r);
        if result.is_ok() {
            let (fw, bw) = result.unwrap();
            let mut possible_rules = vec![fw];
            if let Some(bw) = bw {
                possible_rules.push(bw);
            }

            for r in possible_rules {
                let l_vars = r.lhs.vars();
                let r_vars = r.rhs.vars();

                if r_vars.iter().any(|v| !l_vars.contains(v)) {
                    println!("throwing out rule: {}", r);
                    continue;
                }
                kept.add(r);
            }
        } else {
            println!("throwing out rule: {}", r);
            continue;
        }
    }

    kept
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

    let conditional_prop_rules = ruler::halide::Pred::get_condition_propagation_rules(&cond_wkld);
    let (can, cannot) = ruleset.derive(
        ruler::DeriveType::LhsAndRhs,
        &against,
        ruler::Limits::deriving(),
        Some(conditional_prop_rules.as_ref()),
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

fn caviar_comparison(expr_path: PathBuf, other_ruleset: &Ruleset) -> Vec<RulesetComparisonResult> {
    let mut results: Vec<RulesetComparisonResult> = Vec::new();
    let exprs = caviar::io::reader::read_expressions(&expr_path.into());
    let caviar_ruleset = Ruleset::new(RulesetTag::CaviarAll);
    let default_limits = (100000, 100000, 3.0);
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
        let other_res = caviar::trs::prove_pulses_npp(
            expr_struct.index,
            &expr_struct.expression,
            other_ruleset,
            0.01,
            default_limits,
            true,
            false,
        );

        let res = RulesetComparisonResult {
            expression: expr_struct.expression.clone(),
            other_result: other_res,
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
                "other_result": r.other_result,
                "caviar_result": r.caviar_result,
                "other_explanation": r.other_explanation,
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
    other_ruleset: &Ruleset,
    results: &[RulesetComparisonResult],
) {
    let validation_result_to_string = |res: &ValidationResult| match res {
        ValidationResult::Valid => "valid",
        ValidationResult::Invalid => "invalid",
        ValidationResult::Unknown => "unknown",
    };
    let ruleset_strings: Vec<String> = other_ruleset
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
                "other_result": r.other_result,
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
    other_ruleset: &caviar_new::structs::Ruleset,
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
        let (other_res, other_explanation) = caviar_new::trs::prove_with_explanation(
            expr_struct.index,
            &expr_struct.expression,
            other_ruleset,
            default_limits,
            false,
            false,
        );

        let res = ExplanationResult {
            expression: expr_struct.expression.clone(),
            other_result: other_res,
            caviar_result: caviar_res,
            other_explanation,
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
