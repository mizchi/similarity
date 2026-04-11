use rayon::prelude::*;
use similarity_core::{
    extract_functions, find_similar_functions_fast, find_similar_functions_in_file,
    FastSimilarityOptions, FunctionDefinition, SimilarityResult, TSEDOptions,
};
use std::fs;
use std::path::PathBuf;

/// File with its content and extracted functions
#[derive(Debug)]
pub struct FileData {
    pub path: PathBuf,
    pub content: String,
    pub functions: Vec<FunctionDefinition>,
}

/// Load and parse files in parallel
pub fn load_files_parallel(files: &[PathBuf]) -> Vec<FileData> {
    files
        .par_iter()
        .filter_map(|file| {
            match fs::read_to_string(file) {
                Ok(content) => {
                    let filename = file.to_string_lossy();
                    // Extract functions, skip if parse error
                    match extract_functions(&filename, &content) {
                        Ok(mut functions) => {
                            functions.retain(|function| !function.has_ignore_directive);
                            Some(FileData { path: file.clone(), content, functions })
                        }
                        Err(_) => None, // Skip files with parse errors
                    }
                }
                Err(e) => {
                    eprintln!("Error reading {}: {}", file.display(), e);
                    None
                }
            }
        })
        .collect()
}

/// Check for duplicates within files in parallel
pub fn check_within_file_duplicates_parallel(
    files: &[PathBuf],
    threshold: f64,
    options: &TSEDOptions,
    fast_mode: bool,
) -> Vec<(PathBuf, Vec<SimilarityResult>)> {
    files
        .par_iter()
        .filter_map(|file| match fs::read_to_string(file) {
            Ok(code) => {
                let file_str = file.to_string_lossy();

                let similar_pairs = if fast_mode {
                    let fast_options = FastSimilarityOptions {
                        fingerprint_threshold: 0.3,
                        similarity_threshold: threshold,
                        tsed_options: options.clone(),
                        debug_stats: false,
                    };
                    find_similar_functions_fast(&file_str, &code, &fast_options).ok()
                } else {
                    find_similar_functions_in_file(&file_str, &code, threshold, options).ok()
                };

                similar_pairs.and_then(|pairs| {
                    if pairs.is_empty() {
                        None
                    } else {
                        Some((file.clone(), pairs))
                    }
                })
            }
            Err(_) => None,
        })
        .collect()
}

/// Check for duplicates across files using parallel processing
pub fn check_cross_file_duplicates_parallel(
    file_data: &[FileData],
    threshold: f64,
    options: &TSEDOptions,
    _fast_mode: bool,
) -> Vec<(String, SimilarityResult, String)> {
    // Prepare all function pairs with file information
    let mut all_functions = Vec::new();
    for data in file_data {
        let filename = data.path.to_string_lossy().to_string();
        for func in &data.functions {
            all_functions.push((filename.clone(), data.content.clone(), func.clone()));
        }
    }

    // Generate all cross-file pairs
    let mut pairs_to_check = Vec::new();
    for i in 0..all_functions.len() {
        for j in (i + 1)..all_functions.len() {
            let (file1, _, _) = &all_functions[i];
            let (file2, _, _) = &all_functions[j];

            // Only check across different files
            if file1 != file2 {
                pairs_to_check.push((i, j));
            }
        }
    }

    // Process pairs in parallel
    pairs_to_check
        .into_par_iter()
        .filter_map(|(i, j)| {
            let (file1, content1, func1) = &all_functions[i];
            let (file2, content2, func2) = &all_functions[j];

            // Use core's compare_functions
            match similarity_core::compare_functions(func1, func2, content1, content2, options) {
                Ok(similarity) => {
                    if similarity >= threshold {
                        Some((
                            file1.clone(),
                            SimilarityResult::new(func1.clone(), func2.clone(), similarity),
                            file2.clone(),
                        ))
                    } else {
                        None
                    }
                }
                Err(_) => None,
            }
        })
        .collect()
}
