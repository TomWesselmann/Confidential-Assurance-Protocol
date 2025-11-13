use super::*;
use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "policy")]
#[command(about = "Policy compiler CLI")]
pub struct PolicyCli {
    #[command(subcommand)]
    pub command: PolicyCommand,
}

#[derive(Subcommand)]
pub enum PolicyCommand {
    /// Lint a policy file
    Lint {
        /// Path to policy YAML file
        file: String,
        /// Use strict linting mode
        #[arg(long)]
        strict: bool,
    },
    /// Compile policy to IR v1
    Compile {
        /// Path to policy YAML file
        file: String,
        /// Output IR JSON file
        #[arg(short, long)]
        output: String,
    },
    /// Show IR in human-readable format
    Show {
        /// Path to IR JSON file
        file: String,
    },
}

pub fn run_lint(file: &str, strict: bool) -> Result<i32> {
    let policy = yaml_parser::parse_yaml(file)?;
    let mode = if strict {
        LintMode::Strict
    } else {
        LintMode::Relaxed
    };
    let diagnostics = lint(&policy, mode);

    for diag in &diagnostics {
        let prefix = match diag.level {
            Level::Error => "ERROR",
            Level::Warning => "WARN",
        };
        if let Some(rule_id) = &diag.rule_id {
            println!("[{}] {}: {}", prefix, rule_id, diag.message);
        } else {
            println!("[{}] {}", prefix, diag.message);
        }
    }

    if has_errors(&diagnostics) {
        Ok(3) // Lint error exit code
    } else if !diagnostics.is_empty() {
        Ok(2) // Warnings exit code
    } else {
        println!("✅ Policy is valid");
        Ok(0)
    }
}

pub fn run_compile(file: &str, output: &str) -> Result<i32> {
    // Parse
    let policy = yaml_parser::parse_yaml(file)?;

    // Lint (strict)
    let diagnostics = lint(&policy, LintMode::Strict);
    if has_errors(&diagnostics) {
        for diag in &diagnostics {
            eprintln!("ERROR: {}", diag.message);
        }
        return Ok(3);
    }

    // Compute policy hash
    let policy_json = serde_json::to_string(&policy)?;
    let policy_hash = sha3_256_hex(&policy_json);

    // Generate IR
    let mut ir = generate_ir(&policy, policy_hash)?;

    // Compute IR hash
    let ir_canonical = canonicalize(&ir)?;
    let ir_hash = sha3_256_hex(&ir_canonical);
    ir.ir_hash = ir_hash;

    // Write IR
    let ir_json = serde_json::to_string_pretty(&ir)?;
    std::fs::write(output, &ir_json)?;

    println!("✅ Compiled policy to {}", output);
    println!("   policy_hash: {}", ir.policy_hash);
    println!("   ir_hash: {}", ir.ir_hash);

    Ok(0)
}

pub fn run_show(file: &str) -> Result<i32> {
    let ir_json = std::fs::read_to_string(file)?;
    let ir: types::IrV1 = serde_json::from_str(&ir_json)?;

    println!("Policy ID: {}", ir.policy_id);
    println!("IR Version: {}", ir.ir_version);
    println!("Policy Hash: {}", ir.policy_hash);
    println!("IR Hash: {}", ir.ir_hash);
    println!("\nRules ({}):", ir.rules.len());
    for rule in &ir.rules {
        println!("  - {} ({})", rule.id, rule.op);
    }

    if let Some(adaptivity) = &ir.adaptivity {
        println!("\nAdaptivity:");
        println!("  Predicates: {}", adaptivity.predicates.len());
        println!("  Activations: {}", adaptivity.activations.len());
    }

    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_run_lint_valid() {
        let exit_code = run_lint("examples/lksg_v1.policy.yml", true).unwrap();
        assert_eq!(exit_code, 0);
    }

    #[test]
    fn test_run_compile() {
        let output = "/tmp/test_compile.ir.json";
        let exit_code = run_compile("examples/lksg_v1.policy.yml", output).unwrap();
        assert_eq!(exit_code, 0);
        assert!(std::path::Path::new(output).exists());

        // Cleanup
        let _ = fs::remove_file(output);
    }

    #[test]
    fn test_run_show() {
        let exit_code = run_show("examples/lksg_v1.ir.json").unwrap();
        assert_eq!(exit_code, 0);
    }
}
