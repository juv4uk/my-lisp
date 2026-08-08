use crate::bignum::BigInt;
use crate::{Environment, Expr};
use std::{cmp::Ordering, fmt, rc::Rc, str::FromStr};

/// A reduced exact fraction owned by the language runtime, backed by the
/// hand-rolled `BigInt` in `bignum.rs` — "exact" has no numeric ceiling
/// short of available memory. Rust does this low-level numeric algorithm,
/// the same way it already did the bounded `i64` version this replaced;
/// my-lisp itself never grows an arithmetic primitive (see
/// docs/language-core.md). `denominator` is always positive and the
/// fraction always reduced — the invariant `from_big` maintains on every
/// construction path.
/// Скорочений точний дріб, яким володіє runtime мови, на основі власноруч
/// написаного `BigInt` у `bignum.rs` — "точний" не має числової стелі,
/// окрім доступної пам'яті. Rust робить цей низькорівневий числовий
/// алгоритм так само, як уже робив обмежену `i64`-версію, яку це замінило;
/// сама my-lisp ніколи не розширює арифметичний примітив (див.
/// docs/language-core.md). `denominator` завжди додатний, а дріб завжди
/// скорочений — інваріант, який `from_big` підтримує на кожному шляху
/// побудови.
/// Ein gekürzter exakter Bruch im Besitz der Sprachlaufzeit, basierend auf
/// dem von Hand geschriebenen `BigInt` in `bignum.rs` — "exakt" hat keine
/// numerische Obergrenze außer dem verfügbaren Speicher. Rust erledigt
/// diesen Low-Level-Zahlenalgorithmus, genauso wie es bereits die
/// begrenzte `i64`-Version tat, die dies ersetzt; my-lisp selbst erweitert
/// nie ein arithmetisches Primitiv (siehe docs/language-core.md).
/// `denominator` ist immer positiv und der Bruch immer gekürzt — die
/// Invariante, die `from_big` bei jedem Konstruktionspfad aufrechterhält.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rational {
    numerator: BigInt,
    denominator: BigInt,
}

impl PartialOrd for Rational {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Rational {
    fn cmp(&self, other: &Self) -> Ordering {
        // Denominators are always positive (see `from_big`), so comparing by
        // cross-multiplication is exact — no float involved, no rounding.
        // Знаменники завжди додатні (див. `from_big`), тож порівняння
        // хрест-навхрест точне — без float, без округлення.
        // Nenner sind immer positiv (siehe `from_big`), daher ist der
        // Vergleich per Kreuzmultiplikation exakt — kein Float, keine Rundung.
        self.numerator
            .mul(&other.denominator)
            .cmp(&other.numerator.mul(&self.denominator))
    }
}

impl Rational {
    pub fn new(numerator: i64, denominator: i64) -> Option<Self> {
        Self::from_big(BigInt::from_i64(numerator), BigInt::from_i64(denominator))
    }

    /// Parses a `numerator/denominator` literal directly as arbitrary-precision
    /// integers, for source tokens too large for `i64` (see `parser.rs`).
    /// Парсить літерал `чисельник/знаменник` напряму як цілі довільної
    /// точності, для токенів коду, завеликих для `i64` (див. `parser.rs`).
    /// Parst ein `Zähler/Nenner`-Literal direkt als beliebig genaue Ganzzahlen,
    /// für Quelltoken, die zu groß für `i64` sind (siehe `parser.rs`).
    pub fn from_literal(numerator: &str, denominator: &str) -> Option<Self> {
        let numerator = BigInt::from_str(numerator).ok()?;
        let denominator = BigInt::from_str(denominator).ok()?;
        Self::from_big(numerator, denominator)
    }

    fn from_big(numerator: BigInt, denominator: BigInt) -> Option<Self> {
        if denominator.is_zero() {
            return None;
        }
        let (numerator, denominator) = if denominator.is_negative() {
            (numerator.neg(), denominator.neg())
        } else {
            (numerator, denominator)
        };
        let divisor = numerator.gcd(&denominator);
        if divisor.is_zero() {
            // Only when numerator is also zero (gcd(0, d) = d otherwise);
            // 0/d reduces to the canonical 0/1 without a division step.
            return Some(Self {
                numerator: BigInt::zero(),
                denominator: BigInt::from_i64(1),
            });
        }
        let (numerator, _) = numerator.div_rem(&divisor)?;
        let (denominator, _) = denominator.div_rem(&divisor)?;
        Some(Self {
            numerator,
            denominator,
        })
    }

    pub fn integer(value: i64) -> Self {
        Self {
            numerator: BigInt::from_i64(value),
            denominator: BigInt::from_i64(1),
        }
    }

    pub fn checked_div(self, divisor: Self) -> Option<Self> {
        if divisor.numerator.is_zero() {
            return None;
        }
        Self::from_big(
            self.numerator.mul(&divisor.denominator),
            self.denominator.mul(&divisor.numerator),
        )
    }

    pub fn checked_add(self, other: Self) -> Option<Self> {
        let numerator = self
            .numerator
            .mul(&other.denominator)
            .add(&other.numerator.mul(&self.denominator));
        Self::from_big(numerator, self.denominator.mul(&other.denominator))
    }

    pub fn checked_sub(self, other: Self) -> Option<Self> {
        let numerator = self
            .numerator
            .mul(&other.denominator)
            .sub(&other.numerator.mul(&self.denominator));
        Self::from_big(numerator, self.denominator.mul(&other.denominator))
    }

    pub fn checked_mul(self, other: Self) -> Option<Self> {
        Self::from_big(
            self.numerator.mul(&other.numerator),
            self.denominator.mul(&other.denominator),
        )
    }

    pub fn checked_neg(self) -> Option<Self> {
        Some(Self {
            numerator: self.numerator.neg(),
            denominator: self.denominator,
        })
    }

    pub fn as_f64(&self) -> f64 {
        self.numerator.to_f64() / self.denominator.to_f64()
    }

    pub fn is_integer(&self) -> bool {
        self.denominator.to_i64() == Some(1)
    }

    /// `Some(n)` only if this exact value is a whole number *and* representable
    /// as an `i64` within `f64`'s 2^53 exact-integer range — the one case
    /// `crates/my-lisp/src/eval/arithmetic.rs`'s `exact_value` may cosmetically
    /// print through `Value::Number` instead of `Value::Rational` without ever
    /// losing precision doing so. Anything bigger stays `Value::Rational` (see
    /// `Display`, which omits `/1` for whole numbers) rather than risk exactly
    /// the silent-approximation the exact-number principle forbids.
    /// `Some(n)`, лише якщо це ціле значення *і* влазить в `i64` в межах
    /// 2^53-діапазону точних цілих `f64` — єдиний випадок, коли `exact_value`
    /// у `crates/my-lisp/src/eval/arithmetic.rs` може косметично друкувати
    /// через `Value::Number` замість `Value::Rational`, не втрачаючи точність.
    /// Усе більше лишається `Value::Rational` (див. `Display`, що пропускає
    /// `/1` для цілих чисел), а не ризикує саме тим тихим наближенням, яке
    /// забороняє принцип точних чисел.
    /// `Some(n)` nur, wenn dieser exakte Wert eine ganze Zahl ist *und* als
    /// `i64` innerhalb von `f64`s 2^53-Bereich exakter Ganzzahlen darstellbar
    /// — der eine Fall, in dem `exact_value` in
    /// `crates/my-lisp/src/eval/arithmetic.rs` kosmetisch über `Value::Number`
    /// statt `Value::Rational` drucken darf, ohne dabei Genauigkeit zu
    /// verlieren. Alles Größere bleibt `Value::Rational` (siehe `Display`,
    /// das `/1` bei Ganzzahlen weglässt), statt genau die stille Approximation
    /// zu riskieren, die das Prinzip exakter Zahlen verbietet.
    pub fn as_precise_i64(&self) -> Option<i64> {
        if !self.is_integer() {
            return None;
        }
        const MAX_EXACT: i64 = 1 << 53;
        let value = self.numerator.to_i64()?;
        (-MAX_EXACT..=MAX_EXACT).contains(&value).then_some(value)
    }
}

impl fmt::Display for Rational {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_integer() {
            write!(formatter, "{}", self.numerator)
        } else {
            write!(formatter, "{}/{}", self.numerator, self.denominator)
        }
    }
}

/// A closure keeps executable forms together with their lexical environment.
/// Замикання зберігає виконувані форми разом із їхнім лексичним середовищем.
/// Eine Closure bewahrt ausführbare Formen zusammen mit ihrer lexikalischen Umgebung auf.
#[derive(Clone, Debug)]
pub struct Closure {
    pub(crate) parameters: Vec<Rc<str>>,
    pub(crate) body: Rc<[Expr]>,
    pub(crate) environment: Environment,
}

/// Runtime data is independent of the parser and any host representation.
/// Дані виконання не залежать від парсера та представлення у хост-системі.
/// Laufzeitdaten sind unabhängig vom Parser und von jeder Host-Darstellung.
#[derive(Clone, Debug)]
pub enum Value {
    Nil,
    Bool(bool),
    Number(f64),
    Rational(Rational),
    String(Rc<str>),
    Symbol(Rc<str>),
    Pair(Rc<Value>, Rc<Value>),
    Closure(Rc<Closure>),
    Macro(Rc<Closure>),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Nil, Value::Nil) => true,
            (Value::Bool(left), Value::Bool(right)) => left == right,
            (Value::Number(left), Value::Number(right)) => left == right,
            (Value::Rational(left), Value::Rational(right)) => left == right,
            (Value::String(left), Value::String(right)) => left == right,
            (Value::Symbol(left), Value::Symbol(right)) => left == right,
            (Value::Pair(left_head, left_tail), Value::Pair(right_head, right_tail)) => {
                left_head == right_head && left_tail == right_tail
            }
            // Functions have identity: two separately created closures are not equal.
            // Функції мають ідентичність: два окремо створені замикання не є рівними.
            // Funktionen besitzen Identität: Zwei getrennt erzeugte Closures sind nicht gleich.
            (Value::Closure(left), Value::Closure(right)) => Rc::ptr_eq(left, right),
            (Value::Macro(left), Value::Macro(right)) => Rc::ptr_eq(left, right),
            _ => false,
        }
    }
}

impl Value {
    pub fn list(values: impl IntoIterator<Item = Value>) -> Self {
        values
            .into_iter()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .fold(Value::Nil, |tail, head| {
                Value::Pair(Rc::new(head), Rc::new(tail))
            })
    }

    pub fn is_atom(&self) -> bool {
        !matches!(self, Value::Pair(_, _))
    }

    pub fn is_truthy(&self) -> bool {
        !matches!(self, Value::Nil | Value::Bool(false))
    }
}

impl fmt::Display for Value {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Nil => write!(formatter, "()"),
            Value::Bool(true) => write!(formatter, "t"),
            Value::Bool(false) => write!(formatter, "()"),
            Value::Number(number) => write!(formatter, "{number}"),
            Value::Rational(number) => write!(formatter, "{number}"),
            Value::String(value) => write!(formatter, "\"{value}\""),
            Value::Symbol(symbol) => write!(formatter, "{symbol}"),
            Value::Pair(_, _) => write_pair(formatter, self),
            Value::Closure(_) => write!(formatter, "<lambda>"),
            Value::Macro(_) => write!(formatter, "<macro>"),
        }
    }
}

fn write_pair(formatter: &mut fmt::Formatter<'_>, value: &Value) -> fmt::Result {
    write!(formatter, "(")?;
    let mut current = value;
    let mut first = true;
    loop {
        match current {
            Value::Pair(head, tail) => {
                if !first {
                    write!(formatter, " ")?;
                }
                write!(formatter, "{head}")?;
                current = tail;
                first = false;
            }
            Value::Nil => return write!(formatter, ")"),
            tail => return write!(formatter, " . {tail})"),
        }
    }
}

impl Drop for Value {
    fn drop(&mut self) {
        if !matches!(self, Value::Pair(_, _)) {
            return;
        }

        let mut worklist = Vec::new();
        worklist.push(std::mem::replace(self, Value::Nil));

        while let Some(value) = worklist.pop() {
            let mut value = std::mem::ManuallyDrop::new(value);
            match &mut *value {
                Value::Pair(head, tail) => {
                    let head = unsafe { std::ptr::read(head) };
                    let tail = unsafe { std::ptr::read(tail) };
                    if let Ok(inner) = Rc::try_unwrap(head) {
                        if matches!(inner, Value::Pair(_, _)) {
                            worklist.push(inner);
                        }
                    }
                    if let Ok(inner) = Rc::try_unwrap(tail) {
                        if matches!(inner, Value::Pair(_, _)) {
                            worklist.push(inner);
                        }
                    }
                }
                _ => {
                    unsafe { std::mem::ManuallyDrop::drop(&mut value) };
                }
            }
        }
    }
}
