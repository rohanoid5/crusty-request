use crate::collection::Environment;

/// Interpolate {{variable}} placeholders in text using the active environment
pub fn interpolate(text: &str, env: &Environment) -> String {
    let mut result = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '{' && chars.peek() == Some(&'{') {
            // Consume the second '{'
            chars.next();

            // Read variable name until '}}'
            let mut var_name = String::new();
            let mut found_close = false;

            while let Some(inner) = chars.next() {
                if inner == '}' && chars.peek() == Some(&'}') {
                    chars.next(); // consume second '}'
                    found_close = true;
                    break;
                }
                var_name.push(inner);
            }

            if found_close {
                let trimmed = var_name.trim();
                if let Some(value) = env.get_variable(trimmed) {
                    result.push_str(value);
                } else {
                    // Variable not found — keep original placeholder
                    result.push_str("{{");
                    result.push_str(&var_name);
                    result.push_str("}}");
                }
            } else {
                // Unclosed {{ — emit as-is
                result.push_str("{{");
                result.push_str(&var_name);
            }
        } else {
            result.push(c);
        }
    }

    result
}

/// Interpolate variables in all request fields
pub fn interpolate_all(
    url: &str,
    headers: &crate::key_value::KeyValueEntries,
    params: &crate::key_value::KeyValueEntries,
    auth: &crate::key_value::KeyValueEntries,
    body: &str,
    env: &Environment,
) -> (
    String,
    crate::key_value::KeyValueEntries,
    crate::key_value::KeyValueEntries,
    crate::key_value::KeyValueEntries,
    String,
) {
    let url = interpolate(url, env);
    let body = interpolate(body, env);

    let headers = interpolate_entries(headers, env);
    let params = interpolate_entries(params, env);
    let auth = interpolate_entries(auth, env);

    (url, headers, params, auth, body)
}

/// Interpolate variables in key-value entries
fn interpolate_entries(
    entries: &crate::key_value::KeyValueEntries,
    env: &Environment,
) -> crate::key_value::KeyValueEntries {
    let mut result = entries.clone();
    for entry in &mut result.entries {
        entry.key = crate::text_input::TextInput::from(interpolate(&entry.key, env));
        entry.value = crate::text_input::TextInput::from(interpolate(&entry.value, env));
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_env() -> Environment {
        let mut env = Environment::new("test".to_string());
        env.add_variable("base_url".to_string(), "https://api.example.com".to_string());
        env.add_variable("token".to_string(), "abc123".to_string());
        env.add_variable("port".to_string(), "3001".to_string());
        env
    }

    #[test]
    fn test_simple_interpolation() {
        let env = make_env();
        let result = interpolate("{{base_url}}/users", &env);
        assert_eq!(result, "https://api.example.com/users");
    }

    #[test]
    fn test_multiple_variables() {
        let env = make_env();
        let result = interpolate("{{base_url}}:{{port}}/api", &env);
        assert_eq!(result, "https://api.example.com:3001/api");
    }

    #[test]
    fn test_missing_variable() {
        let env = make_env();
        let result = interpolate("{{unknown}}/path", &env);
        assert_eq!(result, "{{unknown}}/path");
    }

    #[test]
    fn test_no_variables() {
        let env = make_env();
        let result = interpolate("https://example.com/path", &env);
        assert_eq!(result, "https://example.com/path");
    }

    #[test]
    fn test_whitespace_in_variable() {
        let env = make_env();
        let result = interpolate("{{ base_url }}/users", &env);
        assert_eq!(result, "https://api.example.com/users");
    }

    #[test]
    fn test_unclosed_braces() {
        let env = make_env();
        let result = interpolate("{{base_url", &env);
        assert_eq!(result, "{{base_url");
    }

    #[test]
    fn test_empty_string() {
        let env = make_env();
        let result = interpolate("", &env);
        assert_eq!(result, "");
    }

    #[test]
    fn test_bearer_token() {
        let env = make_env();
        let result = interpolate("Bearer {{token}}", &env);
        assert_eq!(result, "Bearer abc123");
    }
}
