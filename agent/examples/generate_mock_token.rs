/// Helper: Generate Mock JWT Token for Testing
///
/// Usage: cargo run --example generate_mock_token

use cap_agent::api::auth::generate_mock_token;
use cap_agent::api::auth::Claims;

fn main() {
    let now = chrono::Utc::now().timestamp() as usize;

    let claims = Claims {
        sub: "test-client-12345".to_string(),
        iss: "https://auth.example.com".to_string(),
        aud: "cap-verifier".to_string(),
        exp: now + 3600, // Valid for 1 hour
        iat: now,
        scope: "verify:read".to_string(),
    };

    let token = generate_mock_token(claims);

    println!("==================================================");
    println!("Mock JWT Token (valid for 1 hour):");
    println!("==================================================");
    println!("{}", token);
    println!("==================================================");
    println!("");
    println!("Usage:");
    println!("  curl -H 'Authorization: Bearer {}' \\", token);
    println!("       http://localhost:8080/verify");
}
