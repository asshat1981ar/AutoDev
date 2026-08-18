use std::fs;
use std::path::{Path, PathBuf};

use forge_core::{compare_reports, EvalReport};
use serde::Serialize;
use thiserror::Error;

use crate::{load_corpus, smoke_fixture};

#[derive(Debug, Error)]
enum CliError {
    #[error("{0}")]
    Usage(String),
    #[error("{0}")]
    Runtime(String),
}

#[derive(Debug, Serialize)]
struct ValidateSummary {
    task_count: usize,
    tasks: Vec<ValidateTask>,
}

#[derive(Debug, Serialize)]
struct ValidateTask {
    id: String,
    task_fingerprint: String,
    verifier_fingerprint: String,
}

#[derive(Debug, Serialize)]
struct SmokeSummary {
    results: Vec<SmokeTask>,
}

#[derive(Debug, Serialize)]
struct SmokeTask {
    task_id: String,
    base_passed: bool,
    reference_passed: bool,
}

pub fn run_cli(args: &[String]) -> i32 {
    match dispatch(args) {
        Ok(code) => code,
        Err(CliError::Usage(message)) => {
            if !message.is_empty() {
                eprintln!("error: {message}");
            }
            eprintln!("{}", usage());
            2
        }
        Err(CliError::Runtime(message)) => {
            eprintln!("error: {message}");
            1
        }
    }
}

fn dispatch(args: &[String]) -> Result<i32, CliError> {
    match args.first().map(String::as_str) {
        Some("validate") => validate_command(&args[1..]),
        Some("smoke") => smoke_command(&args[1..]),
        Some("compare") => compare_command(&args[1..]),
        _ => Err(CliError::Usage(String::new())),
    }
}

fn validate_command(args: &[String]) -> Result<i32, CliError> {
    let fixtures = single_option(args, "--fixtures", "validate --fixtures <dir>")?;
    let corpus = load_corpus(fixtures).map_err(runtime)?;
    let mut tasks = Vec::with_capacity(corpus.len());
    for fixture in corpus {
        let key = fixture.task.key().map_err(runtime)?;
        tasks.push(ValidateTask {
            id: key.task_id,
            task_fingerprint: key.task_fingerprint,
            verifier_fingerprint: key.verifier_fingerprint,
        });
    }
    tasks.sort_by(|left, right| left.id.cmp(&right.id));
    print_json(&ValidateSummary {
        task_count: tasks.len(),
        tasks,
    })?;
    Ok(0)
}

fn smoke_command(args: &[String]) -> Result<i32, CliError> {
    let (fixtures, source_repo) = two_options(
        args,
        "--fixtures",
        "--source-repo",
        "smoke --fixtures <dir> --source-repo <path>",
    )?;
    let corpus = load_corpus(fixtures).map_err(runtime)?;
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut results = Vec::with_capacity(corpus.len());
    for fixture in corpus {
        let result =
            smoke_fixture(&fixture, Path::new(source_repo), &crate_root).map_err(runtime)?;
        results.push(SmokeTask {
            task_id: result.task_id,
            base_passed: result.base_passed,
            reference_passed: result.reference_passed,
        });
    }
    results.sort_by(|left, right| left.task_id.cmp(&right.task_id));
    let healthy = results
        .iter()
        .all(|result| !result.base_passed && result.reference_passed);
    print_json(&SmokeSummary { results })?;
    Ok(if healthy { 0 } else { 1 })
}

fn compare_command(args: &[String]) -> Result<i32, CliError> {
    let (baseline, candidate) = two_options(
        args,
        "--baseline",
        "--candidate",
        "compare --baseline <report.json> --candidate <report.json>",
    )?;
    let baseline = read_report(baseline)?;
    let candidate = read_report(candidate)?;
    print_json(&compare_reports(&baseline, &candidate))?;
    Ok(0)
}

fn read_report(path: &str) -> Result<EvalReport, CliError> {
    let bytes = fs::read(path).map_err(runtime)?;
    serde_json::from_slice(&bytes).map_err(runtime)
}

fn single_option<'a>(
    args: &'a [String],
    option: &str,
    expected: &str,
) -> Result<&'a str, CliError> {
    if args.len() == 2 && args[0] == option {
        Ok(args[1].as_str())
    } else {
        Err(CliError::Usage(expected.into()))
    }
}

fn two_options<'a>(
    args: &'a [String],
    first: &str,
    second: &str,
    expected: &str,
) -> Result<(&'a str, &'a str), CliError> {
    if args.len() == 4 && args[0] == first && args[2] == second {
        Ok((args[1].as_str(), args[3].as_str()))
    } else {
        Err(CliError::Usage(expected.into()))
    }
}

fn print_json(value: &impl Serialize) -> Result<(), CliError> {
    let json = serde_json::to_string_pretty(value).map_err(runtime)?;
    println!("{json}");
    Ok(())
}

fn runtime(error: impl std::fmt::Display) -> CliError {
    CliError::Runtime(error.to_string())
}

fn usage() -> &'static str {
    "usage: autodev-eval <validate|smoke|compare> ..."
}
