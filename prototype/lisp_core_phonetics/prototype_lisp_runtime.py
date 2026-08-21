"""
My-Lisp Core Runtime & Evaluator with Phonetic Vector Primitives
================================================================

Epistemic Layer: Layer 6 (Engineering & Runtime Model)
Status: Prototype / Language Core Extension

This runtime provides an S-expression reader, macro expander, and evaluator
with native support for unboxed 16-bit Phonetic Vector Codes (PVC-16),
64-bit Pratyāhāra Bitmasks, and reader macros `#pvc(...)` and `#prat(...)`.
"""

import re
from typing import Any, Dict, List, Optional, Tuple, Union
from prototype_pvc16 import (
    PhoneticVector, pvc_make, get_phoneme, REGISTRY,
    FLAG_VOWEL, STHANA_MASK, PRAYATNA_MASK, LEN_MASK, MOD_MASK,
    PRAYATNA_SPRSTA, PRAYATNA_GHOSHA, MOD_PALATALIZED,
)
from prototype_pratyahara import (
    prat_member, prat_mask, prat_intersect, prat_union, prat_diff, prat_subset,
    prat_sounds, sound_code, CANONICAL_SOUNDS, SOUND_TO_CODE, CODE_TO_SOUND,
    PRATYAHARA_MASKS,
)


class LispError(Exception):
    """Base error for Lisp evaluation."""
    pass


class Symbol:
    def __init__(self, name: str):
        self.name = name

    def __eq__(self, other):
        return isinstance(other, Symbol) and self.name == other.name

    def __hash__(self):
        return hash(self.name)

    def __repr__(self):
        return self.name


class Keyword:
    def __init__(self, name: str):
        self.name = name.lstrip(":")

    def __eq__(self, other):
        return isinstance(other, Keyword) and self.name == other.name

    def __hash__(self):
        return hash(self.name)

    def __repr__(self):
        return f":{self.name}"


class Environment:
    def __init__(self, parent: Optional["Environment"] = None):
        self.bindings: Dict[str, Any] = {}
        self.parent = parent

    def define(self, name: str, value: Any):
        self.bindings[name] = value

    def get(self, name: str) -> Any:
        if name in self.bindings:
            return self.bindings[name]
        if self.parent:
            return self.parent.get(name)
        raise LispError(f"unknown symbol: {name}")


# ============================================================================
# READER / PARSER SUPPORTING #pvc(...) AND #prat(...)
# ============================================================================

def tokenize(source: str) -> List[str]:
    """Tokenize source string with reader macro support."""
    # Match strings, comments, reader prefixes, parens, symbols/numbers
    token_spec = [
        ("COMMENT", r";[^\n]*"),
        ("STRING", r'"(?:\\.|[^"\\])*"'),
        ("PRAT_MACRO", r"#prat\b"),
        ("PVC_MACRO", r"#pvc\b"),
        ("LPAREN", r"\("),
        ("RPAREN", r"\)"),
        ("KEYWORD", r":[a-zA-Z0-9_-]+"),
        ("HEXNUM", r"0x[0-9a-fA-F]+"),
        ("NUMBER", r"-?[0-9]+(?:\.[0-9]+)?"),
        ("SYMBOL", r"[^ \t\r\n();\"#]+"),
        ("WS", r"[ \t\r\n]+"),
    ]
    tok_regex = "|".join(f"(?P<{pair[0]}>{pair[1]})" for pair in token_spec)
    tokens = []
    for mo in re.finditer(tok_regex, source):
        kind = mo.lastgroup
        val = mo.group()
        if kind in ("WS", "COMMENT"):
            continue
        tokens.append((kind, val))
    return tokens


class Reader:
    def __init__(self, tokens: List[Tuple[str, str]]):
        self.tokens = tokens
        self.cursor = 0

    def peek(self) -> Optional[Tuple[str, str]]:
        if self.cursor < len(self.tokens):
            return self.tokens[self.cursor]
        return None

    def bump(self) -> Optional[Tuple[str, str]]:
        tok = self.peek()
        if tok:
            self.cursor += 1
        return tok

    def read_all(self) -> List[Any]:
        exprs = []
        while self.peek() is not None:
            exprs.append(self.read_form())
        return exprs

    def read_form(self) -> Any:
        tok = self.peek()
        if tok is None:
            raise LispError("unexpected end of input")

        kind, val = tok

        # Reader Macro: #pvc(...)
        if kind == "PVC_MACRO":
            self.bump()
            inner = self.read_form()
            return self.expand_pvc_macro(inner)

        # Reader Macro: #prat(...)
        if kind == "PRAT_MACRO":
            self.bump()
            inner = self.read_form()
            return self.expand_prat_macro(inner)

        if kind == "LPAREN":
            self.bump()
            items = []
            while True:
                nxt = self.peek()
                if nxt is None:
                    raise LispError("unclosed list")
                if nxt[0] == "RPAREN":
                    self.bump()
                    return items
                items.append(self.read_form())

        if kind == "RPAREN":
            raise LispError("unexpected closing parenthesis")

        if kind == "STRING":
            self.bump()
            return val[1:-1].encode("utf-8").decode("unicode_escape")

        if kind == "KEYWORD":
            self.bump()
            return Keyword(val)

        if kind == "HEXNUM":
            self.bump()
            return int(val, 16)

        if kind == "NUMBER":
            self.bump()
            return float(val) if "." in val else int(val)

        if kind == "SYMBOL":
            self.bump()
            if val == "nil":
                return []
            return Symbol(val)

        raise LispError(f"unknown token: {val}")

    def expand_pvc_macro(self, inner: Any) -> Any:
        """
        Expand #pvc(...) reader syntax.
        Forms:
          #pvc("a") or #pvc(k) -> Lookup phoneme in registry
          #pvc(:vowel t :sthana 1 :prayatna 64 :length 1 :modifier 0) -> pvc_make
          #pvc(0x0142) -> Direct PhoneticVector(0x0142)
        """
        if isinstance(inner, str):
            p = get_phoneme(inner)
            if p:
                return p
            raise LispError(f"Unknown phoneme symbol in #pvc: {inner}")
        if isinstance(inner, Symbol):
            p = get_phoneme(inner.name)
            if p:
                return p
            raise LispError(f"Unknown phoneme symbol in #pvc: {inner.name}")
        if isinstance(inner, int):
            return PhoneticVector(code=inner)
        if isinstance(inner, list):
            # Parse keyword argument pairs
            kwargs = {}
            i = 0
            while i < len(inner):
                key = inner[i]
                if isinstance(key, Keyword):
                    val = inner[i + 1] if i + 1 < len(inner) else None
                    if key.name == "vowel":
                        kwargs["vowel"] = bool(val)
                    elif key.name == "sthana":
                        kwargs["sthana"] = val.name if isinstance(val, Symbol) else int(val)
                    elif key.name == "prayatna":
                        kwargs["prayatna"] = int(val)
                    elif key.name == "length":
                        kwargs["length"] = int(val)
                    elif key.name == "modifier":
                        kwargs["modifier"] = int(val)
                    i += 2
                elif i == 0 and isinstance(key, (str, Symbol)):
                    sym = key.name if isinstance(key, Symbol) else key
                    p = get_phoneme(sym)
                    if p:
                        return p
                    i += 1
                elif i == 0 and isinstance(key, int):
                    return PhoneticVector(code=key)
                else:
                    i += 1
            return pvc_make(**kwargs)

        raise LispError(f"Invalid #pvc payload: {inner}")

    def expand_prat_macro(self, inner: Any) -> Any:
        """
        Expand #prat(...) reader syntax into compile-time 64-bit integer mask.
        Forms:
          #prat(ac) -> 0x00000000000001FF
          #prat("hal") -> 0x000003FFFFFFFFFE00
          #prat((a i u f x)) -> Mask computed from sounds
        """
        if isinstance(inner, Symbol):
            return prat_mask(inner.name)
        if isinstance(inner, str):
            return prat_mask(inner)
        if isinstance(inner, list):
            if len(inner) == 1 and isinstance(inner[0], (Symbol, str)):
                name = inner[0].name if isinstance(inner[0], Symbol) else inner[0]
                if name in PRATYAHARA_MASKS:
                    return prat_mask(name)
            sound_strs = [s.name if isinstance(s, Symbol) else str(s) for s in inner]
            return prat_mask(sound_strs)
        raise LispError(f"Invalid #prat payload: {inner}")


def parse(source: str) -> List[Any]:
    tokens = tokenize(source)
    reader = Reader(tokens)
    return reader.read_all()


# ============================================================================
# LISP EVALUATOR & BUILT-IN PHONETIC PRIMITIVES
# ============================================================================

def create_global_env() -> Environment:
    env = Environment()
    env.define("t", True)
    env.define("nil", [])

    # Core Equality & Logic
    env.define("eq", lambda a, b: a == b or (isinstance(a, PhoneticVector) and isinstance(b, PhoneticVector) and a.code == b.code))
    env.define("equal?", lambda a, b: a == b or (isinstance(a, PhoneticVector) and isinstance(b, PhoneticVector) and a.code == b.code))
    env.define("not", lambda a: not a if a is not False and a != [] else True)
    env.define("null?", lambda a: a == [] or a is None)

    # List Primitives
    env.define("cons", lambda a, b: [a] + b if isinstance(b, list) else [a, b])
    env.define("car", lambda lst: lst[0] if isinstance(lst, list) and lst else None)
    env.define("cdr", lambda lst: lst[1:] if isinstance(lst, list) and lst else [])
    env.define("list", lambda *args: list(args))
    env.define("length", lambda lst: len(lst) if isinstance(lst, list) else 0)

    # Arithmetic
    env.define("+", lambda *args: sum(args))
    env.define("-", lambda a, b=None: -a if b is None else a - b)
    env.define("*", lambda *args: (1 if not args else eval_prod(args)))
    env.define("/", lambda a, b: a / b if isinstance(a, float) or isinstance(b, float) else a // b)
    env.define("<", lambda a, b: a < b)
    env.define(">", lambda a, b: a > b)
    env.define("=", lambda a, b: a == b)

    # ========================================================================
    # PHONETIC PRIMITIVES (PVC-16 & PRATYĀHĀRA ENGINE)
    # ========================================================================

    # 1. pvc-make
    def lisp_pvc_make(*args) -> PhoneticVector:
        """
        (pvc-make :vowel bool :sthana int :prayatna int :length int :modifier int)
        or (pvc-make vowel sthana prayatna length modifier)
        """
        if len(args) == 0:
            return pvc_make()
        # Keyword arguments format
        if isinstance(args[0], Keyword):
            kwargs = {}
            i = 0
            while i < len(args):
                k = args[i]
                if isinstance(k, Keyword):
                    v = args[i + 1] if i + 1 < len(args) else None
                    if k.name == "vowel":
                        kwargs["vowel"] = bool(v)
                    elif k.name == "sthana":
                        kwargs["sthana"] = v.name if isinstance(v, Symbol) else int(v)
                    elif k.name == "prayatna":
                        kwargs["prayatna"] = int(v)
                    elif k.name == "length":
                        kwargs["length"] = int(v)
                    elif k.name == "modifier":
                        kwargs["modifier"] = int(v)
                    i += 2
                else:
                    i += 1
            return pvc_make(**kwargs)
        # Positional format: (pvc-make vowel sthana prayatna length modifier)
        vowel = bool(args[0]) if len(args) > 0 else False
        sthana = args[1] if len(args) > 1 else 0
        prayatna = int(args[2]) if len(args) > 2 else 0
        length = int(args[3]) if len(args) > 3 else 0
        modifier = int(args[4]) if len(args) > 4 else 0
        return pvc_make(vowel=vowel, sthana=sthana, prayatna=prayatna, length=length, modifier=modifier)

    env.define("pvc-make", lisp_pvc_make)

    # 2. Savarṇa Homogeneity Check: (savarna? p1 p2) -> Sūtra 1.1.9
    def lisp_savarna(p1: Any, p2: Any) -> bool:
        v1 = to_phonetic_vector(p1)
        v2 = to_phonetic_vector(p2)
        if v1 and v2:
            return v1.is_savarna_with(v2)
        return False

    env.define("savarna?", lisp_savarna)
    env.define("is-savarna?", lisp_savarna)

    # 3. Pratyāhāra Membership: (prat-member? sound-code mask-64)
    def lisp_prat_member(sound: Any, mask_or_name: Any) -> bool:
        s_val = sound.name if isinstance(sound, Symbol) else sound
        m_val = mask_or_name.name if isinstance(mask_or_name, Symbol) else mask_or_name
        return prat_member(s_val, m_val)

    env.define("prat-member?", lisp_prat_member)

    # 4. Bitwise Transformations: (sandhi-voice sound), (palatalize sound)
    def lisp_sandhi_voice(sound: Any) -> PhoneticVector:
        v = to_phonetic_vector(sound)
        if not v:
            raise LispError(f"sandhi-voice expects a phonetic vector, got: {sound}")
        return v.with_voicing(True)

    def lisp_sandhi_devoice(sound: Any) -> PhoneticVector:
        v = to_phonetic_vector(sound)
        if not v:
            raise LispError(f"sandhi-devoice expects a phonetic vector, got: {sound}")
        return v.with_voicing(False)

    def lisp_palatalize(sound: Any) -> PhoneticVector:
        v = to_phonetic_vector(sound)
        if not v:
            raise LispError(f"palatalize expects a phonetic vector, got: {sound}")
        return v.with_palatalization(True)

    def lisp_depalatalize(sound: Any) -> PhoneticVector:
        v = to_phonetic_vector(sound)
        if not v:
            raise LispError(f"depalatalize expects a phonetic vector, got: {sound}")
        return v.with_palatalization(False)

    env.define("sandhi-voice", lisp_sandhi_voice)
    env.define("sandhi-devoice", lisp_sandhi_devoice)
    env.define("palatalize", lisp_palatalize)
    env.define("depalatalize", lisp_depalatalize)

    # 5. Pratyāhāra Set Algebra
    env.define("prat-mask", lambda name: prat_mask(name.name if isinstance(name, Symbol) else name))
    env.define("prat-intersect", prat_intersect)
    env.define("prat-union", prat_union)
    env.define("prat-diff", prat_diff)
    env.define("prat-subset?", prat_subset)
    env.define("prat-sounds", lambda m: [Symbol(s) for s in prat_sounds(m)])

    # 6. PVC Field Accessors
    env.define("pvc-code", lambda p: to_phonetic_vector(p).code)
    env.define("pvc-vowel?", lambda p: to_phonetic_vector(p).is_vowel)
    env.define("pvc-consonant?", lambda p: to_phonetic_vector(p).is_consonant)
    env.define("pvc-sthana", lambda p: to_phonetic_vector(p).sthana)
    env.define("pvc-sthana-name", lambda p: Symbol(to_phonetic_vector(p).sthana_name))
    env.define("pvc-prayatna", lambda p: to_phonetic_vector(p).prayatna)
    env.define("pvc-length", lambda p: to_phonetic_vector(p).length)
    env.define("pvc-voiced?", lambda p: to_phonetic_vector(p).is_voiced)
    env.define("pvc-aspirate?", lambda p: to_phonetic_vector(p).is_aspirate)
    env.define("pvc-sprsta?", lambda p: to_phonetic_vector(p).is_sprsta)
    env.define("pvc-nasal?", lambda p: to_phonetic_vector(p).is_nasal)
    env.define("pvc-palatalized?", lambda p: to_phonetic_vector(p).is_palatalized)
    env.define("pvc-from-sym", lambda s: get_phoneme(s.name if isinstance(s, Symbol) else str(s)))

    return env


def eval_prod(lst):
    p = 1
    for x in lst:
        p *= x
    return p


def to_phonetic_vector(val: Any) -> Optional[PhoneticVector]:
    """Coerce value to PhoneticVector."""
    if isinstance(val, PhoneticVector):
        return val
    if isinstance(val, int):
        return PhoneticVector(code=val)
    if isinstance(val, Symbol):
        return get_phoneme(val.name)
    if isinstance(val, str):
        return get_phoneme(val)
    return None


def evaluate(expr: Any, env: Environment) -> Any:
    """Evaluate Lisp expression in environment."""
    if isinstance(expr, (int, float, str, bool, PhoneticVector, Keyword)):
        return expr
    if expr == []:
        return []
    if isinstance(expr, Symbol):
        return env.get(expr.name)

    if isinstance(expr, list):
        if not expr:
            return []
        head = expr[0]

        # Special form: (quote ...)
        if head == Symbol("quote"):
            if len(expr) != 2:
                raise LispError("quote expects 1 argument")
            return expr[1]

        # Special form: (def name val)
        if head == Symbol("def"):
            if len(expr) != 3 or not isinstance(expr[1], Symbol):
                raise LispError("def expects symbol and value")
            val = evaluate(expr[2], env)
            env.define(expr[1].name, val)
            return val

        # Special form: (lambda (params...) body...)
        if head == Symbol("lambda"):
            params = [p.name if isinstance(p, Symbol) else str(p) for p in expr[1]]
            body = expr[2:]
            return lambda *args: eval_lambda(params, body, args, env)

        # Special form: (cond (test1 expr1...) (test2 expr2...) ...)
        if head == Symbol("cond"):
            for clause in expr[1:]:
                if not isinstance(clause, list) or len(clause) < 2:
                    raise LispError("invalid cond clause")
                test_res = evaluate(clause[0], env)
                if test_res not in (False, [], None):
                    res = None
                    for b in clause[1:]:
                        res = evaluate(b, env)
                    return res
            return []

        # Function application
        fn = evaluate(head, env)
        args = [evaluate(arg, env) for arg in expr[1:]]
        if callable(fn):
            return fn(*args)
        raise LispError(f"not a callable function: {head}")

    raise LispError(f"unknown expression: {expr}")


def eval_lambda(params: List[str], body: List[Any], args: Tuple[Any, ...], parent_env: Environment) -> Any:
    call_env = Environment(parent_env)
    for p, a in zip(params, args):
        call_env.define(p, a)
    res = []
    for form in body:
        res = evaluate(form, call_env)
    return res


def eval_string(source: str, env: Optional[Environment] = None) -> Any:
    """Read and evaluate string."""
    if env is None:
        env = create_global_env()
    forms = parse(source)
    res = None
    for form in forms:
        res = evaluate(form, env)
    return res
