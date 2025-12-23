use std::fs;
use std::path::PathBuf;

use glyph_verify::{compare_spellbooks, parse_spellbook, CompareOptions};

fn usage() -> ! {
    eprintln!(
        "Usage: glyph-verify <spell_path> <invocation_path> [--deny-extra] [--strict-intent] [--allow-open-questions]\n\n\
Compares a canonical Spellbook against an Invocation (both in Glyph) and returns pass/fail.\n\
- By default, extra items in the invocation are allowed (not checked).\n\
- By default, intent must exist but may differ (use --strict-intent to require exact match).\n\
- By default, open-question lines ('?' as leading glyph) are forbidden in the invocation.\n"
    );
    std::process::exit(2);
}

fn main() {
    let mut args = std::env::args().skip(1);

    let spell_path: PathBuf = match args.next() {
        Some(v) => v.into(),
        None => usage(),
    };

    let invocation_path: PathBuf = match args.next() {
        Some(v) => v.into(),
        None => usage(),
    };

    let mut opts = CompareOptions::default();

    for arg in args {
        match arg.as_str() {
            "--deny-extra" => opts.deny_extra = true,
            "--strict-intent" => opts.strict_intent = true,
            "--allow-open-questions" => opts.deny_open_questions_in_invocation = false,
            "-h" | "--help" => usage(),
            other => {
                eprintln!("unknown option: {other}\n");
                usage();
            }
        }
    }

    let spell_text = match fs::read_to_string(&spell_path) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("failed to read spell file {}: {e}", spell_path.display());
            std::process::exit(2);
        }
    };

    let invocation_text = match fs::read_to_string(&invocation_path) {
        Ok(v) => v,
        Err(e) => {
            eprintln!(
                "failed to read invocation file {}: {e}",
                invocation_path.display()
            );
            std::process::exit(2);
        }
    };

    let spellbook = match parse_spellbook(&spell_text) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("failed to parse spellbook: {e}");
            std::process::exit(2);
        }
    };

    let invocation = match parse_spellbook(&invocation_text) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("failed to parse invocation: {e}");
            std::process::exit(2);
        }
    };

    let report = compare_spellbooks(&spellbook, &invocation, &opts);

    if !report.errors.is_empty() {
        eprintln!("NOT BOUND");
        for err in report.errors {
            eprintln!("- {err}");
        }
        std::process::exit(1);
    }

    if !report.warnings.is_empty() {
        eprintln!("WARN");
        for w in report.warnings {
            eprintln!("- {w}");
        }
    }

    eprintln!("BOUND");
}
