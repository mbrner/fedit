#!/usr/bin/env rustx
// Simple manpage generator for FEdit-like CLIs using clap_mangen
// This binary defines a Clap-based CLI that mirrors the fedit CLI surface
// and can generate a manpage via clap_mangen.

use std::path::PathBuf;

// Re-export for ease of maintenance; in real usage you may want to split
// the CLI into a shared library. This is a self-contained example.
use clap::{Arg, Command};
use clap_mangen::Man;

fn build_cli() -> Command<'static> {
    Command::new("fedit")
        .about(
            "Whitespace-insensitive search and replace with encoding and line ending preservation.",
        )
        .arg(Arg::new("path").help("Path to the target file"))
        .arg(Arg::new("search").help("Search string to replace (may contain whitespace)"))
        .arg(Arg::new("replace").help("Replacement string"))
        .arg(
            Arg::new("encoding")
                .short('e')
                .long("encoding")
                .value_name("ENCODING")
                .default_value("utf-8")
                .help("File encoding to use"),
        )
        .arg(
            Arg::new("multiple")
                .short('m')
                .long("multiple")
                .help("Replace all occurrences when multiple matches exist"),
        )
        .arg(
            Arg::new("dry_run")
                .short('n')
                .long("dry-run")
                .help("Preview changes without modifying the file"),
        )
        .arg(
            Arg::new("ignore_whitespace")
                .short('w')
                .long("ignore-whitespace")
                .help("Whitespace-insensitive search"),
        )
        .arg(
            Arg::new("structured")
                .short('s')
                .long("structured")
                .help("Structured mode: exact key-path matching"),
        )
        .arg(
            Arg::new("gen-man")
                .long("gen-man")
                .value_name("PATH")
                .help("Generate a manpage to PATH and exit"),
        )
}

fn main() {
    // Build CLI and parse, then optionally generate a manpage.
    let mut app = build_cli();
    let matches = app.clone().get_matches();

    if let Some(out_path) = matches.value_of("gen-man") {
        // Generate a manpage from the CLI definition.
        // Clap-mangen expects a Clap App, so we use the App we built above.
        // Best-effort: write to the specified path.
        let mut buf: Vec<u8> = Vec::new();
        let man = Man::new(app);
        // render writes into a writer implementing std::io::Write
        // The exact API may vary by clap_mangen version; handle the common case.
        if let Err(e) = man.render(&mut buf) {
            eprintln!("Failed to render manpage: {}", e);
            std::process::exit(1);
        }
        let p = PathBuf::from(out_path);
        if let Err(e) = std::fs::write(&p, buf) {
            eprintln!("Failed to write manpage to {}: {}", p.display(), e);
            std::process::exit(1);
        }
        println!("Wrote manpage to {}", p.display());
        return;
    }

    // Placeholder: actual runtime behavior would implement the fedit feature set.
    // For the scope of this patch, print a brief teaser and exit.
    eprintln!("This binary is a helper to generate the fedit manpage. Use --gen-man <path> to produce the manpage.");
    std::process::exit(0);
}
