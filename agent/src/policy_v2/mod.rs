pub mod cli;
pub mod hasher;
pub mod ir;
pub mod linter;
pub mod types;
pub mod yaml_parser;

// Re-export commonly used types
pub use cli::{run_compile, run_lint, run_show, PolicyCli, PolicyCommand};
pub use hasher::sha3_256_hex;
pub use ir::{canonicalize, generate_ir};
pub use linter::{
    has_errors, http_status_from_diagnostics, lint, Level, LintCode, LintDiagnostic, LintMode,
};
pub use types::{IrExpression, IrRule, IrV1, PolicyV2, Rule};
pub use yaml_parser::{parse_yaml, parse_yaml_str};
