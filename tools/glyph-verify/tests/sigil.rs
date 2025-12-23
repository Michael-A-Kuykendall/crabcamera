use glyph_verify::{compare_spellbooks, parse_spellbook, CompareOptions};

#[test]
fn sigil_rejects_unknown_glyphs_with_line_number() {
    let text = include_str!("fixtures/unknown_glyph.spell");
    let err = parse_spellbook(text).unwrap_err();
    assert!(err.contains("unknown glyph"));
    assert!(err.contains("line"));
    assert!(err.contains("$") || err.contains("'$") );
}

#[test]
fn sigil_parses_multiple_spells_and_ignores_fences_and_comments() {
    let text = include_str!("fixtures/multi_spell_fenced.spell");
    let book = parse_spellbook(text).unwrap();

    assert!(book.spells.contains_key("A"));
    assert!(book.spells.contains_key("B"));
    assert!(!book.spells.contains_key("IgnoredInsideFence"));

    let a = book.spells.get("A").unwrap();
    assert_eq!(a.intents.len(), 1);
    assert!(a.entities.contains_key("E"));

    let b = book.spells.get("B").unwrap();
    assert!(b.entities.contains_key("E"));
}

#[test]
fn sigil_does_not_treat_question_mark_in_payload_as_open_question() {
    let text = "#Spell: Q\n^ Intent: q\n\n@E\n  : foo? -> bar\n  - network\n";
    let book = parse_spellbook(text).unwrap();
    let q = book.spells.get("Q").unwrap();
    assert!(q.open_questions.is_empty());
    let e = q.entities.get("E").unwrap();
    assert!(e.contracts.contains_key("foo? -> bar"));
}

#[test]
fn sigil_blocks_open_questions_in_invocation_by_default() {
    let spell = "#Spell: S\n^ Intent: s\n\n@E\n  - network\n";
    let inv = "#Spell: S\n^ Intent: s\n\n? still_uncertain\n\n@E\n  - network\n";

    let s = parse_spellbook(spell).unwrap();
    let i = parse_spellbook(inv).unwrap();
    let rep = compare_spellbooks(&s, &i, &CompareOptions::default());
    assert!(!rep.ok);
    assert!(rep.errors.iter().any(|e| e.contains("open questions")));
}

#[test]
fn sigil_deny_extra_catches_extra_constraints_and_entities() {
    let spell = "#Spell: S\n^ Intent: s\n\n@E\n  - network\n";
    let inv = "#Spell: S\n^ Intent: s\n\n@E\n  - network\n  ! deterministic\n\n@Extra\n  - filesystem_writes\n";

    let s = parse_spellbook(spell).unwrap();
    let i = parse_spellbook(inv).unwrap();

    let mut opts = CompareOptions::default();
    opts.deny_extra = true;

    let rep = compare_spellbooks(&s, &i, &opts);
    assert!(!rep.ok);
    assert!(rep.errors.iter().any(|e| e.contains("extra")));
}

#[test]
fn sigil_strict_intent_catches_intent_mismatch() {
    let spell = "#Spell: S\n^ Intent: canonical\n\n@E\n  - network\n";
    let inv = "#Spell: S\n^ Intent: different\n\n@E\n  - network\n";

    let s = parse_spellbook(spell).unwrap();
    let i = parse_spellbook(inv).unwrap();

    let mut opts = CompareOptions::default();
    opts.strict_intent = true;

    let rep = compare_spellbooks(&s, &i, &opts);
    assert!(!rep.ok);
    assert!(rep.errors.iter().any(|e| e.contains("intent mismatch")));
}

#[test]
fn sigil_handles_crlf_inputs() {
    let spell = "#Spell: S\r\n^ Intent: s\r\n\r\n@E\r\n  - network\r\n";
    let book = parse_spellbook(spell).unwrap();
    assert!(book.spells.contains_key("S"));
    let s = book.spells.get("S").unwrap();
    assert!(s.entities.contains_key("E"));
}

#[test]
fn sigil_stress_parses_reasonable_size_quickly() {
    // Keep this moderate so it stays stable in CI.
    let mut text = String::new();
    for i in 0..200u32 {
        text.push_str(&format!("#Spell: S{i}\n^ Intent: stress\n\n@E\n  ! deterministic\n  - network\n  : in -> out\n  > dep\n\n"));
    }

    let book = parse_spellbook(&text).unwrap();
    assert_eq!(book.spells.len(), 200);
}
