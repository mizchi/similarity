// Test file for Rust structures with derive attributes

// Structs with common derives
#[derive(Debug, Clone, PartialEq)]
pub struct User {
    pub id: u64,
    pub name: String,
    pub email: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Person {
    pub id: u64,
    pub full_name: String,
    pub email_address: String,
}

// Similar structure but different derives
#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    pub id: u64,
    pub username: String,
    pub email: String,
}

// Completely different derives
#[derive(Default)]
pub struct Profile {
    pub user_id: u64,
    pub display_name: String,
    pub contact_email: String,
}

// Enums with derives
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Active,
    Inactive,
    Pending,
    Suspended,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UserStatus {
    Active,
    Inactive,
    Pending,
    Banned,
}

// Different enum with same derives
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Red,
    Green,
    Blue,
    Yellow,
}

// Complex derives with serde
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ApiResult {
    pub is_success: bool,
    pub error_message: String,
    pub payload: Option<String>,
}

// Structs with custom attributes
#[derive(Debug, Clone)]
#[cfg(feature = "postgres")]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
}

#[derive(Debug, Clone)]
#[cfg(feature = "mysql")]
pub struct DbConfig {
    pub hostname: String,
    pub port_number: u16,
    pub db_name: String,
}

// Generic structs with derives
#[derive(Debug, Clone, PartialEq)]
pub struct Result<T, E> {
    value: Option<T>,
    error: Option<E>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Response<T, E> {
    data: Option<T>,
    err: Option<E>,
}