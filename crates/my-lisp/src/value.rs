use crate::bignum::BigInt;
use crate::{Environment, Exactness, Expr};
use std::{cell::RefCell, cmp::Ordering, fmt, net::TcpListener, net::TcpStream, rc::Rc, str::FromStr};

/// A reduced exact fraction owned by the language runtime, backed by the
/// hand-rolled `BigInt` in `bignum.rs` — "exact" has no numeric ceiling
/// short of available memory. Rust does this low-level numeric algorithm,
/// the same way it already did the bounded `i64` version this replaced;
/// my-lisp itself never grows an arithmetic primitive (see
/// docs/language-core.md). `denominator` is always positive and the
/// fraction always reduced — the invariant `from_big` maintains on every
/// construction path.
/// Skorochenyi tochnyi drib, yakym volodiie runtime movy, na osnovi vlasnoruch
/// napysanoho `BigInt` u `bignum.rs` — "tochnyi" ne maie chyslovoi steli,
/// okrim dostupnoi pamiati. Rust robyt tsei nyzkorivnevyi chyslovyi
/// alhorytm tak samo, yak uzhe robyv obmezhenu `i64`-versiiu, yaku tse zaminylo;
/// sama my-lisp nikoly ne rozshyriuie aryfmetychnyi prymityv (dyv.
/// docs/language-core.md). `denominator` zavzhdy dodatnyi, a drib zavzhdy
/// skorochenyi — invariant, yakyi `from_big` pidtrymuie na kozhnomu shliakhu
/// pobudovy.
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
        // Znamennyky zavzhdy dodatni (dyv. `from_big`), tozh porivniannia
        // khrest-navkhrest tochne — bez float, bez okruhlennia.
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
    /// Parsyt literal `chyselnyk/znamennyk` napriamu yak tsili dovilnoi
    /// tochnosti, dlia tokeniv kodu, zavelykykh dlia `i64` (dyv. `parser.rs`).
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

    /// The wider of numerator/denominator bit width — see `BigInt::bit_length`
    /// for why this exists (an opt-in resource-limit check, not ordinary use).
    /// Shyrsha z dvokh velychyn chyselnyka/znamennyka v bitakh — dyv.
    /// `BigInt::bit_length`, chomu tse isnuie (optsiina perevirka obmezhennia
    /// resursu, ne zvychaine vykorystannia).
    pub fn bit_length(&self) -> usize {
        self.numerator.bit_length().max(self.denominator.bit_length())
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
    /// `Some(n)`, lyshe yakshcho tse tsile znachennia *i* vlazyt v `i64` v mezhakh
    /// 2^53-diapazonu tochnykh tsilykh `f64` — yedynyi vypadok, koly `exact_value`
    /// u `crates/my-lisp/src/eval/arithmetic.rs` mozhe kosmetychno drukuvaty
    /// cherez `Value::Number` zamist `Value::Rational`, ne vtrachaiuchy tochnist.
    /// Use bilshe lyshaietsia `Value::Rational` (dyv. `Display`, shcho propuskaie
    /// `/1` dlia tsilykh chysel), a ne ryzykuie same tym tykhym nablyzhenniam, yake
    /// zaboroniaie pryntsyp tochnykh chysel.
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
/// Zamykannia zberihaie vykonuvani formy razom iz yikhnim leksychnym seredovyshchem.
/// Eine Closure bewahrt ausführbare Formen zusammen mit ihrer lexikalischen Umgebung auf.
#[derive(Clone, Debug)]
pub struct Closure {
    pub(crate) parameters: Vec<Rc<str>>,
    /// The dotted-list/bare-symbol rest parameter, if any — bound to a list
    /// of every argument past `parameters.len()`. `None` means the closure
    /// takes exactly `parameters.len()` arguments, no more.
    /// Variatyvnyi parametr (dotted-list/holyi symvol), yakshcho ye — zviazuietsia
    /// zi spyskom usikh arhumentiv ponad `parameters.len()`. `None` oznachaie,
    /// shcho zamykannia pryimaie tochno `parameters.len()` arhumentiv, ne bilshe.
    /// Der variadische Rest-Parameter (Dotted-List/nacktes Symbol), falls
    /// vorhanden — gebunden an eine Liste aller Argumente über
    /// `parameters.len()` hinaus. `None` bedeutet, die Closure nimmt genau
    /// `parameters.len()` Argumente, nicht mehr.
    pub(crate) rest: Option<Rc<str>>,
    pub(crate) body: Rc<[Expr]>,
    pub(crate) environment: Environment,
}

/// Runtime data is independent of the parser and any host representation.
/// Dani vykonannia ne zalezhat vid parsera ta predstavlennia u khost-systemi.
/// Laufzeitdaten sind unabhängig vom Parser und von jeder Host-Darstellung.
#[derive(Clone, Debug)]
pub enum Value {
    Nil,
    Bool(bool),
    Number(f64, Exactness),
    Rational(Rational),
    String(Rc<str>),
    Symbol(Rc<str>),
    Pair(Rc<Value>, Rc<Value>),
    Closure(Rc<Closure>),
    Macro(Rc<Closure>),
    /// An open TCP connection (PLAN.md item 21) — the outbound-client half
    /// of "talk to other AI systems," principle 3 extended to external
    /// agents/LLM APIs. Opaque host-capability handle, the same category
    /// `read-file`/`write-file` already occupy — not user-visible mutable
    /// *data* in the item-16 sense (nothing here lets a program mutate an
    /// ordinary binding), just a resource the host manages on the
    /// language's behalf. `RefCell` because reading/writing a stream
    /// genuinely advances its position — there is no persistent,
    /// side-effect-free way to model "the next unread byte of a live
    /// network connection."
    /// Vidkryte TCP-ziednannia (PLAN.md, punkt 21) — vykhidna/kliientska
    /// polovyna "spilkuvatys z inshymy AI-systemamy", pryntsyp 3, poshyrenyi
    /// na zovnishnikh ahentiv/LLM API. Neprozoryi handle host-mozhlyvosti, ta
    /// sama katehoriia, shcho vzhe zaimaiut `read-file`/`write-file` — ne
    /// vydyma korystuvachu mutabelna *dani* v sensi item 16 (nishcho tut ne
    /// daie prohrami mutuvaty zvychaine zviazuvannia), lyshe resurs, yakym khost
    /// keruie vid imeni movy. `RefCell`, bo chytannia/zapys u potik spravdi
    /// prosuvaie yoho pozytsiiu — nemaie postiinoho, bez-pobichno-efektnoho
    /// sposobu zmodeliuvaty "nastupnyi neprochytanyi bait zhyvoho merezhevoho
    /// ziednannia".
    TcpConnection(Rc<RefCell<TcpStream>>),
    /// A listening TCP socket (PLAN.md item 21) — the inbound-server half:
    /// lets my-lisp accept connections from other agents, not just call
    /// out to them. No `RefCell` needed — `TcpListener::accept` takes
    /// `&self`, it doesn't need to mutate the listener itself.
    /// TCP-soket, shcho slukhaie (PLAN.md, punkt 21) — vkhidna/serverna
    /// polovyna: dozvoliaie my-lisp pryimaty ziednannia vid inshykh ahentiv, ne
    /// lyshe zvertatys do nykh. `RefCell` ne potriben — `TcpListener::accept`
    /// bere `&self`, ne potrebuie mutuvaty sam listener.
    TcpListener(Rc<TcpListener>),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Nil, Value::Nil) => true,
            (Value::Bool(left), Value::Bool(right)) => left == right,
            // Exactness is part of a number's identity (PLAN.md item 10, Path
            // A): (eq 3 3.0) is () because these are different values in the
            // model, even though (= 3 3.0) is t (same magnitude). See `=` in
            // arithmetic.rs for the magnitude-only comparison.
            // Exactness — chastyna identychnosti chysla (PLAN.md, punkt 10,
            // shliakh A): (eq 3 3.0) daie (), bo tse rizni znachennia v modeli,
            // khocha (= 3 3.0) daie t (ta sama velychyna). Dyv. `=` v
            // arithmetic.rs dlia porivniannia lyshe za velychynoiu.
            (Value::Number(left, left_exactness), Value::Number(right, right_exactness)) => {
                left == right && left_exactness == right_exactness
            }
            (Value::Rational(left), Value::Rational(right)) => left == right,
            (Value::String(left), Value::String(right)) => left == right,
            (Value::Symbol(left), Value::Symbol(right)) => left == right,
            (Value::Pair(left_head, left_tail), Value::Pair(right_head, right_tail)) => {
                left_head == right_head && left_tail == right_tail
            }
            // Functions have identity: two separately created closures are not equal.
            // Funktsii maiut identychnist: dva okremo stvoreni zamykannia ne ye rivnymy.
            // Funktionen besitzen Identität: Zwei getrennt erzeugte Closures sind nicht gleich.
            (Value::Closure(left), Value::Closure(right)) => Rc::ptr_eq(left, right),
            (Value::Macro(left), Value::Macro(right)) => Rc::ptr_eq(left, right),
            // Same identity rule as Closure/Macro — a resource handle is
            // itself, not a value with structural equality.
            // Te same pravylo identychnosti, shcho y Closure/Macro — handle
            // resursu — tse vin sam, ne znachennia zi strukturnoiu rivnistiu.
            (Value::TcpConnection(left), Value::TcpConnection(right)) => Rc::ptr_eq(left, right),
            (Value::TcpListener(left), Value::TcpListener(right)) => Rc::ptr_eq(left, right),
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
        write!(formatter, "{}", render(self, true))
    }
}

impl Value {
    /// The `princ`/`display` half of the classic Lisp print-function pair
    /// (Common Lisp `princ`, Scheme `display`): human-facing output, no
    /// quotes or escapes around strings — as opposed to `Display`/`to_string`
    /// above, which is the `prin1`/`write` half (re-readable by `read`,
    /// strings quoted and escaped). Neither replaces the other; `print`
    /// (this crate's primitive, backed by `Display`) needs to round-trip
    /// through `read`, so it keeps quoting — `princ` exists for output
    /// that's meant to be read by a person or reassembled as raw text
    /// (e.g. a tool that emits new .my source), never re-parsed as data.
    /// «princ»/«display»-polovyna klasychnoi Lisp-pary funktsii druku
    /// (Common Lisp `princ`, Scheme `display`): vyvid dlia liudyny, bez
    /// lapok i ekranuvannia navkolo riadkiv — na vidminu vid `Display`/
    /// `to_string` vyshche, shcho ye «prin1»/«write»-polovynoiu (chytaietsia nazad
    /// cherez `read`, riadky v lapkakh i ekranovani). Zhodna ne zaminiuie inshu;
    /// `print` (prymityv tsoho kreita, na osnovi `Display`) maie korektno
    /// chytatys nazad cherez `read`, tozh zberihaie lapky — `princ` isnuie dlia
    /// vyvodu, pryznachenoho liudyni chy povtornomu skladanniu yak syryi tekst
    /// (napr. instrument, shcho vydaie novyi `.my`-syrtsevyi kod), nikoly ne
    /// dlia povtornoho parsynhu yak danykh.
    pub fn to_princ_string(&self) -> String {
        render(self, false)
    }
}

/// Shared by `Display` (`quote_strings: true`, escaped — `prin1`/`write`
/// semantics) and `Value::to_princ_string` (`quote_strings: false`, raw —
/// `princ`/`display` semantics). One recursive walk, one flag, so the two
/// output modes can never silently diverge on anything but string handling.
/// Spilne dlia `Display` (`quote_strings: true`, z ekranuvanniam —
/// semantyka `prin1`/`write`) i `Value::to_princ_string` (`quote_strings:
/// false`, syryi vyvid — semantyka `princ`/`display`). Odyn rekursyvnyi
/// obkhid, odyn praporets, tozh dva rezhymy vyvodu ne mozhut movchky roziitys
/// u chomus, krim obrobky riadkiv.
fn render(value: &Value, quote_strings: bool) -> String {
    match value {
        Value::Nil => "()".to_string(),
        Value::Bool(true) => "t".to_string(),
        Value::Bool(false) => "()".to_string(),
        Value::Number(number, Exactness::Exact) => number.to_string(),
        // Rust's `{}` for f64 prints a whole number like 3.0 as "3", which
        // would silently erase the written-with-a-decimal-point intent this
        // whole redesign exists to preserve — force at least one decimal
        // digit so an inexact number always prints as inexact.
        // Rust's `{}` dlia f64 drukuie tsile chyslo na kshtalt 3.0 yak "3", shcho
        // movchky sterlo b same toi namir "napysano z krapkoiu", zarady
        // yakoho isnuie ves tsei redyzain — prymusovo drukuvaty shchonaimenshe
        // odnu desiatkovu tsyfru, shchob netochne chyslo zavzhdy drukuvalos yak
        // netochne.
        Value::Number(number, Exactness::Inexact) => {
            if number.fract() == 0.0 && number.is_finite() {
                format!("{number:.1}")
            } else {
                number.to_string()
            }
        }
        Value::Rational(number) => number.to_string(),
        Value::String(text) => {
            if quote_strings {
                let mut escaped = String::with_capacity(text.len() + 2);
                escaped.push('"');
                for ch in text.chars() {
                    match ch {
                        '"' => escaped.push_str("\\\""),
                        '\\' => escaped.push_str("\\\\"),
                        '\n' => escaped.push_str("\\n"),
                        '\t' => escaped.push_str("\\t"),
                        other => escaped.push(other),
                    }
                }
                escaped.push('"');
                escaped
            } else {
                text.to_string()
            }
        }
        Value::Symbol(symbol) => symbol.to_string(),
        Value::Pair(_, _) => render_pair(value, quote_strings),
        Value::Closure(_) => "<lambda>".to_string(),
        Value::Macro(_) => "<macro>".to_string(),
        Value::TcpConnection(_) => "<tcp-connection>".to_string(),
        Value::TcpListener(_) => "<tcp-listener>".to_string(),
    }
}

fn render_pair(value: &Value, quote_strings: bool) -> String {
    let mut out = String::from("(");
    let mut current = value;
    let mut first = true;
    loop {
        match current {
            Value::Pair(head, tail) => {
                if !first {
                    out.push(' ');
                }
                out.push_str(&render(head, quote_strings));
                current = tail;
                first = false;
            }
            Value::Nil => {
                out.push(')');
                return out;
            }
            tail => {
                out.push_str(" . ");
                out.push_str(&render(tail, quote_strings));
                out.push(')');
                return out;
            }
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
