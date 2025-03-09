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
    pass

def parse_rewrite(rw: str) -> Rewrite:
    name, rewrite = rw.split(";")
    # make it so that the name is just the stuff between the double quotes.
    name = name.split('"')[1]
    rewrite = re.split(r"=>|if", rewrite)

    assert len(rewrite) in [2, 3], f"{rewrite} has unacceptable number of components: {len(rewrite)}"

    # just get the stuff between the double quotes
    lhs = rewrite[0].split('"')[1]
    rhs = rewrite[1].split('"')[1]
    cond = None if len(rewrite) == 2 else rewrite[2].strip()

    return Rewrite(name, lhs, rhs, cond)



if __name__== "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=str, help="Output file")
    args = parser.parse_args()
    rws = get_rewrites()
    for rw in rws:
        print(parse_rewrite(rw))
