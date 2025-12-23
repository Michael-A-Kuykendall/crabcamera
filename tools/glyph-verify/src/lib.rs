use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spellbook {
    pub spells: BTreeMap<String, Spell>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Spell {
    pub name: String,
    pub header_line: usize,
    pub intents: Vec<String>,
    pub entities: BTreeMap<String, Entity>,
    pub guarantees: BTreeMap<String, usize>,
    pub exclusions: BTreeMap<String, usize>,
    pub assumptions: BTreeMap<String, usize>,
    pub dependencies: BTreeMap<String, usize>,
    pub contracts: BTreeMap<String, usize>,
    pub open_questions: Vec<(String, usize)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Entity {
    pub name: String,
    pub header_line: usize,
    pub guarantees: BTreeMap<String, usize>,
    pub exclusions: BTreeMap<String, usize>,
    pub assumptions: BTreeMap<String, usize>,
    pub dependencies: BTreeMap<String, usize>,
    pub contracts: BTreeMap<String, usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompareOptions {
    pub deny_extra: bool,
    pub strict_intent: bool,
    pub deny_open_questions_in_invocation: bool,
}

impl Default for CompareOptions {
    fn default() -> Self {
        Self {
            deny_extra: false,
            strict_intent: false,
            deny_open_questions_in_invocation: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompareReport {
    pub ok: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl CompareReport {
    pub fn success() -> Self {
        Self {
            ok: true,
            errors: vec![],
            warnings: vec![],
        }
    }

    pub fn push_error(&mut self, msg: impl Into<String>) {
        self.ok = false;
        self.errors.push(msg.into());
    }

    pub fn push_warning(&mut self, msg: impl Into<String>) {
        self.warnings.push(msg.into());
    }
}

fn normalize_payload(payload: &str) -> String {
    payload
        .trim()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn insert_first(map: &mut BTreeMap<String, usize>, key: String, line_no: usize) {
    map.entry(key)
        .and_modify(|existing| {
            if line_no < *existing {
                *existing = line_no;
            }
        })
        .or_insert(line_no);
}

fn strip_inline_comment(line: &str) -> &str {
    // Remove trailing inline comments like: "> @Tokenizer  # depends on Tokenize"
    // Only treat `#` as a comment delimiter if it appears after at least one whitespace.
    let bytes = line.as_bytes();
    for i in 0..bytes.len() {
        if bytes[i] == b'#' {
            if i > 0 && bytes[i - 1].is_ascii_whitespace() {
                return &line[..i];
            }
        }
    }
    line
}

fn is_markdown_fence(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with("```") || trimmed.starts_with("~~~")
}

fn is_comment_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with('#') && !trimmed.starts_with("#Spell:")
}

fn parse_name_after_prefix(line: &str, prefix: &str) -> Option<String> {
    let trimmed = line.trim_start();
    if !trimmed.starts_with(prefix) {
        return None;
    }
    let rest = trimmed[prefix.len()..].trim();
    if rest.is_empty() {
        None
    } else {
        Some(rest.to_string())
    }
}

fn parse_entity_name(line: &str) -> Option<String> {
    let trimmed = line.trim_start();
    if !trimmed.starts_with('@') {
        return None;
    }

    let mut rest = trimmed[1..].trim();
    if rest.is_empty() {
        return None;
    }

    if let Some((name, _tail)) = rest.split_once(char::is_whitespace) {
        rest = name;
    }

    Some(rest.to_string())
}

pub fn parse_spellbook(input: &str) -> Result<Spellbook, String> {
    let mut spells: BTreeMap<String, Spell> = BTreeMap::new();

    let mut current_spell_name: Option<String> = None;
    let mut current_entity_name: Option<String> = None;
    let mut in_fence: bool = false;

    for (line_idx, line) in input.lines().enumerate() {
        let line_no = line_idx + 1;
        if is_markdown_fence(line) {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        if is_comment_line(line) {
            continue;
        }

        let line = strip_inline_comment(line);
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Some(name) = parse_name_after_prefix(trimmed, "#Spell:") {
            current_spell_name = Some(name.clone());
            current_entity_name = None;
            spells
                .entry(name.clone())
                .or_insert_with(|| Spell {
                    name,
                    header_line: line_no,
                    ..Default::default()
                });
            continue;
        }

        let Some(spell_name) = current_spell_name.clone() else {
            // Ignore preamble lines outside any spell.
            continue;
        };

        let spell = spells
            .get_mut(&spell_name)
            .expect("spell should exist");

        if let Some(entity_name) = parse_entity_name(trimmed) {
            current_entity_name = Some(entity_name.clone());
            spell
                .entities
                .entry(entity_name.clone())
                .or_insert_with(|| Entity {
                    name: entity_name,
                    header_line: line_no,
                    ..Default::default()
                });
            continue;
        }

        // Glyph line: first non-whitespace char is the glyph.
        let (glyph, payload) = trimmed.split_at(1);
        let payload = normalize_payload(payload);
        if payload.is_empty() {
            continue;
        }

        let is_known_glyph = matches!(glyph, "^" | "!" | "-" | "~" | ">" | ":" | "?");
        if !is_known_glyph {
            return Err(format!("unknown glyph '{glyph}' at line {line_no}: {trimmed}"));
        }

        if let Some(entity_name) = current_entity_name.clone() {
            let entity = spell
                .entities
                .get_mut(&entity_name)
                .expect("entity should exist");

            match glyph {
                "!" => {
                    insert_first(&mut entity.guarantees, payload, line_no);
                }
                "-" => {
                    insert_first(&mut entity.exclusions, payload, line_no);
                }
                "~" => {
                    insert_first(&mut entity.assumptions, payload, line_no);
                }
                ">" => {
                    insert_first(&mut entity.dependencies, payload, line_no);
                }
                ":" => {
                    insert_first(&mut entity.contracts, payload, line_no);
                }
                "^" => {
                    // Some dialects put intents under entities; treat as spell intent.
                    spell.intents.push(payload);
                }
                "?" => {
                    // Open questions are only the leading-glyph form.
                    spell.open_questions.push((payload, line_no));
                }
                _ => {}
            }
        } else {
            match glyph {
                "^" => spell.intents.push(payload),
                "!" => {
                    insert_first(&mut spell.guarantees, payload, line_no);
                }
                "-" => {
                    insert_first(&mut spell.exclusions, payload, line_no);
                }
                "~" => {
                    insert_first(&mut spell.assumptions, payload, line_no);
                }
                ">" => {
                    insert_first(&mut spell.dependencies, payload, line_no);
                }
                ":" => {
                    insert_first(&mut spell.contracts, payload, line_no);
                }
                "?" => spell.open_questions.push((payload, line_no)),
                _ => {}
            }
        }
    }

    if spells.is_empty() {
        return Err("no spells found".to_string());
    }

    Ok(Spellbook { spells })
}

fn map_missing(required: &BTreeMap<String, usize>, got: &BTreeMap<String, usize>) -> Vec<(String, usize)> {
    required
        .iter()
        .filter(|(req, _line)| !got.contains_key(*req))
        .map(|(k, v)| (k.clone(), *v))
        .collect()
}

fn map_extra(required: &BTreeMap<String, usize>, got: &BTreeMap<String, usize>) -> Vec<(String, usize)> {
    got.iter()
        .filter(|(g, _line)| !required.contains_key(*g))
        .map(|(k, v)| (k.clone(), *v))
        .collect()
}

fn fmt_line(line: usize) -> String {
    if line == 0 {
        "".to_string()
    } else {
        format!(" (line {line})")
    }
}

const FORBIDDEN_TERMS: &[&str] = &[
    "sorcery",
    "spell",
    "glyph",
    "sigil",
    "invocation",
    "incantation",
];

fn find_forbidden_term(text: &str) -> Option<&'static str> {
    let lower = text.to_ascii_lowercase();
    FORBIDDEN_TERMS
        .iter()
        .cloned()
        .find(|term| lower.contains(term))
}

fn check_forbidden_payloads(
    report: &mut CompareReport,
    spell_name: &str,
    entity_name: Option<&str>,
    glyph: char,
    payloads: &BTreeMap<String, usize>,
) {
    for (payload, line) in payloads {
        if let Some(term) = find_forbidden_term(payload) {
            let scope = match entity_name {
                Some(name) => format!(" @{name}"),
                None => String::new(),
            };
            report.push_error(format!(
                "forbidden sorcery term '{term}' in invocation for {spell_name}{scope}: {glyph} {payload}{}",
                fmt_line(*line)
            ));
        }
    }
}

pub fn compare_spellbooks(
    spellbook: &Spellbook,
    invocation: &Spellbook,
    options: &CompareOptions,
) -> CompareReport {
    let mut report = CompareReport::success();

    for (spell_name, spell) in &spellbook.spells {
        let Some(inv) = invocation.spells.get(spell_name) else {
            report.push_error(format!(
                "missing invocation spell: {spell_name}{}",
                fmt_line(spell.header_line)
            ));
            continue;
        };

        if spell.intents.is_empty() {
            report.push_error(format!("spell missing intent: {spell_name}"));
        }
        if inv.intents.is_empty() {
            report.push_error(format!("invocation missing intent: {spell_name}"));
        }

        if options.strict_intent && spell.intents != inv.intents {
            report.push_error(format!(
                "intent mismatch for {spell_name}: spell={:?} invocation={:?}",
                spell.intents, inv.intents
            ));
        }

        if options.deny_open_questions_in_invocation && !inv.open_questions.is_empty() {
            report.push_error(format!(
                "invocation contains open questions ('?') for {spell_name}: {:?}",
                inv.open_questions
            ));
        }

        check_forbidden_payloads(&mut report, spell_name, None, '!', &inv.guarantees);
        check_forbidden_payloads(&mut report, spell_name, None, '-', &inv.exclusions);
        check_forbidden_payloads(&mut report, spell_name, None, '~', &inv.assumptions);
        check_forbidden_payloads(&mut report, spell_name, None, '>', &inv.dependencies);
        check_forbidden_payloads(&mut report, spell_name, None, ':', &inv.contracts);

        // Spell-level sets.
        for (missing, line) in map_missing(&spell.guarantees, &inv.guarantees) {
            report.push_error(format!(
                "missing guarantee in invocation for {spell_name}: ! {missing}{}",
                fmt_line(line)
            ));
        }
        for (missing, line) in map_missing(&spell.exclusions, &inv.exclusions) {
            report.push_error(format!(
                "missing exclusion in invocation for {spell_name}: - {missing}{}",
                fmt_line(line)
            ));
        }
        for (missing, line) in map_missing(&spell.dependencies, &inv.dependencies) {
            report.push_error(format!(
                "missing dependency in invocation for {spell_name}: > {missing}{}",
                fmt_line(line)
            ));
        }
        for (missing, line) in map_missing(&spell.contracts, &inv.contracts) {
            report.push_error(format!(
                "missing contract in invocation for {spell_name}: : {missing}{}",
                fmt_line(line)
            ));
        }

        if options.deny_extra {
            for (extra, line) in map_extra(&spell.guarantees, &inv.guarantees) {
                report.push_error(format!(
                    "extra guarantee in invocation for {spell_name}: ! {extra}{}",
                    fmt_line(line)
                ));
            }
            for (extra, line) in map_extra(&spell.exclusions, &inv.exclusions) {
                report.push_error(format!(
                    "extra exclusion in invocation for {spell_name}: - {extra}{}",
                    fmt_line(line)
                ));
            }
            for (extra, line) in map_extra(&spell.dependencies, &inv.dependencies) {
                report.push_error(format!(
                    "extra dependency in invocation for {spell_name}: > {extra}{}",
                    fmt_line(line)
                ));
            }
            for (extra, line) in map_extra(&spell.contracts, &inv.contracts) {
                report.push_error(format!(
                    "extra contract in invocation for {spell_name}: : {extra}{}",
                    fmt_line(line)
                ));
            }
        }

        // Entities
        for (entity_name, entity) in &spell.entities {
            let Some(inv_entity) = inv.entities.get(entity_name) else {
                report.push_error(format!(
                    "missing entity in invocation for {spell_name}: @{entity_name}{}",
                    fmt_line(entity.header_line)
                ));
                continue;
            };

            check_forbidden_payloads(
                &mut report,
                spell_name,
                Some(entity_name),
                '!',
                &inv_entity.guarantees,
            );
            check_forbidden_payloads(
                &mut report,
                spell_name,
                Some(entity_name),
                '-',
                &inv_entity.exclusions,
            );
            check_forbidden_payloads(
                &mut report,
                spell_name,
                Some(entity_name),
                '~',
                &inv_entity.assumptions,
            );
            check_forbidden_payloads(
                &mut report,
                spell_name,
                Some(entity_name),
                '>',
                &inv_entity.dependencies,
            );
            check_forbidden_payloads(
                &mut report,
                spell_name,
                Some(entity_name),
                ':',
                &inv_entity.contracts,
            );

            for (missing, line) in map_missing(&entity.guarantees, &inv_entity.guarantees) {
                report.push_error(format!(
                    "missing entity guarantee in invocation for {spell_name} @{entity_name}: ! {missing}{}",
                    fmt_line(line)
                ));
            }
            for (missing, line) in map_missing(&entity.exclusions, &inv_entity.exclusions) {
                report.push_error(format!(
                    "missing entity exclusion in invocation for {spell_name} @{entity_name}: - {missing}{}",
                    fmt_line(line)
                ));
            }
            for (missing, line) in map_missing(&entity.dependencies, &inv_entity.dependencies) {
                report.push_error(format!(
                    "missing entity dependency in invocation for {spell_name} @{entity_name}: > {missing}{}",
                    fmt_line(line)
                ));
            }
            for (missing, line) in map_missing(&entity.contracts, &inv_entity.contracts) {
                report.push_error(format!(
                    "missing entity contract in invocation for {spell_name} @{entity_name}: : {missing}{}",
                    fmt_line(line)
                ));
            }

            if options.deny_extra {
                for (extra, line) in map_extra(&entity.guarantees, &inv_entity.guarantees) {
                    report.push_error(format!(
                        "extra entity guarantee in invocation for {spell_name} @{entity_name}: ! {extra}{}",
                        fmt_line(line)
                    ));
                }
                for (extra, line) in map_extra(&entity.exclusions, &inv_entity.exclusions) {
                    report.push_error(format!(
                        "extra entity exclusion in invocation for {spell_name} @{entity_name}: - {extra}{}",
                        fmt_line(line)
                    ));
                }
                for (extra, line) in map_extra(&entity.dependencies, &inv_entity.dependencies) {
                    report.push_error(format!(
                        "extra entity dependency in invocation for {spell_name} @{entity_name}: > {extra}{}",
                        fmt_line(line)
                    ));
                }
                for (extra, line) in map_extra(&entity.contracts, &inv_entity.contracts) {
                    report.push_error(format!(
                        "extra entity contract in invocation for {spell_name} @{entity_name}: : {extra}{}",
                        fmt_line(line)
                    ));
                }
            }
        }

        if options.deny_extra {
            for inv_entity_name in inv.entities.keys() {
                if !spell.entities.contains_key(inv_entity_name) {
                    report.push_error(format!(
                        "extra entity in invocation for {spell_name}: @{inv_entity_name}"
                    ));
                }
            }
        }
    }

    if options.deny_extra {
        for inv_spell_name in invocation.spells.keys() {
            if !spellbook.spells.contains_key(inv_spell_name) {
                report.push_error(format!("extra invocation spell: {inv_spell_name}"));
            }
        }
    }

    report
}

#[cfg(test)]
mod tests {
    use super::*;

    const GLYPH_VERIFY_SPELL: &str = include_str!("../../../examples/syntax/glyph_verify.spell");

    #[test]
    fn parses_minimal_spellbook() {
        let input = r#"
#Spell: Tokenize
^ Intent: stable tokens

@Tokenizer
  : utf8 -> tokens
  ! deterministic
  - network
"#;

        let book = parse_spellbook(input).unwrap();
        assert!(book.spells.contains_key("Tokenize"));
        let s = book.spells.get("Tokenize").unwrap();
        assert_eq!(s.intents.len(), 1);
        assert!(s.entities.contains_key("Tokenizer"));
    }

    #[test]
    fn compare_fails_on_missing_exclusion() {
        let spell = r#"
#Spell: X
^ Intent: x

@E
  ! a
  - b
"#;

        let inv = r#"
#Spell: X
^ Intent: x

@E
  ! a
"#;

        let s = parse_spellbook(spell).unwrap();
        let i = parse_spellbook(inv).unwrap();
        let rep = compare_spellbooks(&s, &i, &CompareOptions::default());
        assert!(!rep.ok);
        assert!(rep
            .errors
            .iter()
            .any(|e| e.contains("missing entity exclusion") || e.contains("- b")));
    }

    #[test]
    fn strips_inline_comment_after_whitespace_hash() {
        let spell = r#"
#Spell: S
^ Intent: x

@E
  - network  # forbidden
"#;
        let s = parse_spellbook(spell).unwrap();
        let sp = s.spells.get("S").unwrap();
        let ent = sp.entities.get("E").unwrap();
        assert!(ent.exclusions.contains_key("network"));
    }

    #[test]
    fn parses_and_self_compares_glyph_verify_spell() {
        let s = parse_spellbook(GLYPH_VERIFY_SPELL).unwrap();
        assert!(s.spells.contains_key("ConstraintGate"));

        let rep = compare_spellbooks(&s, &s, &CompareOptions::default());
        assert!(rep.ok, "expected ok, got errors: {:?}", rep.errors);
    }
}
