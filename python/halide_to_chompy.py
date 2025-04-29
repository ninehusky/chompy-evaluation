from lark import Lark, Transformer, Token
import argparse
import re

grammar = r"""
    ?start: expr

    ?expr: logic_expr

    ?logic_expr: logic_expr "&&" logic_term   -> and_
               | logic_expr "||" logic_term   -> or_
               | logic_term

    ?logic_term: "!" logic_term               -> not_
               | comp_expr

    ?comp_expr: sum_expr COMPOP sum_expr      -> cmp
              | sum_expr

    ?sum_expr: sum_expr "+" term              -> add
             | sum_expr "-" term              -> sub
             | term

    ?term: term "*" factor                    -> mul
         | term "/" factor                    -> div
         | factor

    ?factor: NUMBER                           -> number
            | "-" factor                      -> neg
           | func_call
           | NAME                             -> var
           | "(" expr ")"                     -> parens

    func_call: NAME "(" args? ")"
    args: expr ("," expr)*

    COMPOP: "==" | "!=" | "<" | "<=" | ">" | ">="

    %import common.CNAME -> NAME
    %import common.NUMBER
    %import common.WS
    %ignore WS
"""


class ToSExpr(Transformer):
    def neg(self, items):
        return f"(- {items[0]})"


    def add(self, items):
        return f"(+ {items[0]} {items[1]})"

    def sub(self, items):
        return f"(- {items[0]} {items[1]})"

    def mul(self, items):
        return f"(* {items[0]} {items[1]})"

    def div(self, items):
        return f"(/ {items[0]} {items[1]})"
    
    def or_(self, items):
        return f"(|| {items[0]} {items[1]})"

    def and_(self, items):
        return f"(&& {items[0]} {items[1]})"

    def not_(self, items):
        return f"(! {items[0]})"

    def cmp(self, items):
        op = items[1].value
        return f"({op} {items[0]} {items[2]})"

    def number(self, token):
        return token[0].value

    def var(self, token):
        return token[0].value

    def parens(self, items):
        return items[0]

    def func_call(self, items):
        name = items[0]
        args = items[1] if len(items) > 1 else []
        return f"({name} {' '.join(args)})"

    def args(self, items):
        return items

if __name__ == "__main__":
    argparser = argparse.ArgumentParser(description="Convert Halide expressions to S-expressions.")
    argparser.add_argument("input", type=str, help="Input file containing Halide expressions.")
    argparser.add_argument("output", type=str, help="Output file for S-expressions.")

    args = argparser.parse_args()
    rules = []
    with open(args.input, "r") as f:
        parser = Lark(grammar, parser="lalr", transformer=ToSExpr())
        delimiters = [" if ", " ==> "]
        for line in f.readlines():
            for delim in delimiters:
                line = line.replace(delim, ";")
            
            parts = line.split(";")
            assert len(parts) == 3, "Expected lhs ==> rhs if cond, got: " + line

            lhs, rhs, cond = parts

            parsed_rule = f"{parser.parse(lhs)} ==> {parser.parse(rhs)} if {parser.parse(cond)}"
            rules.append(parsed_rule)
    
    with open(args.output, "w") as f:
        for rule in rules:
            f.write(rule + "\n")





