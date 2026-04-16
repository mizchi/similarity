use criterion::{black_box, criterion_group, criterion_main, Criterion};
use similarity_core::language_parser::LanguageParser;
use similarity_core::tsed::{calculate_tsed, calculate_tsed_with_threshold, TSEDOptions};
use similarity_mbt::moonbit_parser::MoonBitParser;

const SMALL_SOURCE: &str = r#"
fn add(x : Int, y : Int) -> Int {
  x + y
}

fn multiply(x : Int, y : Int) -> Int {
  x * y
}
"#;

const MEDIUM_SOURCE: &str = r#"
pub fn process_items(items : Array[Int]) -> Array[Int] {
  let result : Array[Int] = []
  let mut i = 0
  while i < items.length() {
    if items[i] > 0 {
      result.push(items[i] * 2)
    }
    i = i + 1
  }
  result
}

pub fn handle_items(data : Array[Int]) -> Array[Int] {
  let output : Array[Int] = []
  let mut j = 0
  while j < data.length() {
    if data[j] > 0 {
      output.push(data[j] * 2)
    }
    j = j + 1
  }
  output
}

pub fn transform_data(input : Array[Int]) -> Array[Int] {
  let out : Array[Int] = []
  let mut k = 0
  while k < input.length() {
    if input[k] > 0 {
      out.push(input[k] * 2)
    }
    k = k + 1
  }
  out
}

pub fn filter_positive(arr : Array[Int]) -> Array[Int] {
  let filtered : Array[Int] = []
  let mut idx = 0
  while idx < arr.length() {
    if arr[idx] > 0 {
      filtered.push(arr[idx])
    }
    idx = idx + 1
  }
  filtered
}
"#;

fn generate_large_source(num_functions: usize) -> String {
    let mut source = String::new();
    for i in 0..num_functions {
        source.push_str(&format!(
            r#"
fn func_{i}(x : Int, y : Int) -> Int {{
  let a = x + {i}
  let b = y * {mult}
  let c = a + b
  if c > 0 {{
    c * 2
  }} else {{
    c + 1
  }}
}}
"#,
            i = i,
            mult = i + 1
        ));
    }
    source
}

fn bench_parser_creation(c: &mut Criterion) {
    c.bench_function("parser_creation", |b| {
        b.iter(|| {
            let parser = MoonBitParser::new().unwrap();
            black_box(parser);
        });
    });
}

fn bench_parse_small(c: &mut Criterion) {
    let mut parser = MoonBitParser::new().unwrap();
    c.bench_function("parse_small", |b| {
        b.iter(|| {
            let tree = parser.parse(black_box(SMALL_SOURCE), "test.mbt").unwrap();
            black_box(tree);
        });
    });
}

fn bench_parse_medium(c: &mut Criterion) {
    let mut parser = MoonBitParser::new().unwrap();
    c.bench_function("parse_medium", |b| {
        b.iter(|| {
            let tree = parser.parse(black_box(MEDIUM_SOURCE), "test.mbt").unwrap();
            black_box(tree);
        });
    });
}

fn bench_extract_functions_medium(c: &mut Criterion) {
    let mut parser = MoonBitParser::new().unwrap();
    c.bench_function("extract_functions_medium", |b| {
        b.iter(|| {
            let funcs = parser.extract_functions(black_box(MEDIUM_SOURCE), "test.mbt").unwrap();
            black_box(funcs);
        });
    });
}

fn bench_tsed_comparison(c: &mut Criterion) {
    let mut parser = MoonBitParser::new().unwrap();

    let body1 = r#"{
  let result : Array[Int] = []
  let mut i = 0
  while i < items.length() {
    if items[i] > 0 {
      result.push(items[i] * 2)
    }
    i = i + 1
  }
  result
}"#;

    let body2 = r#"{
  let output : Array[Int] = []
  let mut j = 0
  while j < data.length() {
    if data[j] > 0 {
      output.push(data[j] * 2)
    }
    j = j + 1
  }
  output
}"#;

    let tree1 = parser.parse(body1, "body1.mbt").unwrap();
    let tree2 = parser.parse(body2, "body2.mbt").unwrap();
    let options = TSEDOptions::default();

    c.bench_function("tsed_comparison", |b| {
        b.iter(|| {
            let sim = calculate_tsed(black_box(&tree1), black_box(&tree2), &options);
            black_box(sim);
        });
    });
}

fn bench_full_pipeline_10_functions(c: &mut Criterion) {
    let source = generate_large_source(10);
    let options = TSEDOptions::default();

    c.bench_function("full_pipeline_10_funcs", |b| {
        b.iter(|| {
            let mut parser = MoonBitParser::new().unwrap();
            let functions = parser.extract_functions(black_box(&source), "test.mbt").unwrap();

            let lines: Vec<&str> = source.lines().collect();

            // Pre-parse all function bodies once
            let trees: Vec<_> = functions
                .iter()
                .map(|func| {
                    let start = (func.body_start_line.saturating_sub(1)) as usize;
                    let end = std::cmp::min(func.body_end_line as usize, lines.len());
                    let body = lines[start..end].join("\n");
                    parser.parse(&body, "body.mbt").ok()
                })
                .collect();

            let mut count = 0;
            for i in 0..functions.len() {
                for j in (i + 1)..functions.len() {
                    if let (Some(tree1), Some(tree2)) = (trees[i].as_ref(), trees[j].as_ref()) {
                        let sim = calculate_tsed_with_threshold(tree1, tree2, &options, 0.85);
                        black_box(sim);
                        count += 1;
                    }
                }
            }
            black_box(count);
        });
    });
}

fn bench_full_pipeline_20_functions(c: &mut Criterion) {
    let source = generate_large_source(20);
    let options = TSEDOptions::default();

    c.bench_function("full_pipeline_20_funcs", |b| {
        b.iter(|| {
            let mut parser = MoonBitParser::new().unwrap();
            let functions = parser.extract_functions(black_box(&source), "test.mbt").unwrap();

            let lines: Vec<&str> = source.lines().collect();

            // Pre-parse all function bodies once
            let trees: Vec<_> = functions
                .iter()
                .map(|func| {
                    let start = (func.body_start_line.saturating_sub(1)) as usize;
                    let end = std::cmp::min(func.body_end_line as usize, lines.len());
                    let body = lines[start..end].join("\n");
                    parser.parse(&body, "body.mbt").ok()
                })
                .collect();

            let mut count = 0;
            for i in 0..functions.len() {
                for j in (i + 1)..functions.len() {
                    if let (Some(tree1), Some(tree2)) = (trees[i].as_ref(), trees[j].as_ref()) {
                        let sim = calculate_tsed_with_threshold(tree1, tree2, &options, 0.85);
                        black_box(sim);
                        count += 1;
                    }
                }
            }
            black_box(count);
        });
    });
}

criterion_group!(
    benches,
    bench_parser_creation,
    bench_parse_small,
    bench_parse_medium,
    bench_extract_functions_medium,
    bench_tsed_comparison,
    bench_full_pipeline_10_functions,
    bench_full_pipeline_20_functions,
);
criterion_main!(benches);
