use std::{fs, path::Path, process};

use vz_diagnostics::{CompilerDiagnostic, SemanticDiagnosticEngine};

mod benchmark;
mod profile;
mod query;
mod registry;
mod validation;
mod verification;

fn main() {
    let mut args = std::env::args().skip(1);
    let command = args.next();

    match command.as_deref() {
        Some("--help") | Some("-h") | Some("help") => print_help(),
        Some("analyze") => {
            eprintln!("`vz analyze` is not a public 1.0 command. Use `vz explain <source>` or `vz verify`.");
            process::exit(2);
        }
        Some("explain") => {
            let Some(file) = args.next() else {
                eprintln!("usage: vz explain <source>");
                process::exit(2);
            };
            if args.next().is_some() {
                eprintln!("usage: vz explain <source>");
                process::exit(2);
            }
            let graph = match graph_for(&file) {
                Ok(graph) => graph,
                Err(error) => exit_with_error(error),
            };
            let report = vz_reasoning::ReasoningEngine::new().analyze(&graph);
            print!("{}", vz_reasoning::render_explanation(&report));
        }
        Some("diagnose") => {
            let Some(diagnostic_file) = args.next() else {
                eprintln!("usage: vz diagnose <compiler-diagnostic.json> <source>");
                process::exit(2);
            };
            let Some(transport_file) = args.next() else {
                eprintln!("usage: vz diagnose <compiler-diagnostic.json> <source>");
                process::exit(2);
            };
            if args.next().is_some() {
                eprintln!("usage: vz diagnose <compiler-diagnostic.json> <source>");
                process::exit(2);
            }
            let diagnostic_input = match fs::read_to_string(&diagnostic_file) {
                Ok(input) => input,
                Err(error) => {
                    eprintln!("cannot read '{diagnostic_file}': {error}");
                    process::exit(1);
                }
            };
            let compiler_diagnostic: CompilerDiagnostic =
                match serde_json::from_str(&diagnostic_input) {
                    Ok(diagnostic) => diagnostic,
                    Err(error) => {
                        eprintln!("cannot decode compiler diagnostic: {error}");
                        process::exit(1);
                    }
                };
            let graph = match graph_for(&transport_file) {
                Ok(graph) => graph,
                Err(error) => exit_with_error(error),
            };
            let intent_report = vz_reasoning::ReasoningEngine::new().analyze(&graph);
            let semantic_diagnostic = match SemanticDiagnosticEngine::new()
                .diagnose(compiler_diagnostic, &intent_report)
            {
                Ok(diagnostic) => diagnostic,
                Err(error) => {
                    eprintln!("cannot create semantic diagnostic: {error}");
                    process::exit(1);
                }
            };
            match serde_json::to_string_pretty(&semantic_diagnostic) {
                Ok(output) => println!("{output}"),
                Err(error) => {
                    eprintln!("cannot serialize semantic diagnostic: {error}");
                    process::exit(1);
                }
            }
        }
        Some("query") => {
            let Some(expression) = args.next() else {
                eprintln!("usage: vz query <expression>");
                process::exit(2);
            };
            if args.next().is_some() {
                eprintln!("usage: vz query <expression>");
                process::exit(2);
            }
            match query::run(&expression, Path::new(".")) {
                Ok(report) => match serde_json::to_string_pretty(&report) {
                    Ok(output) => println!("{output}"),
                    Err(error) => {
                        eprintln!("cannot serialize query report: {error}");
                        process::exit(1);
                    }
                },
                Err(error) => {
                    eprintln!("query failed: {error}");
                    process::exit(1);
                }
            }
        }
        Some("profile") => match profile::run(Path::new(".")) {
            Ok(report) => match serde_json::to_string_pretty(&report) {
                Ok(output) => println!("{output}"),
                Err(error) => {
                    eprintln!("cannot serialize performance profile: {error}");
                    process::exit(1);
                }
            },
            Err(error) => {
                eprintln!("profile failed: {error}");
                process::exit(1);
            }
        },
        Some("stats") => {
            if args.next().is_some() {
                eprintln!("usage: vz stats");
                process::exit(2);
            }
            print_cache_stats();
        }
        Some("cache") => match (args.next(), args.next()) {
            (Some(command), None) if command == "stats" => print_cache_stats(),
            _ => {
                eprintln!("usage: vz cache stats");
                process::exit(2);
            }
        },
        Some("verify") => match verification::run() {
            Ok(report) => println!("verified {} patterns", report.pattern_count()),
            Err(error) => {
                eprintln!("verification failed: {error}");
                process::exit(1);
            }
        },
        Some("validate") => match validation::run() {
            Ok(report) => println!(
                "validation passed: {} semantic patterns",
                report.semantic_patterns.passed
            ),
            Err(error) => {
                eprintln!("validation failed: {error}");
                process::exit(1);
            }
        },
        Some("benchmark") => match benchmark::run() {
            Ok(_) => println!("benchmark reports written to verification/"),
            Err(error) => {
                eprintln!("benchmark failed: {error}");
                process::exit(1);
            }
        },
        Some("export") => {
            eprintln!("`vz export` is not a Vinglish Zero command. For Vinglish source, use the compiler-owned `vng --emit-ir <source>` transport.");
            process::exit(2);
        }
        Some("doctor") => {
            eprintln!("`vz doctor` is not a public 1.0 command. Use `vz validate`, `vz verify`, and `vz profile` for repository health checks.");
            process::exit(2);
        }
        Some(other) => {
            eprintln!("unknown command: {other}");
            eprintln!(
                "usage: vz <benchmark|cache|diagnose|explain|profile|query|stats|validate|verify>"
            );
            process::exit(2);
        }
        None => {
            print_help();
        }
    }
}

fn graph_for(file: &str) -> Result<vz_semantic_ir::SemanticGraph, String> {
    let path = Path::new(file);
    registry::default_registry()
        .map_err(|error| error.to_string())?
        .semantic_graph(path)
        .map_err(|error| error.to_string())
}

fn exit_with_error(error: String) -> ! {
    eprintln!("{error}");
    process::exit(1);
}

fn print_cache_stats() {
    match profile::cache_stats(Path::new(".")) {
        Ok(report) => match serde_json::to_string_pretty(&report) {
            Ok(output) => println!("{output}"),
            Err(error) => {
                eprintln!("cannot serialize cache statistics: {error}");
                process::exit(1);
            }
        },
        Err(error) => {
            eprintln!("cannot read semantic cache statistics: {error}");
            process::exit(1);
        }
    }
}

fn print_help() {
    println!("Vinglish Zero deterministic semantic analysis\n\nUsage:\n  vz explain <source>\n  vz diagnose <compiler-diagnostic.json> <source>\n  vz query <expression>\n  vz profile\n  vz stats\n  vz cache stats\n  vz verify\n  vz validate\n  vz benchmark\n\nQueries scan registered source files and compiler transport fixtures in the current repository. Source extensions select a registered adapter. `vz stats` reads the persistent semantic cache without invoking a frontend. Vinglish JSON remains the stable compiler interoperability contract.");
}
