use my_lisp::syntax::Expr;
use my_lisp::{parse, ExprKind};

// Historical filename; ADR-005 defines this table as the current trilingual
// registry. The runtime interpretation is EN/UK/SA peer surfaces over one
// canonical semantic identity, not English as semantic authority.
const REGISTRY: &str = include_str!("../../../../lib/surface/uk-sa-coverage.wsm");
const UK_API_DOCS: &str = include_str!("../../../../lib/surface/uk-docs.wsm");

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
struct SurfaceEntry {
    category: String,
    canonical: String,
    en: String,
    uk: Option<String>,
    sa: Option<String>,
    en_status: Status,
    uk_status: Status,
    sa_status: Status,
    notes: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SurfaceDoc {
    category: String,
    canonical: String,
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

fn registry_entries() -> Result<Vec<SurfaceEntry>, String> {
    let mut entries = Vec::new();
    for (index, line) in REGISTRY.lines().enumerate() {
        let fields = line.split_whitespace().collect::<Vec<_>>();
        if fields.first() != Some(&"(entry") {
            continue;
        }
        if fields.len() < 7 {
            return Err(format!("registry line {}: malformed entry", index + 1));
        }

        let category = fields[1].trim_start_matches('(').to_string();
        let canonical = fields[2].to_string();
        let uk_status = Status::parse(fields[5])?;
        let sa_status = Status::parse(fields[6])?;
        // ADR-005: the legacy canonical/EN column plays two explicit roles.
        // EN is a peer surface whose current spelling happens to coincide with
        // canonical identity. Rows outside the selected public denominator are
        // compatibility-only for EN as well.
        let en_status = if uk_status == Status::CompatibilityOnly {
            Status::CompatibilityOnly
        } else {
            Status::Stable
        };
        let uk = (fields[3] != "—").then(|| fields[3].to_string());
        let sa = (fields[4] != "—").then(|| fields[4].to_string());

        if matches!(uk_status, Status::Stable | Status::Candidate) && uk.is_none() {
            return Err(format!(
                "registry line {}: {} UK row has no name",
                index + 1,
                uk_status.machine_name()
            ));
        }
        if uk_status == Status::Missing && uk.is_some() {
            return Err(format!("registry line {}: missing UK row has a name", index + 1));
        }
        if matches!(sa_status, Status::Stable | Status::Candidate) && sa.is_none() {
            return Err(format!(
                "registry line {}: {} SA row has no name",
                index + 1,
                sa_status.machine_name()
            ));
        }
        if sa_status == Status::Missing && sa.is_some() {
            return Err(format!("registry line {}: missing SA row has a name", index + 1));
        }

        let notes = line
            .split_once('"')
            .and_then(|(_, tail)| tail.rsplit_once('"').map(|(text, _)| text))
            .unwrap_or("")
            .to_string();

        entries.push(SurfaceEntry {
            category,
            en: canonical.clone(),
            canonical,
            uk,
            sa,
            en_status,
            uk_status,
            sa_status,
            notes,
        });
    }

    if entries.is_empty() {
        return Err("trilingual registry contains no entries".to_string());
    }
    Ok(entries)
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
            let fields = expr_list(entry).ok_or_else(|| "uk-docs.wsm: doc має бути списком".to_string())?;
            if fields.len() != 7 || fields.first().and_then(expr_symbol) != Some("doc") {
                return Err("uk-docs.wsm: некоректний doc-запис".to_string());
            }
            Ok(SurfaceDoc {
                category: expr_symbol(&fields[1])
                    .ok_or_else(|| "uk-docs.wsm: category має бути символом".to_string())?
                    .to_string(),
                canonical: expr_symbol(&fields[2])
                    .ok_or_else(|| "uk-docs.wsm: canonical має бути символом".to_string())?
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

fn status_for(entry: &SurfaceEntry, surface: &str) -> Status {
    match surface {
        "en" => entry.en_status,
        "uk" | "ук" => entry.uk_status,
        "sa" => entry.sa_status,
        _ => Status::CompatibilityOnly,
    }
}

fn name_for<'a>(entry: &'a SurfaceEntry, surface: &str) -> Option<&'a str> {
    match surface {
        "en" => Some(&entry.en),
        "uk" | "ук" => entry.uk.as_deref(),
        "sa" => entry.sa.as_deref(),
        _ => None,
    }
}

fn counts_for(entries: &[SurfaceEntry], surface: &str) -> Counts {
    let mut counts = Counts::default();
    for entry in entries {
        counts.add(status_for(entry, surface));
    }
    counts
}

fn public_denominator(entries: &[SurfaceEntry]) -> usize {
    entries
        .iter()
        .filter(|entry| entry.en_status != Status::CompatibilityOnly)
        .count()
}

fn find_entry<'a>(entries: &'a [SurfaceEntry], requested: &str) -> Option<&'a SurfaceEntry> {
    entries.iter().find(|entry| {
        entry.canonical == requested
            || entry.en == requested
            || entry.uk.as_deref() == Some(requested)
            || entry.sa.as_deref() == Some(requested)
    })
}

fn rendered_name(name: Option<&str>, status: Status) -> String {
    match name {
        Some(name) => format!("{name} [{}]", status.machine_name()),
        None => format!("— [{}]", status.machine_name()),
    }
}

pub(crate) fn render_status() -> Result<String, String> {
    let entries = registry_entries()?;
    let denominator = public_denominator(&entries);
    let en = counts_for(&entries, "en");
    let uk = counts_for(&entries, "uk");
    let sa = counts_for(&entries, "sa");
    let trilingual_stable = entries
        .iter()
        .filter(|entry| entry.en_status != Status::CompatibilityOnly)
        .filter(|entry| {
            entry.en_status == Status::Stable
                && entry.uk_status == Status::Stable
                && entry.sa_status == Status::Stable
        })
        .count();

    Ok(format!(
        "Триєдина поверхня · public identities: {denominator}\n\
         EN  stable {:>3} · candidate {:>3} · missing {:>3} · compatibility {:>3}\n\
         UK  stable {:>3} · candidate {:>3} · missing {:>3} · compatibility {:>3}\n\
         SA  stable {:>3} · candidate {:>3} · missing {:>3} · compatibility {:>3}\n\
         trilingual stable: {trilingual_stable}/{denominator}\n\
         release parity: {}",
        en.stable,
        en.candidate,
        en.missing,
        en.compatibility,
        uk.stable,
        uk.candidate,
        uk.missing,
        uk.compatibility,
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
    if surface == "core" {
        return Ok(
            "core — канонічний машинний шар, не четверта людська мова. Використайте :мова en, :мова ук або :мова sa."
                .to_string(),
        );
    }
    let entries = registry_entries()?;
    let counts = counts_for(&entries, surface);
    let denominator = public_denominator(&entries);
    let mut output = format!(
        "surface {surface}: stable {} · candidate {} · missing {} · public {denominator}\n",
        counts.stable, counts.candidate, counts.missing
    );

    let mut current_category: Option<&str> = None;
    for entry in entries
        .iter()
        .filter(|entry| status_for(entry, surface) != Status::CompatibilityOnly)
    {
        if current_category != Some(entry.category.as_str()) {
            current_category = Some(&entry.category);
            output.push('\n');
            output.push('[');
            output.push_str(&entry.category);
            output.push_str("]\n  ");
        } else {
            output.push_str(" · ");
        }
        match (name_for(entry, surface), status_for(entry, surface)) {
            (Some(name), Status::Candidate) => {
                output.push('~');
                output.push_str(name);
            }
            (Some(name), Status::Stable) => output.push_str(name),
            (None, Status::Missing) => {
                output.push_str("—{");
                output.push_str(&entry.canonical);
                output.push('}');
            }
            (Some(name), _) => output.push_str(name),
            (None, _) => output.push('—'),
        }
    }
    if counts.candidate > 0 {
        output.push_str("\n\n~name = candidate, not yet ratified");
    }
    if counts.missing > 0 {
        output.push_str("\n—{identity} = public semantic identity with no surface name yet");
    }
    Ok(output)
}

pub(crate) fn render_name(surface: &str, requested: &str) -> Result<String, String> {
    let entries = registry_entries()?;
    let Some(entry) = find_entry(&entries, requested) else {
        return Ok(format!(
            "«{requested}» не знайдено у триєдиному surface registry; сире середовище перевіряється через (env)/(середовище)."
        ));
    };

    let mut output = format!(
        "identity: {}\n  EN: {}\n  UK: {}\n  SA: {}\n  current: {}",
        entry.canonical,
        rendered_name(Some(&entry.en), entry.en_status),
        rendered_name(entry.uk.as_deref(), entry.uk_status),
        rendered_name(entry.sa.as_deref(), entry.sa_status),
        surface
    );
    if !entry.notes.is_empty() {
        output.push_str("\n  notes: ");
        output.push_str(&entry.notes);
    }

    // Ukrainian already has a complete machine-readable human docs index.
    // Keep that richer help without pretending EN/SA documentation parity is
    // closed. ADR-005 makes docs/presentation separate completion evidence.
    if surface == "uk" || surface == "ук" {
        if let Some(doc) = ukrainian_docs()?
            .into_iter()
            .find(|doc| doc.canonical == entry.canonical)
        {
            output.push_str("\n  категорія: ");
            output.push_str(&doc.category);
            output.push_str("\n  тип: ");
            output.push_str(&doc.kind);
            output.push_str("\n  виклик: ");
            output.push_str(&doc.call);
            output.push_str("\n  ");
            output.push_str(&doc.description);
            debug_assert_eq!(doc.name, entry.uk.as_deref().unwrap_or(""));
        }
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_registry_exposes_three_peer_surface_counts() {
        let entries = registry_entries().expect("registry");
        assert_eq!(entries.len(), 161);
        assert_eq!(public_denominator(&entries), 140);
        assert_eq!(counts_for(&entries, "en"), Counts { stable: 140, candidate: 0, missing: 0, compatibility: 21 });
        assert_eq!(counts_for(&entries, "uk"), Counts { stable: 140, candidate: 0, missing: 0, compatibility: 21 });
        assert_eq!(counts_for(&entries, "sa"), Counts { stable: 36, candidate: 88, missing: 16, compatibility: 21 });
    }

    #[test]
    fn any_language_name_resolves_to_one_semantic_identity() {
        let entries = registry_entries().expect("registry");
        for requested in ["map", "відобразити", "āvartana"] {
            assert_eq!(find_entry(&entries, requested).map(|entry| entry.canonical.as_str()), Some("map"));
        }
    }

    #[test]
    fn catalogs_exist_for_all_three_human_surfaces() {
        let en = render_names("en").expect("EN catalog");
        let uk = render_names("uk").expect("UK catalog");
        let sa = render_names("sa").expect("SA catalog");
        assert!(en.contains("surface en: stable 140"));
        assert!(uk.contains("surface uk: stable 140"));
        assert!(sa.contains("surface sa: stable 36 · candidate 88 · missing 16"));
        assert!(sa.contains("~āvartana"));
        assert!(sa.contains("—{lambda}"));
    }

    #[test]
    fn status_refuses_to_call_current_state_complete() {
        let status = render_status().expect("status");
        assert!(status.contains("trilingual stable: 36/140"));
        assert!(status.contains("release parity: OPEN"));
    }

    #[test]
    fn ukrainian_name_help_keeps_rich_docs_and_cross_language_identity() {
        let help = render_name("uk", "відобразити").expect("help");
        assert!(help.contains("identity: map"));
        assert!(help.contains("EN: map [stable]"));
        assert!(help.contains("UK: відобразити [stable]"));
        assert!(help.contains("SA: āvartana [candidate]"));
        assert!(help.contains("виклик: (відобразити функція список)"));
    }
}
