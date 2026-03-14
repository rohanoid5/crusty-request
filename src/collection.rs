use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app::HttpMethod;
use crate::key_value::KeyValueEntries;

/// A named collection of API requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collection {
    pub id: String,
    pub name: String,
    pub requests: Vec<SavedRequest>,
    pub environments: Vec<Environment>,
}

impl Collection {
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            requests: Vec::new(),
            environments: Vec::new(),
        }
    }

    /// Add a request to the collection
    pub fn add_request(&mut self, request: SavedRequest) {
        self.requests.push(request);
    }

    /// Remove a request by index
    pub fn remove_request(&mut self, index: usize) {
        if index < self.requests.len() {
            self.requests.remove(index);
        }
    }

    /// Add an environment to the collection
    pub fn add_environment(&mut self, env: Environment) {
        self.environments.push(env);
    }

    /// Get the active environment (if any)
    pub fn active_environment(&self) -> Option<&Environment> {
        self.environments.iter().find(|e| e.active)
    }

    /// Set active environment by index, deactivating all others
    pub fn set_active_environment(&mut self, index: usize) {
        for (i, env) in self.environments.iter_mut().enumerate() {
            env.active = i == index;
        }
    }

    /// Deactivate all environments
    pub fn clear_active_environment(&mut self) {
        for env in &mut self.environments {
            env.active = false;
        }
    }
}

/// A saved API request within a collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedRequest {
    pub id: String,
    pub name: String,
    pub method: HttpMethod,
    pub url: String,
    pub headers: KeyValueEntries,
    pub params: KeyValueEntries,
    pub auth: KeyValueEntries,
    pub body: String,
}

impl SavedRequest {
    pub fn new(
        name: String,
        method: HttpMethod,
        url: String,
        headers: KeyValueEntries,
        params: KeyValueEntries,
        auth: KeyValueEntries,
        body: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            method,
            url,
            headers,
            params,
            auth,
            body,
        }
    }
}

/// An environment containing variables
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment {
    pub name: String,
    pub variables: Vec<EnvironmentVariable>,
    pub active: bool,
}

impl Environment {
    pub fn new(name: String) -> Self {
        Self {
            name,
            variables: Vec::new(),
            active: false,
        }
    }

    pub fn add_variable(&mut self, key: String, value: String) {
        self.variables.push(EnvironmentVariable {
            key,
            initial_value: value.clone(),
            current_value: value,
            enabled: true,
        });
    }

    /// Get the current value of a variable by key
    pub fn get_variable(&self, key: &str) -> Option<&str> {
        self.variables
            .iter()
            .find(|v| v.enabled && v.key == key)
            .map(|v| v.current_value.as_str())
    }
}

/// A single environment variable
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentVariable {
    pub key: String,
    pub initial_value: String,
    pub current_value: String,
    pub enabled: bool,
}
