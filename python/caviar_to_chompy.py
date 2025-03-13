import argparse
from dataclasses import dataclass
from typing import Optional
import os
import re

RULE_DIR = f"{os.environ['CHOMPY_EVAL_DIR']}caviar/src/rules"

EXPECTED_FILES = [
        "add.rs",
        "and.rs",
        "andor.rs",
        "div.rs",
        "eq.rs",
        "ineq.rs",
        "lt.rs",
        "max.rs",
        "min.rs",
        "mod.rs",
        "modulo.rs",
        "mul.rs",
        "not.rs",
        "or.rs",
        "sub.rs",
]

@dataclass
class Rewrite:
    name: str
    lhs: str
    rhs: str
    cond: Optional[str]

    def __str__(self):
        if self.cond is not None:
            return f"{self.lhs} ==> {self.rhs} if {self.cond}"
        else:
            return f"{self.lhs} ==> {self.rhs}"


def get_rewrites() -> list[str]:
    assert set(os.listdir(RULE_DIR)) == set(EXPECTED_FILES), "The files in the rules directory do not match the expected files"
    rws = []

    for file in os.listdir(RULE_DIR):
        if file.endswith("mod.rs"):
            continue

        with open(f"{RULE_DIR}/{file}", "r") as f:
            for line in f:
                if line.strip().startswith("rw!("):
                    rws.append(line.strip())

    return rws

def parse_condition(cond: str) -> str:
    fn = cond.split("(")[0]
    # the stuff between the parentheses.
    args = cond.split("(")[1].split(")")[0].replace("\"", "").replace(" ", "").split(",")
    match fn:
        case "crate::trs::is_const_pos":
            assert len(args) == 1, f"Expected 1 argument for is_const_pos, got {len(args)}"
            return f"(> {args[0]} 0)".replace("\"", "")
        case "crate::trs::is_const_neg":
            assert len(args) == 1, f"Expected 1 argument for is_const_neg, got {len(args)}"
            return f"(< {args[0]} 0)".replace("\"", "")
        case "crate::trs::is_not_zero":
            assert len(args) == 1, f"Expected 1 argument for is_not_zero, got {len(args)}"
            # TODO (@ninehusky): should this be (!= {args 0})?
            return f"(! (= {args[0]} 0))"
        case "crate::trs::compare_c0_c1":
            assert len(args) == 3, f"Expected 3 arguments for compare_c0_c1, got {len(args)}"
            comp_op = args[-1]
            match comp_op:
                case "<":
                    return f"(< {args[0]} {args[1]})"
                case "<a":
                    return f"(< {args[0]} (abs {args[1]}))"
                case "<=":
                    return f"(<= {args[0]} {args[1]})"
                case "<=+1":
                    return f"(<= {args[0]} (+ {args[1]} 1))"
                case "<=a":
                    return f"(<= {args[0]} (abs {args[1]}))"
                case "<=-a":
                    return f"(<= {args[0]} (- (abs {args[1]})))"
                case "<=-a+1":
                    return f"(<= {args[0]} (- (abs (+ {args[1]} 1))))"
                case ">":
                    return f"(> {args[0]} {args[1]})"
                case ">a":
                    return f"(> {args[0]} (abs {args[1]}))"
                case ">=":
                    return f"(>= {args[0]} {args[1]})"
                case ">=a":
                    return f"(>= {args[0]} (abs {args[1]}))"
                case ">=a-1":
                    return f"(>= {args[0]} (- (abs {args[1]}) 1))"
                case "!=":
                    return f"(!= {args[0]} {args[1]})"
                case "%0":
                    return f"((&& (!= {args[1]} 0) (== (% {args[0]} {args[1]}) 0))"
                case "!%0":
                    return f"((&& (!= {args[1]} 0) (!= (% {args[0]} {args[1]}) 0))"
                case "%0<":
                    return f"((&& (> {args[1]} 0) (== (% {args[0]} {args[1]}) 0))"
                case "%0>":
                    return f"((&& (< {args[1]} 0) (== (% {args[0]} {args[1]}) 0))"
                case _:
                    raise ValueError(f"Unknown comparison operator {comp_op}")
        case _:
            raise ValueError(f"Unknown function {fn}")


def parse_rewrite(rw: str) -> Rewrite:
    name, rewrite = rw.split(";")
    # make it so that the name is just the stuff between the double quotes.
    name = name.split('"')[1]
    rewrite = re.split(r"=>|if", rewrite)

    assert len(rewrite) in [2, 3], f"{rewrite} has unacceptable number of components: {len(rewrite)}"

    # just get the stuff between the double quotes
    lhs = rewrite[0].split('"')[1]
    rhs = rewrite[1].split('"')[1]
    cond = None if len(rewrite) == 2 else parse_condition(rewrite[2].strip())

    return Rewrite(name, lhs, rhs, cond)

if __name__== "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=str, help="Output file")
    args = parser.parse_args()
    rws = get_rewrites()
    for rw in rws:
        print(parse_rewrite(rw))
