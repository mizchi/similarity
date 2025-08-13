// Test patterns for Rust similarity detection

// Pattern 1: Similar error handling functions with minor differences
fn read_file_v1(path: &str) -> Result<String, std::io::Error> {
    use std::fs;
    let content = fs::read_to_string(path)?;
    Ok(content)
}

fn read_file_v2(file_path: &str) -> Result<String, std::io::Error> {
    use std::fs;
    let data = fs::read_to_string(file_path)?;
    Ok(data)
}

// Pattern 2: Iterator chains with different variable names
fn process_numbers_v1(numbers: Vec<i32>) -> Vec<i32> {
    numbers
        .iter()
        .filter(|&n| n > &0)
        .map(|n| n * 2)
        .collect()
}

fn process_numbers_v2(nums: Vec<i32>) -> Vec<i32> {
    nums
        .iter()
        .filter(|&x| x > &0)
        .map(|x| x * 2)
        .collect()
}

// Pattern 3: Struct with similar methods
struct Calculator {
    value: f64,
}

impl Calculator {
    fn add(&mut self, x: f64) -> f64 {
        self.value += x;
        self.value
    }
    
    fn subtract(&mut self, x: f64) -> f64 {
        self.value -= x;
        self.value
    }
}

struct Accumulator {
    total: f64,
}

impl Accumulator {
    fn add(&mut self, amount: f64) -> f64 {
        self.total += amount;
        self.total
    }
    
    fn subtract(&mut self, amount: f64) -> f64 {
        self.total -= amount;
        self.total
    }
}

// Pattern 4: Match expressions with similar structure
fn handle_option_v1(opt: Option<i32>) -> String {
    match opt {
        Some(value) => format!("Value: {}", value),
        None => String::from("No value"),
    }
}

fn handle_option_v2(maybe_val: Option<i32>) -> String {
    match maybe_val {
        Some(v) => format!("Value: {}", v),
        None => String::from("No value"),
    }
}

// Pattern 5: Generic functions with similar logic
fn find_max<T: Ord>(items: &[T]) -> Option<&T> {
    if items.is_empty() {
        return None;
    }
    let mut max = &items[0];
    for item in items {
        if item > max {
            max = item;
        }
    }
    Some(max)
}

fn get_maximum<T: Ord>(elements: &[T]) -> Option<&T> {
    if elements.is_empty() {
        return None;
    }
    let mut maximum = &elements[0];
    for elem in elements {
        if elem > maximum {
            maximum = elem;
        }
    }
    Some(maximum)
}

// Pattern 6: Async functions with similar structure
async fn fetch_data_v1(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let response = reqwest::get(url).await?;
    let body = response.text().await?;
    Ok(body)
}

async fn fetch_data_v2(endpoint: &str) -> Result<String, Box<dyn std::error::Error>> {
    let resp = reqwest::get(endpoint).await?;
    let content = resp.text().await?;
    Ok(content)
}

// Pattern 7: Different implementations (should have low similarity)
fn bubble_sort(mut arr: Vec<i32>) -> Vec<i32> {
    let n = arr.len();
    for i in 0..n {
        for j in 0..n - i - 1 {
            if arr[j] > arr[j + 1] {
                arr.swap(j, j + 1);
            }
        }
    }
    arr
}

fn quick_sort(mut arr: Vec<i32>) -> Vec<i32> {
    if arr.len() <= 1 {
        return arr;
    }
    let pivot = arr.remove(0);
    let mut left = vec![];
    let mut right = vec![];
    for x in arr {
        if x <= pivot {
            left.push(x);
        } else {
            right.push(x);
        }
    }
    let mut result = quick_sort(left);
    result.push(pivot);
    result.extend(quick_sort(right));
    result
}

// Pattern 8: Simple vs complex implementation (should have low similarity)
fn is_even_simple(n: i32) -> bool {
    n % 2 == 0
}

fn is_even_complex(n: i32) -> bool {
    let mut count = 0;
    let abs_n = n.abs();
    for i in 0..=abs_n {
        if i * 2 == abs_n {
            return true;
        }
        count += 1;
        if count > abs_n / 2 + 1 {
            break;
        }
    }
    false
}

// Pattern 9: Macro definitions (similar pattern)
macro_rules! create_function_v1 {
    ($func_name:ident, $val:expr) => {
        fn $func_name() -> i32 {
            $val
        }
    };
}

macro_rules! create_function_v2 {
    ($name:ident, $value:expr) => {
        fn $name() -> i32 {
            $value
        }
    };
}

// Pattern 10: Trait implementations
trait Display {
    fn display(&self) -> String;
}

struct Person {
    name: String,
    age: u32,
}

impl Display for Person {
    fn display(&self) -> String {
        format!("{} is {} years old", self.name, self.age)
    }
}

struct User {
    username: String,
    years: u32,
}

impl Display for User {
    fn display(&self) -> String {
        format!("{} is {} years old", self.username, self.years)
    }
}