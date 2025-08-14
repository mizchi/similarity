// Test file for Rust structure comparison

// User struct with common fields
#[derive(Debug, Clone)]
pub struct User {
    pub id: u64,
    pub name: String,
    pub email: String,
    pub age: Option<u32>,
}

// Person struct with same fields (should be detected as similar)
#[derive(Debug, Clone)]
pub struct Person {
    pub id: u64,
    pub name: String,
    pub email: String,
    pub age: Option<u32>,
}

// Customer struct with same structure
#[derive(Debug)]
struct Customer {
    id: u64,
    name: String,
    email: String,
    age: Option<u32>,
}

// Admin struct with different field (role instead of email)
pub struct Admin {
    pub id: u64,
    pub name: String,
    pub role: String,
    pub age: Option<u32>,
}

// Result-like enum
pub enum MyResult<T, E> {
    Ok(T),
    Err(E),
}

// Another Result-like enum (should be detected as similar)
pub enum CustomResult<V, F> {
    Success(V),
    Failure(F),
}

// Option-like enum
pub enum MyOption<T> {
    Some(T),
    None,
}

// Status enum with different variants
pub enum Status {
    Pending,
    Active,
    Inactive,
    Deleted,
}

// Similar status enum with slightly different names
pub enum UserStatus {
    Waiting,
    Enabled,
    Disabled,
    Removed,
}

// Complex enum with different variant types
pub enum Message {
    Text(String),
    Number(i32),
    Struct { x: f64, y: f64 },
    Empty,
}

// Tuple struct
pub struct Point(f64, f64, f64);

// Another tuple struct with same structure
pub struct Vector(f64, f64, f64);