#![allow(clippy::uninlined_format_args)]

pub mod apted;
pub mod ast_exchange;
pub mod ast_fingerprint;
pub mod class_comparator;
pub mod class_extractor;
pub mod enhanced_similarity;
pub mod fast_similarity;
pub mod function_extractor;
pub mod generic_overlap_detector;
pub mod generic_parser_config;
pub mod generic_tree_sitter_parser;
pub mod language_parser;
pub mod overlap_detector;
pub mod parser;
pub mod subtree_fingerprint;
pub mod tree;
pub mod tsed;
pub mod type_comparator;
pub mod type_extractor;
pub mod type_fingerprint;
pub mod type_normalizer;
pub mod unified_type_comparator;

// CLI utilities
pub mod cli_file_utils;
pub mod cli_output;
pub mod cli_parallel;

pub use apted::{compute_edit_distance, APTEDOptions};
pub use enhanced_similarity::{
    calculate_enhanced_similarity, calculate_semantic_similarity, EnhancedSimilarityOptions,
};
pub use function_extractor::{
    compare_functions, extract_functions, find_similar_functions_across_files,
    find_similar_functions_in_file, FunctionDefinition, FunctionType, SimilarityResult,
};
pub use parser::{ast_to_tree_node, parse_and_convert_to_tree};
pub use tree::TreeNode;
pub use tsed::{calculate_tsed, calculate_tsed_from_code, TSEDOptions};

// Type-related exports
pub use type_comparator::{
    compare_type_literal_with_type, compare_types, find_duplicate_types,
    find_similar_type_literals, find_similar_type_literals_pairs, find_similar_types,
    group_similar_types, MatchedProperty, SimilarTypePair, TypeComparisonOptions,
    TypeComparisonResult, TypeDifferences, TypeLiteralComparisonPair, TypeMismatch,
};
pub use type_extractor::{
    extract_type_literals_from_code, extract_type_literals_from_files, extract_types_from_code,
    extract_types_from_files, PropertyDefinition, TypeDefinition, TypeKind, TypeLiteralContext,
    TypeLiteralDefinition,
};
pub use type_normalizer::{
    calculate_property_similarity, calculate_type_similarity, find_property_matches,
    normalize_type, NormalizationOptions, NormalizedType, PropertyMatch,
};
pub use unified_type_comparator::{
    find_similar_unified_types, UnifiedType, UnifiedTypeComparisonPair,
};

// Fast similarity exports
pub use ast_fingerprint::AstFingerprint;
pub use fast_similarity::{
    find_similar_functions_across_files_fast, find_similar_functions_fast, FastSimilarityOptions,
};

// Subtree fingerprint exports
pub use subtree_fingerprint::{
    create_sliding_windows, detect_partial_overlaps, generate_subtree_fingerprints,
    IndexedFunction, OverlapOptions, PartialOverlap, SubtreeFingerprint,
};

// Overlap detector exports
pub use overlap_detector::{
    find_function_overlaps, find_overlaps_across_files, find_overlaps_with_similarity,
    DetailedOverlap, PartialOverlapWithFiles,
};

// Generic overlap detector exports
pub use generic_overlap_detector::{
    find_function_overlaps_generic, find_overlaps_across_files_generic,
    find_overlaps_with_similarity_generic, DetailedOverlap as GenericDetailedOverlap,
    PartialOverlapWithFiles as GenericPartialOverlapWithFiles,
};

// Class-related exports
pub use class_comparator::{
    compare_classes, find_similar_classes, find_similar_classes_across_files, normalize_class,
    ClassComparisonResult, ClassDifferences, MethodMismatch, NormalizedClass, PropertyMismatch,
    SimilarClassPair,
};
pub use class_extractor::{
    extract_classes_from_code, extract_classes_from_files, ClassDefinition, ClassMethod,
    ClassProperty, MethodKind,
};
