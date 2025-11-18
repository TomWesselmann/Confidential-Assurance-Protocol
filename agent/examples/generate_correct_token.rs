/// Generate Mock JWT Token with correct scope
use cap_agent::api::auth::generate_mock_token;
use cap_agent::api::auth::Claims;

fn main() {
    let now = chrono::Utc::now().timestamp() as usize;

    let claims = Claims {
        sub: "test-client-12345".to_string(),
        iss: "https://auth.example.com".to_string(),
        aud: "cap-verifier".to_string(),
        exp: now + 36000, // Valid for 10 hours
        iat: now,
        scope: "verify:run".to_string(), // Correct scope from auth.yaml
    };

    let token = generate_mock_token(claims);

    println!("==================================================");
    println!("Mock JWT Token (valid for 10 hours, scope: verify:run):");
    println!("==================================================");
    println!("{}", token);
    println!("==================================================");
}
