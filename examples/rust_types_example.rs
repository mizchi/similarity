// Example file with similar Rust types for testing

// Very similar structs with different names
struct User {
    id: u64,
    name: String,
    email: String,
    created_at: std::time::SystemTime,
}

struct Person {
    id: u64,
    full_name: String,
    email_address: String,
    birth_date: std::time::SystemTime,
}

struct Customer {
    customer_id: u64,
    customer_name: String,
    contact_email: String,
    registration_date: std::time::SystemTime,
}

// Similar enums
enum Status {
    Active,
    Inactive,
    Pending,
    Completed,
}

enum State {
    Running,
    Stopped,
    Waiting,
    Finished,
}

enum TaskStatus {
    InProgress,
    Paused,
    Queued,
    Done,
}

// Generic structs
struct Response<T> {
    data: T,
    status: u16,
    message: String,
}

struct ApiResult<T> {
    result: T,
    code: u16,
    description: String,
}

struct ServerResponse<T> {
    payload: T,
    status_code: u16,
    error_message: String,
}

// Nested structs
struct ComplexUser {
    id: u64,
    profile: UserProfile,
    settings: UserSettings,
}

struct UserProfile {
    name: String,
    email: String,
    phone: String,
}

struct UserSettings {
    theme: String,
    notifications: bool,
}

struct ComplexPerson {
    person_id: u64,
    person_profile: PersonProfile,
    person_settings: PersonSettings,
}

struct PersonProfile {
    full_name: String,
    email_address: String,
    phone_number: String,
}

struct PersonSettings {
    ui_theme: String,
    enable_notifications: bool,
}

// Different structures
struct Product {
    sku: String,
    name: String,
    price: f64,
    in_stock: bool,
}

struct Order {
    order_id: String,
    items: Vec<String>,
    total: f64,
    paid: bool,
}

// Type aliases
type UserId = u64;
type CustomerId = u64;
type OrderId = String;