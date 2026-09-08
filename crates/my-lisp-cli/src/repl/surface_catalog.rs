use my_lisp::syntax::Expr;
use my_lisp::{parse, ExprKind};

const REGISTRY: &str = include_str!("../../../../lib/surface/semantic-registry.wsm");
const UK_API_DOCS: &str = include_str!("../../../../lib/surface/uk-docs.wsm");
const HUMAN_SURFACES: [&str; 3] = ["uk", "en", "sa"];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Status {
    Stable,
    Candidate,
    Missing,
    CompatibilityOnly,
}

impl Status {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "stable" => Ok(Self::Stable),
            "candidate" => Ok(Self::Candidate),
            "missing" => Ok(Self::Missing),
            "compatibility-only" => Ok(Self::CompatibilityOnly),
            other => Err(format!("невідомий статус поверхні: {other}")),
        }
    }

    fn machine_name(self) -> &'static str {
        match self {
            Self::Stable => "stable",
            Self::Candidate => "candidate",
            Self::Missing => "missing",
            Self::CompatibilityOnly => "compatibility-only",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SurfaceName {
    surface: String,
    name: Option<String>,
    status: Status,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SurfaceEntry {
    identity: String,
    names: Vec<SurfaceName>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SurfaceDoc {
    category: String,
    name: String,
    kind: String,
    call: String,
    description: String,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct Counts {
    stable: usize,
    candidate: usize,
    missing: usize,
    compatibility: usize,
}

impl Counts {
    fn add(&mut self, status: Status) {
        match status {
            Status::Stable => self.stable += 1,
            Status::Candidate => self.candidate += 1,
            Status::Missing => self.missing += 1,
            Status::CompatibilityOnly => self.compatibility += 1,
        }
    }
}

fn normalize_surface(surface: &str) -> &str {
    match surface {
        "ук" => "uk",
        other => other,
    }
}

fn registry_entries() -> Result<Vec<SurfaceEntry>, String> {
    let mut entries = Vec::new();

    for (index, line) in REGISTRY.lines().enumerate() {
        let fields = line.split_whitespace().collect::<Vec<_>>();
        let Some(first) = fields.first() else {
            continue;
        };
        if !first.starts_with('(') {
            continue;
        }
        let identity = first.trim_start_matches('(');
        if identity == "sr/1" || !identity.chars().all(|character| character.is_ascii_digit()) {
            continue;
        }
        if identity.len() < 4 {
            return Err(format!("registry line {}: semantic ID is too short", index + 1));
        }
        if (fields.len() - 1) % 3 != 0 {
            return Err(format!("registry line {}: malformed surface triples", index + 1));
        }

        let mut names = Vec::new();
        for triple in fields[1..].as_chunks::<3>().0 {
            let surface = triple[0].trim_start_matches('(').to_string();
            let raw_name = triple[1];
            let status = Status::parse(triple[2].trim_end_matches(')'))?;
            if names.iter().any(|name: &SurfaceName| name.surface == surface) {
                return Err(format!(
                    "registry line {}: duplicate surface {surface}",
                    index + 1
                ));
            }
            names.push(SurfaceName {
                surface,
                name: (raw_name != "—").then(|| raw_name.to_string()),
                status,
            });
        }

        for required in HUMAN_SURFACES {
            if !names.iter().any(|name| name.surface == required) {
                return Err(format!(
                    "registry line {}: {identity} has no explicit {required} row",
                    index + 1
                ));
            }
        }
        entries.push(SurfaceEntry {
            identity: identity.to_string(),
            names,
        });
    }

    if entries.is_empty() {
        return Err("numeric semantic registry contains no entries".to_string());
    }
    Ok(entries)
}

fn surface_name<'a>(entry: &'a SurfaceEntry, surface: &str) -> Option<&'a SurfaceName> {
    let surface = normalize_surface(surface);
    entry.names.iter().find(|name| name.surface == surface)
}

fn is_public(entry: &SurfaceEntry) -> bool {
    !HUMAN_SURFACES.iter().all(|surface| {
        surface_name(entry, surface)
            .is_some_and(|name| name.status == Status::CompatibilityOnly)
    })
}

fn counts_for(entries: &[SurfaceEntry], surface: &str) -> Counts {
    let mut counts = Counts::default();
    for entry in entries {
        if let Some(name) = surface_name(entry, surface) {
            counts.add(name.status);
        }
    }
    counts
}

fn public_denominator(entries: &[SurfaceEntry]) -> usize {
    entries.iter().filter(|entry| is_public(entry)).count()
}

fn find_entry<'a>(entries: &'a [SurfaceEntry], requested: &str) -> Option<&'a SurfaceEntry> {
    entries.iter().find(|entry| {
        entry.identity == requested
            || entry
                .names
                .iter()
                .any(|surface| surface.name.as_deref() == Some(requested))
    })
}

fn rendered_name(surface: Option<&SurfaceName>) -> String {
    match surface {
        Some(surface) => match surface.name.as_deref() {
            Some(name) => format!("{name} [{}]", surface.status.machine_name()),
            None => format!("— [{}]", surface.status.machine_name()),
        },
        None => "— [not-defined]".to_string(),
    }
}

fn expr_list(expr: &Expr) -> Option<&[Expr]> {
    match &expr.kind {
        ExprKind::List(items) => Some(items.as_ref()),
        _ => None,
    }
}

fn expr_symbol(expr: &Expr) -> Option<&str> {
    match &expr.kind {
        ExprKind::Symbol(symbol) => Some(symbol.as_ref()),
        _ => None,
    }
}

fn expr_string(expr: &Expr) -> Option<&str> {
    match &expr.kind {
        ExprKind::String(text) => Some(text.as_ref()),
        _ => None,
    }
}

fn ukrainian_docs() -> Result<Vec<SurfaceDoc>, String> {
    let program = parse(UK_API_DOCS)
        .map_err(|error| format!("не вдалося прочитати uk-docs.wsm: {}", error.render(UK_API_DOCS)))?;
    let root = program
        .first()
        .and_then(expr_list)
        .ok_or_else(|| "uk-docs.wsm: очікувався кореневий список".to_string())?;
    if root.first().and_then(expr_symbol) != Some("uk-api-docs") {
        return Err("uk-docs.wsm: невідомий кореневий тег".to_string());
    }
    let docs_form = root
        .iter()
        .filter_map(expr_list)
        .find(|items| items.first().and_then(expr_symbol) == Some("docs"))
        .ok_or_else(|| "uk-docs.wsm: відсутня секція docs".to_string())?;

    docs_form
        .iter()
        .skip(1)
        .map(|entry| {
            let fields =
                expr_list(entry).ok_or_else(|| "uk-docs.wsm: doc має бути списком".to_string())?;
            if fields.len() != 7 || fields.first().and_then(expr_symbol) != Some("doc") {
                return Err("uk-docs.wsm: некоректний doc-запис".to_string());
            }
            Ok(SurfaceDoc {
                category: expr_symbol(&fields[1])
                    .ok_or_else(|| "uk-docs.wsm: category має бути символом".to_string())?
                    .to_string(),
                name: expr_symbol(&fields[3])
                    .ok_or_else(|| "uk-docs.wsm: UK name має бути символом".to_string())?
                    .to_string(),
                kind: expr_symbol(&fields[4])
                    .ok_or_else(|| "uk-docs.wsm: kind має бути символом".to_string())?
                    .to_string(),
                call: expr_string(&fields[5])
                    .ok_or_else(|| "uk-docs.wsm: call має бути рядком".to_string())?
                    .to_string(),
                description: expr_string(&fields[6])
                    .ok_or_else(|| "uk-docs.wsm: description має бути рядком".to_string())?
                    .to_string(),
            })
        })
        .collect()
}

pub(crate) fn render_status() -> Result<String, String> {
    let entries = registry_entries()?;
    let denominator = public_denominator(&entries);
    let uk = counts_for(&entries, "uk");
    let en = counts_for(&entries, "en");
    let sa = counts_for(&entries, "sa");
    let symbolic = entries
        .iter()
        .filter(|entry| surface_name(entry, "sym").is_some())
        .count();
    let trilingual_stable = entries
        .iter()
        .filter(|entry| is_public(entry))
        .filter(|entry| {
            HUMAN_SURFACES.iter().all(|surface| {
                surface_name(entry, surface)
                    .is_some_and(|name| name.status == Status::Stable)
            })
        })
        .count();

    Ok(format!(
        "Рівноправні людські поверхні · numeric identities: {denominator}\n\
         UK  stable {:>3} · candidate {:>3} · missing {:>3} · compatibility {:>3}\n\
         EN  stable {:>3} · candidate {:>3} · missing {:>3} · compatibility {:>3}\n\
         SA  stable {:>3} · candidate {:>3} · missing {:>3} · compatibility {:>3}\n\
         shared sym identities: {symbolic}\n\
         trilingual stable: {trilingual_stable}/{denominator}\n\
         release parity: {}",
        uk.stable,
        uk.candidate,
        uk.missing,
        uk.compatibility,
        en.stable,
        en.candidate,
        en.missing,
        en.compatibility,
        sa.stable,
        sa.candidate,
        sa.missing,
        sa.compatibility,
        if trilingual_stable == denominator {
            "CONFIRMED"
        } else {
            "OPEN"
        }
    ))
}

pub(crate) fn render_names(surface: &str) -> Result<String, String> {
    let entries = registry_entries()?;
    if surface == "core" {
        let public = entries.iter().filter(|entry| is_public(entry));
        let mut output = format!(
            "core: numeric semantic identities · public {}\n",
            public_denominator(&entries)
        );
        for (index, entry) in public.enumerate() {
            if index > 0 {
                output.push_str(if index % 12 == 0 { "\n" } else { " · " });
            }
            output.push_str(&entry.identity);
        }
        output.push_str("\n\ncore показує machine handles; людські назви дивіться через :ім'я <ID>.");
        return Ok(output);
    }

    let surface = normalize_surface(surface);
    let counts = counts_for(&entries, surface);
    let denominator = public_denominator(&entries);
    let mut output = format!(
        "surface {surface}: stable {} · candidate {} · missing {} · public {denominator}\n  ",
        counts.stable, counts.candidate, counts.missing
    );
    let mut first = true;
    for entry in entries.iter().filter(|entry| is_public(entry)) {
        let Some(name) = surface_name(entry, surface) else {
            continue;
        };
        if name.status == Status::CompatibilityOnly {
            continue;
        }
        if !first {
            output.push_str(" · ");
        }
        first = false;
        match (&name.name, name.status) {
            (Some(name), Status::Candidate) => {
                output.push('~');
                output.push_str(name);
            }
            (Some(name), Status::Stable) => output.push_str(name),
            (None, Status::Missing) => {
                output.push_str("—{");
                output.push_str(&entry.identity);
                output.push('}');
                if let Some(symbolic) = surface_name(entry, "sym").and_then(|item| item.name.as_deref()) {
                    output.push_str("[sym ");
                    output.push_str(symbolic);
                    output.push(']');
                }
            }
            (Some(name), _) => output.push_str(name),
            (None, _) => output.push('—'),
        }
    }
    if counts.candidate > 0 {
        output.push_str("\n\n~name = candidate, ще не ратифіковане");
    }
    if counts.missing > 0 {
        output.push_str("\n—{ID} = numeric semantic identity без людського імені цієї surface");
    }
    Ok(output)
}

pub(crate) fn render_name(surface: &str, requested: &str) -> Result<String, String> {
    let entries = registry_entries()?;
    let Some(entry) = find_entry(&entries, requested) else {
        return Ok(format!(
            "«{requested}» не знайдено у numeric semantic registry; сире середовище перевіряється через (env)/(середовище)."
        ));
    };

    let mut output = format!(
        "identity: {}\n  UK: {}\n  EN: {}\n  SA: {}",
        entry.identity,
        rendered_name(surface_name(entry, "uk")),
        rendered_name(surface_name(entry, "en")),
        rendered_name(surface_name(entry, "sa")),
    );
    if surface_name(entry, "sym").is_some() {
        output.push_str("\n  SYM: ");
        output.push_str(&rendered_name(surface_name(entry, "sym")));
    }
    output.push_str("\n  current: ");
    output.push_str(surface);

    if normalize_surface(surface) == "uk" {
        if let Some(uk_name) = surface_name(entry, "uk").and_then(|name| name.name.as_deref()) {
            if let Some(doc) = ukrainian_docs()?.into_iter().find(|doc| doc.name == uk_name) {
                output.push_str("\n  категорія: ");
                output.push_str(&doc.category);
                output.push_str("\n  тип: ");
                output.push_str(&doc.kind);
                output.push_str("\n  виклик: ");
                output.push_str(&doc.call);
                output.push_str("\n  ");
                output.push_str(&doc.description);
            }
        }
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numeric_identity_is_the_only_machine_key() {
        let entries = registry_entries().expect("numeric registry");
        for requested in ["0101", "map", "відобразити", "āvartana"] {
            assert_eq!(
                find_entry(&entries, requested).map(|entry| entry.identity.as_str()),
                Some("0101")
            );
        }
    }

    #[test]
    fn plus_is_shared_symbol_not_english() {
        let entries = registry_entries().expect("numeric registry");
        let entry = find_entry(&entries, "+").expect("+ identity");
        assert_eq!(entry.identity, "0104");
        assert_eq!(surface_name(entry, "en").and_then(|item| item.name.as_deref()), None);
        assert_eq!(surface_name(entry, "uk").and_then(|item| item.name.as_deref()), Some("додати"));
        assert_eq!(surface_name(entry, "sa").and_then(|item| item.name.as_deref()), Some("yoga"));
        assert_eq!(surface_name(entry, "sym").and_then(|item| item.name.as_deref()), Some("+"));
    }

    #[test]
    fn repl_never_reports_a_human_spelling_as_identity() {
        let output = render_name("uk", "map").expect("render map");
        assert!(output.starts_with("identity: 0101\n"));
        assert!(!output.contains("identity: map"));

        let plus = render_name("uk", "+").expect("render +");
        assert!(plus.starts_with("identity: 0104\n"));
        assert!(plus.contains("EN: — [missing]"));
        assert!(plus.contains("SYM: + [stable]"));
    }

    #[test]
    fn core_catalog_is_numeric() {
        let output = render_names("core").expect("core catalog");
        assert!(output.contains("0101"));
        assert!(output.contains("0104"));
        assert!(!output.contains(" · map"));
    }

    #[test]
    fn ratified_sanskrit_cond_spelling_is_visible() {
        let output = render_name("sa", "anukrama").expect("cond");
        assert!(output.starts_with("identity: 0007\n"));
    }
}
