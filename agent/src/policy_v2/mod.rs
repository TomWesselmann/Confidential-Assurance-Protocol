pub mod types;
pub mod yaml_parser;
pub mod linter;
pub mod ir;
pub mod hasher;
pub mod cli;

// Re-export commonly used types
pub use types::{PolicyV2, IrV1, IrRule, IrExpression, Rule};
pub use yaml_parser::{parse_yaml, parse_yaml_str};
pub use linter::{lint, has_errors, http_status_from_diagnostics, LintMode, LintDiagnostic, Level, LintCode};
pub use ir::{generate_ir, canonicalize};
pub use hasher::sha3_256_hex;
pub use cli::{PolicyCli, PolicyCommand, run_lint, run_compile, run_show};
