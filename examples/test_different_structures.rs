// Test file to verify that clearly different structures are not detected as similar

// Simple enum with few variants
pub enum Color {
    Red,
    Green,
    Blue,
}

// Complex struct with many fields
pub struct DatabaseConnection {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database_name: String,
    pub max_connections: u32,
    pub timeout: std::time::Duration,
    pub use_ssl: bool,
    pub certificate_path: Option<String>,
}

// Unit struct (no fields)
pub struct EmptyMarker;

// Tuple struct with single field
pub struct Id(u64);

// Enum with complex variants
pub enum Request {
    Get { url: String, headers: Vec<String> },
    Post { url: String, body: Vec<u8>, headers: Vec<String> },
    Put { url: String, body: Vec<u8> },
    Delete { url: String },
}

// Simple struct with two fields
pub struct Point2D {
    pub x: f64,
    pub y: f64,
}

// Another enum with different structure
pub enum Shape {
    Circle(f64),
    Rectangle(f64, f64),
    Triangle(f64, f64, f64),
    Polygon(Vec<(f64, f64)>),
}

// Struct that might look similar to Point2D but has different types
pub struct Coordinate {
    pub lat: f64,
    pub lng: f64,
}

// Large struct that should not match with anything
pub struct Configuration {
    pub app_name: String,
    pub version: String,
    pub environment: String,
    pub debug_mode: bool,
    pub log_level: String,
    pub api_key: String,
    pub secret_key: String,
    pub endpoints: Vec<String>,
    pub features: std::collections::HashMap<String, bool>,
    pub limits: std::collections::HashMap<String, u32>,
}