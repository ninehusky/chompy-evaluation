from lark import Lark, Transformer, Token

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

parser = Lark(grammar, parser="lalr", transformer=ToSExpr())

# Example expressions
for expr in [
    "f(1, 2 + 3)",
    "add(mul(2, 3), div(4, 2))",
    "outer(inner1(1), inner2(2, 3), 4)",
    "!(x == 2 || f(3) > 1) && y < 5"
]:
    print(f"Input: {expr}")
    print("Parsed:", parser.parse(expr))
    print()

expr = "!(x == 2 || f(3) > 1) && y < 5"
print(parser.parse(expr))
