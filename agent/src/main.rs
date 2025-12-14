mod audit;
mod blob_store;
mod bundle;
mod cli;
mod commitment;
mod io;
mod keys;
mod manifest;
mod package_verifier;
mod policy;
mod proof_engine;
mod proof_mock;
mod registry;
mod sign;

// Re-export library modules for use by bin modules (crate::crypto, etc.)
pub use cap_agent::crypto;
pub use cap_agent::verifier;
pub use cap_agent::bundle as cap_bundle;

use clap::Parser;
use cli::{
    AuditCommands, BlobCommands, Cli, Commands, KeysCommands, ManifestCommands,
    PolicyCommands, ProofCommands, RegistryCommands, SignCommands, VerifierCommands,
};
use serde::{Deserialize, Serialize};

const VERSION: &str = "0.2.0"; // Minimal Local Agent (without REST API)

/// Verification Report für manifest verify Kommando
#[allow(dead_code)] // Type definition kept for future use, actual construction in cli::manifest
#[derive(Debug, Serialize, Deserialize)]
struct VerificationReport {
    pub manifest_hash: String,
    pub proof_hash: String,
    pub timestamp_valid: bool,
    pub registry_match: bool,
    pub signature_valid: bool,
    pub status: String,
}

// All run_* functions have been extracted to cli/* modules:
// - cli::prepare (run_prepare, run_inspect)
// - cli::policy (run_policy_validate)
// - cli::manifest (run_manifest_*)
// - cli::proof (run_proof_*, run_zk_*)
// - cli::sign (run_sign_*, run_verify_manifest)
// - cli::verifier (run_verifier_*)
// - cli::audit (run_audit_*)
// - cli::registry (run_lists_*, run_registry_*)
// - cli::keys (run_keys_*)
// - cli::bundle (run_bundle_v2, run_verify_bundle)
// - cli::blob (run_blob_*)

/// Zeigt die Version an
fn run_version() {
    println!("cap-agent v{}", VERSION);
}

fn main() {
    let cli = Cli::parse();

    let result = match &cli.command {
        Commands::Prepare { suppliers, ubos } => cli::prepare::run_prepare(suppliers, ubos),
        Commands::Inspect { path } => cli::prepare::run_inspect(path),
        Commands::Policy(cmd) => match cmd {
            PolicyCommands::Validate { file } => cli::policy::run_policy_validate(file),
            PolicyCommands::Lint { file, strict } => {
                use cap_agent::policy_v2;
                let exit_code = policy_v2::run_lint(file, *strict).unwrap_or(1);
                std::process::exit(exit_code);
            }
            PolicyCommands::Compile { file, output } => {
                use cap_agent::policy_v2;
                let exit_code = policy_v2::run_compile(file, output).unwrap_or(1);
                std::process::exit(exit_code);
            }
            PolicyCommands::Show { file } => {
                use cap_agent::policy_v2;
                let exit_code = policy_v2::run_show(file).unwrap_or(1);
                std::process::exit(exit_code);
            }
        },
        Commands::Manifest(cmd) => match cmd {
            ManifestCommands::Build { policy, out } => {
                cli::manifest::run_manifest_build(policy, out.clone())
            }
            ManifestCommands::Validate { file, schema } => {
                cli::manifest::run_manifest_validate(file, schema.clone())
            }
            ManifestCommands::Verify {
                manifest,
                proof,
                registry,
                timestamp,
                out,
            } => cli::manifest::run_manifest_verify(
                manifest,
                proof,
                registry,
                timestamp.clone(),
                out.clone(),
            ),
        },
        Commands::Proof(cmd) => match cmd {
            ProofCommands::Mock { policy, manifest } => {
                cli::proof::run_proof_mock(policy, manifest)
            }
            ProofCommands::Build { policy, manifest } => {
                cli::proof::run_proof_build(policy, manifest)
            }
            ProofCommands::Verify { proof, manifest } => {
                cli::proof::run_proof_verify_v3(proof, manifest)
            }
            ProofCommands::Export {
                manifest,
                proof,
                timestamp,
                registry,
                report,
                out,
                force,
            } => cli::proof::run_proof_export(
                manifest,
                proof,
                timestamp.clone(),
                registry.clone(),
                report.clone(),
                out.clone(),
                *force,
            ),
            // Note: ZK commands removed in minimal local agent
            ProofCommands::ZkBuild { .. } => {
                eprintln!("ZK-Build nicht verfügbar im minimalen lokalen Agenten");
                Err("ZK commands removed".into())
            }
            ProofCommands::ZkVerify { .. } => {
                eprintln!("ZK-Verify nicht verfügbar im minimalen lokalen Agenten");
                Err("ZK commands removed".into())
            }
            ProofCommands::Bench { .. } => {
                eprintln!("ZK-Bench nicht verfügbar im minimalen lokalen Agenten");
                Err("ZK commands removed".into())
            }
            ProofCommands::Adapt { .. } => {
                eprintln!("Proof-Adapt nicht verfügbar im minimalen lokalen Agenten");
                Err("Orchestrator removed".into())
            }
        },
        Commands::Sign(cmd) => match cmd {
            SignCommands::Keygen { dir } => cli::sign::run_sign_keygen(dir.clone()),
            SignCommands::Manifest {
                key,
                manifest_in,
                out,
                signer,
            } => cli::sign::run_sign_manifest(key, manifest_in, out, signer.clone()),
            SignCommands::VerifyManifest { pub_key, signed_in } => {
                cli::sign::run_verify_manifest(pub_key, signed_in)
            }
        },
        Commands::Verifier(cmd) => match cmd {
            VerifierCommands::Run { package } => cli::verifier::run_verifier_run(package),
            VerifierCommands::Extract { package } => cli::verifier::run_verifier_extract(package),
            VerifierCommands::Audit { package } => cli::verifier::run_verifier_audit(package),
        },
        Commands::Audit(cmd) => match cmd {
            AuditCommands::Tip { out } => cli::audit::run_audit_tip(out.clone()),
            AuditCommands::Anchor {
                kind,
                reference,
                manifest_in,
                manifest_out,
            } => cli::audit::run_audit_anchor(kind, reference, manifest_in, manifest_out),
            AuditCommands::Timestamp {
                head,
                out,
                mock,
                tsa_url,
            } => cli::audit::run_audit_timestamp(head, out.clone(), *mock, tsa_url.clone()),
            AuditCommands::VerifyTimestamp { head, timestamp } => {
                cli::audit::run_audit_verify_timestamp(head, timestamp)
            }
            AuditCommands::SetPrivateAnchor {
                manifest,
                audit_tip,
                created_at,
            } => cli::audit::run_audit_set_private_anchor(manifest, audit_tip, created_at.clone()),
            AuditCommands::SetPublicAnchor {
                manifest,
                chain,
                txid,
                digest,
                created_at,
            } => cli::audit::run_audit_set_public_anchor(
                manifest,
                chain,
                txid,
                digest,
                created_at.clone(),
            ),
            AuditCommands::VerifyAnchor { manifest, out } => {
                cli::audit::run_audit_verify_anchor(manifest, out.clone())
            }
            AuditCommands::Append {
                file,
                event,
                policy_id,
                ir_hash,
                manifest_hash,
                result,
                run_id,
            } => cli::audit::run_audit_append(
                file,
                event,
                policy_id.clone(),
                ir_hash.clone(),
                manifest_hash.clone(),
                result.clone(),
                run_id.clone(),
            ),
            AuditCommands::Verify { file, out } => {
                cli::audit::run_audit_verify_chain(file, out.clone())
            }
            AuditCommands::Export {
                file,
                from,
                to,
                policy_id,
                out,
            } => cli::audit::run_audit_export(
                file,
                from.clone(),
                to.clone(),
                policy_id.clone(),
                out.clone(),
            ),
        },
        // Note: Lists commands removed in minimal local agent
        Commands::Lists(_cmd) => {
            eprintln!("Lists-Commands nicht verfügbar im minimalen lokalen Agenten");
            Err("Lists module removed".into())
        }
        Commands::Registry(cmd) => match cmd {
            RegistryCommands::Add {
                manifest,
                proof,
                timestamp,
                registry,
                backend,
                signing_key,
                validate_key,
                keys_dir,
            } => cli::registry::run_registry_add(
                manifest,
                proof,
                timestamp.clone(),
                registry.clone(),
                backend,
                signing_key.clone(),
                *validate_key,
                keys_dir,
            ),
            RegistryCommands::List { registry, backend } => {
                cli::registry::run_registry_list(registry.clone(), backend)
            }
            RegistryCommands::Verify {
                manifest,
                proof,
                registry,
                backend,
            } => cli::registry::run_registry_verify(manifest, proof, registry.clone(), backend),
            RegistryCommands::Migrate {
                from,
                input,
                to,
                output,
            } => cli::registry::run_registry_migrate(from, input, to, output),
            RegistryCommands::Inspect { registry } => {
                cli::registry::run_registry_inspect(registry.clone())
            }
            RegistryCommands::BackfillKid { registry, output } => {
                cli::registry::run_registry_backfill_kid(registry.clone(), output.clone())
            }
        },
        Commands::Keys(cmd) => match cmd {
            KeysCommands::Keygen {
                owner,
                algo,
                out,
                valid_days,
                comment,
            } => cli::keys::run_keys_keygen(owner, algo, out, *valid_days, comment.clone()),
            KeysCommands::List { dir, status, owner } => {
                cli::keys::run_keys_list(dir, status.clone(), owner.clone())
            }
            KeysCommands::Show { dir, kid } => cli::keys::run_keys_show(dir, kid),
            KeysCommands::Rotate { dir, current, new } => {
                cli::keys::run_keys_rotate(dir, current, new)
            }
            KeysCommands::Attest {
                signer,
                subject,
                out,
            } => cli::keys::run_keys_attest(signer, subject, out),
            KeysCommands::Archive { dir, kid } => cli::keys::run_keys_archive(dir, kid),
            KeysCommands::VerifyChain { dir, attestations } => {
                cli::keys::run_keys_verify_chain(dir, attestations)
            }
        },
        Commands::Blob(cmd) => match cmd {
            BlobCommands::Put {
                file,
                r#type,
                registry,
                link_entry_id,
                stdin,
                out,
                no_dedup,
            } => cli::blob::run_blob_put(
                file.clone(),
                r#type,
                registry,
                link_entry_id.clone(),
                *stdin,
                out.clone(),
                *no_dedup,
            ),
            BlobCommands::Get {
                id,
                out,
                stdout,
                registry,
            } => cli::blob::run_blob_get(id, out.clone(), *stdout, registry),
            BlobCommands::List {
                r#type,
                min_size,
                max_size,
                unused_only,
                limit,
                order,
                registry,
            } => cli::blob::run_blob_list(
                r#type.clone(),
                *min_size,
                *max_size,
                *unused_only,
                *limit,
                order,
                registry,
            ),
            BlobCommands::Gc {
                dry_run,
                force,
                min_age,
                print_ids,
                registry,
            } => cli::blob::run_blob_gc(*dry_run, *force, min_age.clone(), *print_ids, registry),
        },
        Commands::BundleV2 {
            manifest,
            proof,
            verifier_wasm,
            out,
            zip,
            force,
        } => cli::bundle::run_bundle_v2(manifest, proof, verifier_wasm.clone(), out, *zip, *force),
        Commands::VerifyBundle { bundle, out } => cli::bundle::run_verify_bundle(bundle, out.clone()),
        Commands::Version => {
            run_version();
            Ok(())
        }
    };

    if let Err(e) = result {
        eprintln!("❌ Fehler: {}", e);
        std::process::exit(1);
    }
}
