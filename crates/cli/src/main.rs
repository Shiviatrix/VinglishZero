use std::{fs, path::Path, process};

use vz_adapter_vinglish::VinglishAdapter;
use vz_diagnostics::{CompilerDiagnostic, SemanticDiagnosticEngine};

mod benchmark;
mod profile;
mod query;
mod registry;
mod transport_input;
mod verification;

use transport_input::SemanticTransportAcquirer;

fn main() {
    let mut args = std::env::args().skip(1);
    let command = args.next();

    match command.as_deref() {
        Some("--help") | Some("-h") | Some("help") => print_help(),
        Some("analyze") => println!("vz analyze: not implemented"),
        Some("explain") => {
            let Some(file) = args.next() else {
                eprintln!("usage: vz explain <source>");
                process::exit(2);
            };
            if args.next().is_some() {
                eprintln!("usage: vz explain <source>");
                process::exit(2);
            }
            let graph = graph_for(&file);
            print!("{}", vz_semantic_engine::explanation::explain(&graph));
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
            let graph = graph_for(&transport_file);
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
            Ok(report) => println!(
                "{}",
                serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".to_owned())
            ),
            Err(error) => {
                eprintln!("profile failed: {error}");
                process::exit(1);
            }
        },
        Some("stats") | Some("cache") => {
            println!("Use `vz profile` for persistent cache and repository performance statistics.")
        }
        Some("verify") => match verification::run() {
            Ok(report) => println!("verified {} patterns", report.pattern_count()),
            Err(error) => {
                eprintln!("verification failed: {error}");
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
        Some("export") => println!("vz export: not implemented"),
        Some("doctor") => println!("vz doctor: not implemented"),
        Some(other) => {
            eprintln!("unknown command: {other}");
            eprintln!("usage: vz <analyze|benchmark|diagnose|explain|export|profile|query|stats|verify|doctor>");
            process::exit(2);
        }
        None => {
            print_help();
        }
    }
}

fn graph_for(file: &str) -> vz_semantic_ir::SemanticGraph {
    let path = Path::new(file);
    if path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("json"))
    {
        let input = SemanticTransportAcquirer::default()
            .acquire(path)
            .unwrap_or_else(|error| {
                eprintln!("cannot obtain Vinglish semantic export: {error}");
                process::exit(1);
            });
        return VinglishAdapter.import_json(&input).unwrap_or_else(|error| {
            eprintln!("cannot import Vinglish semantic export: {error}");
            process::exit(1);
        });
    }
    registry::default_registry()
        .and_then(|registry| {
            registry.semantic_graph(path).map_err(|error| match error {
                vz_adapters::RegistryLookupError::Registry(error) => error,
                vz_adapters::RegistryLookupError::Adapter(error) => {
                    eprintln!("{error}");
                    process::exit(1)
                }
            })
        })
        .unwrap_or_else(|error| {
            eprintln!("{error}");
            process::exit(1)
        })
}

fn print_help() {
    println!("Vinglish Zero deterministic semantic analysis\n\nUsage:\n  vz explain <source>\n  vz diagnose <compiler-diagnostic.json> <source>\n  vz query <expression>\n  vz profile\n  vz stats\n  vz verify\n  vz benchmark\n\nQueries scan registered source files and compiler transport fixtures in the current repository. Source extensions select a registered adapter. Vinglish JSON remains the stable compiler interoperability contract.");
}
