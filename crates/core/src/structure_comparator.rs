use std::collections::HashMap;

/// 一般化された構造定義
#[derive(Debug, Clone)]
pub struct Structure {
    /// 識別子（名前、種類、名前空間）
    pub identifier: StructureIdentifier,
    
    /// メンバー（プロパティ、フィールド、メソッドなど）
    pub members: Vec<StructureMember>,
    
    /// メタデータ（位置情報、ジェネリクス、継承など）
    pub metadata: StructureMetadata,
}

#[derive(Debug, Clone)]
pub struct StructureIdentifier {
    pub name: String,
    pub kind: StructureKind,
    pub namespace: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StructureKind {
    TypeScriptInterface,
    TypeScriptTypeAlias,
    TypeScriptTypeLiteral,
    TypeScriptClass,
    RustStruct,
    RustEnum,
    CssRule,
    CssClass,
    Generic(String),
}

#[derive(Debug, Clone)]
pub struct StructureMember {
    pub name: String,
    pub value_type: String,
    pub modifiers: Vec<String>,
    pub nested: Option<Box<Structure>>,
}

#[derive(Debug, Clone, Default)]
pub struct StructureMetadata {
    pub location: SourceLocation,
    pub generics: Vec<String>,
    pub extends: Vec<String>,
    pub visibility: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct SourceLocation {
    pub file_path: String,
    pub start_line: usize,
    pub end_line: usize,
}

/// 構造比較の結果
#[derive(Debug, Clone)]
pub struct StructureComparisonResult {
    pub overall_similarity: f64,
    pub identifier_similarity: f64,
    pub member_similarity: f64,
    pub member_matches: Vec<MemberMatch>,
    pub differences: StructureDifferences,
}

#[derive(Debug, Clone)]
pub struct MemberMatch {
    pub member1: String,
    pub member2: String,
    pub similarity: f64,
}

#[derive(Debug, Clone)]
pub struct StructureDifferences {
    pub missing_members: Vec<String>,
    pub extra_members: Vec<String>,
    pub type_mismatches: Vec<(String, String, String)>, // (name, type1, type2)
}

/// 比較オプション
#[derive(Debug, Clone)]
pub struct ComparisonOptions {
    pub name_weight: f64,
    pub structure_weight: f64,
    pub member_comparison: MemberComparisonStrategy,
    pub ignore_order: bool,
    pub fuzzy_matching: bool,
    pub threshold: f64,
    pub strict_size_check: bool,  // サイズチェックを厳格にする
    pub require_type_match: bool, // 型の一致を要求する
}

impl Default for ComparisonOptions {
    fn default() -> Self {
        Self {
            name_weight: 0.3,
            structure_weight: 0.7,
            member_comparison: MemberComparisonStrategy::Normalized,
            ignore_order: true,
            fuzzy_matching: true,
            threshold: 0.7,
            strict_size_check: true,
            require_type_match: false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum MemberComparisonStrategy {
    Exact,
    Normalized,
    Semantic,
}

/// 汎用構造比較エンジン
pub struct StructureComparator {
    options: ComparisonOptions,
    fingerprint_cache: HashMap<String, String>,
}

impl StructureComparator {
    pub fn new(options: ComparisonOptions) -> Self {
        Self {
            options,
            fingerprint_cache: HashMap::new(),
        }
    }
    
    pub fn compare(&mut self, s1: &Structure, s2: &Structure) -> StructureComparisonResult {
        // 識別子の類似性
        let identifier_similarity = self.compare_identifiers(&s1.identifier, &s2.identifier);
        
        // メンバーの類似性と詳細
        let (member_similarity, member_matches, differences) = 
            self.compare_members(&s1.members, &s2.members);
        
        // メンバー数の違いによるペナルティを計算
        let size_penalty = self.calculate_size_penalty(s1.members.len(), s2.members.len());
        
        // 全体的な類似性を計算（サイズペナルティを適用）
        let base_similarity = 
            self.options.name_weight * identifier_similarity +
            self.options.structure_weight * member_similarity;
        
        let overall_similarity = base_similarity * size_penalty;
        
        StructureComparisonResult {
            overall_similarity,
            identifier_similarity,
            member_similarity,
            member_matches,
            differences,
        }
    }
    
    fn calculate_size_penalty(&self, size1: usize, size2: usize) -> f64 {
        let min_size = size1.min(size2) as f64;
        let max_size = size1.max(size2) as f64;
        
        if max_size == 0.0 {
            return 1.0;
        }
        
        let ratio = min_size / max_size;
        
        if self.options.strict_size_check {
            // 厳格モード: より強いペナルティ
            if ratio < 0.3 {
                // 30%未満: 非常に強いペナルティ
                ratio * ratio * 0.5
            } else if ratio < 0.5 {
                // 30-50%: 強いペナルティ
                ratio * ratio
            } else if ratio < 0.7 {
                // 50-70%: 中程度のペナルティ
                0.4 + (ratio * 0.6)
            } else {
                // 70%以上: 軽いペナルティ
                0.7 + (ratio * 0.3)
            }
        } else {
            // 通常モード: 従来のペナルティ
            if ratio < 0.5 {
                ratio * ratio
            } else {
                0.25 + (ratio * 0.75)
            }
        }
    }
    
    fn compare_identifiers(&self, id1: &StructureIdentifier, id2: &StructureIdentifier) -> f64 {
        // 種類が異なる場合はペナルティ
        let kind_factor = if id1.kind == id2.kind { 1.0 } else { 0.8 };
        
        // 名前の類似性
        let name_similarity = calculate_string_similarity(&id1.name, &id2.name);
        
        name_similarity * kind_factor
    }
    
    fn compare_members(
        &self,
        members1: &[StructureMember],
        members2: &[StructureMember],
    ) -> (f64, Vec<MemberMatch>, StructureDifferences) {
        let mut matches = Vec::new();
        let mut matched_indices1 = vec![false; members1.len()];
        let mut matched_indices2 = vec![false; members2.len()];
        
        // 各メンバーの最良マッチを見つける
        for (i, m1) in members1.iter().enumerate() {
            let mut best_match = None;
            let mut best_score = 0.0;
            
            for (j, m2) in members2.iter().enumerate() {
                if matched_indices2[j] {
                    continue;
                }
                
                let score = self.compare_single_member(m1, m2);
                if score > best_score && score >= self.options.threshold {
                    best_score = score;
                    best_match = Some(j);
                }
            }
            
            if let Some(j) = best_match {
                matched_indices1[i] = true;
                matched_indices2[j] = true;
                matches.push(MemberMatch {
                    member1: m1.name.clone(),
                    member2: members2[j].name.clone(),
                    similarity: best_score,
                });
            }
        }
        
        // 差分を収集
        let missing_members: Vec<String> = members1
            .iter()
            .enumerate()
            .filter(|(i, _)| !matched_indices1[*i])
            .map(|(_, m)| m.name.clone())
            .collect();
        
        let extra_members: Vec<String> = members2
            .iter()
            .enumerate()
            .filter(|(i, _)| !matched_indices2[*i])
            .map(|(_, m)| m.name.clone())
            .collect();
        
        let type_mismatches: Vec<(String, String, String)> = matches
            .iter()
            .filter_map(|m| {
                let m1 = members1.iter().find(|member| member.name == m.member1)?;
                let m2 = members2.iter().find(|member| member.name == m.member2)?;
                if m1.value_type != m2.value_type {
                    Some((m.member1.clone(), m1.value_type.clone(), m2.value_type.clone()))
                } else {
                    None
                }
            })
            .collect();
        
        // 類似性スコアを計算
        // マッチしたメンバー数と最小メンバー数の両方を考慮
        let min_members = members1.len().min(members2.len()) as f64;
        let max_members = members1.len().max(members2.len()) as f64;
        
        let similarity = if max_members > 0.0 {
            // マッチしたメンバーの割合を計算
            let match_ratio = matches.len() as f64 / max_members;
            
            // すべてのメンバーがマッチしているかチェック
            if matches.len() as f64 >= min_members && min_members == max_members {
                // 完全一致
                match_ratio
            } else if matches.len() as f64 >= min_members {
                // 部分一致（追加フィールドあり）
                match_ratio * 0.9
            } else {
                // 不完全な一致
                match_ratio * 0.7
            }
        } else {
            1.0
        };
        
        let differences = StructureDifferences {
            missing_members,
            extra_members,
            type_mismatches,
        };
        
        (similarity, matches, differences)
    }
    
    fn compare_single_member(&self, m1: &StructureMember, m2: &StructureMember) -> f64 {
        let name_sim = calculate_string_similarity(&m1.name, &m2.name);
        
        let type_sim = match self.options.member_comparison {
            MemberComparisonStrategy::Exact => {
                if m1.value_type == m2.value_type { 1.0 } else { 0.0 }
            }
            MemberComparisonStrategy::Normalized => {
                calculate_type_similarity(&m1.value_type, &m2.value_type)
            }
            MemberComparisonStrategy::Semantic => {
                // 意味的な類似性（将来実装）
                calculate_type_similarity(&m1.value_type, &m2.value_type)
            }
        };
        
        // 修飾子の一致度
        let modifier_sim = calculate_modifier_similarity(&m1.modifiers, &m2.modifiers);
        
        // 重み付き平均
        0.4 * name_sim + 0.5 * type_sim + 0.1 * modifier_sim
    }
    
    pub fn generate_fingerprint(&mut self, structure: &Structure) -> String {
        let key = format!("{}::{}", structure.identifier.namespace.as_deref().unwrap_or(""), 
                         structure.identifier.name);
        
        self.fingerprint_cache
            .entry(key)
            .or_insert_with(|| compute_structure_fingerprint(structure))
            .clone()
    }
}

/// 構造のフィンガープリントを計算
pub fn compute_structure_fingerprint(structure: &Structure) -> String {
    let mut parts = Vec::new();
    
    // 種類
    parts.push(format!("kind:{:?}", structure.identifier.kind));
    
    // メンバー数（より細かい分類）
    let member_count = structure.members.len();
    let member_category = match member_count {
        0 => "empty",
        1 => "single",
        2..=3 => "small",
        4..=6 => "medium",
        7..=10 => "large",
        _ => "huge",
    };
    parts.push(format!("size:{}", member_category));
    parts.push(format!("members:{}", member_count));
    
    // 型の分布を計算
    let mut type_counts: HashMap<String, usize> = HashMap::new();
    for member in &structure.members {
        let normalized_type = normalize_type(&member.value_type);
        *type_counts.entry(normalized_type).or_insert(0) += 1;
    }
    
    // ソートして一貫性を保つ
    let mut type_entries: Vec<_> = type_counts.iter().collect();
    type_entries.sort_by_key(|(k, _)| k.as_str());
    
    for (type_name, count) in type_entries {
        parts.push(format!("{}:{}", type_name, count));
    }
    
    // ジェネリクスがあれば追加
    if !structure.metadata.generics.is_empty() {
        parts.push(format!("generics:{}", structure.metadata.generics.len()));
    }
    
    parts.join(",")
}

/// フィンガープリントが比較対象として妥当かチェック
pub fn should_compare_fingerprints(fp1: &str, fp2: &str) -> bool {
    let parts1 = parse_fingerprint(fp1);
    let parts2 = parse_fingerprint(fp2);
    
    // 種類が違う場合は比較しない（TypeScriptInterfaceとRustStructなど）
    if let (Some(kind1), Some(kind2)) = (parts1.get("kind"), parts2.get("kind")) {
        if kind1 != kind2 {
            return false;
        }
    }
    
    // サイズカテゴリが大きく異なる場合は比較しない
    if let (Some(size1), Some(size2)) = (parts1.get("size"), parts2.get("size")) {
        let size_diff = size_category_distance(size1, size2);
        if size_diff > 2 {
            return false;
        }
    }
    
    // メンバー数が大きく異なる場合は比較しない
    if let (Some(members1), Some(members2)) = (parts1.get("members"), parts2.get("members")) {
        if let (Ok(count1), Ok(count2)) = (members1.parse::<usize>(), members2.parse::<usize>()) {
            let min = count1.min(count2);
            let max = count1.max(count2);
            if max > 0 && (min as f64 / max as f64) < 0.3 {
                return false;
            }
        }
    }
    
    true
}

fn parse_fingerprint(fp: &str) -> HashMap<String, String> {
    fp.split(',')
        .filter_map(|part| {
            let mut iter = part.split(':');
            Some((iter.next()?.to_string(), iter.next()?.to_string()))
        })
        .collect()
}

fn size_category_distance(cat1: &str, cat2: &str) -> usize {
    let categories = ["empty", "single", "small", "medium", "large", "huge"];
    let pos1 = categories.iter().position(|&c| c == cat1).unwrap_or(0);
    let pos2 = categories.iter().position(|&c| c == cat2).unwrap_or(0);
    if pos1 > pos2 { pos1 - pos2 } else { pos2 - pos1 }
}

/// 型を正規化
fn normalize_type(type_str: &str) -> String {
    // Check for array patterns first (before checking for the base type)
    if type_str.contains("[]") || type_str.contains("Array") {
        return "array".to_string();
    }
    
    match type_str {
        s if s.contains("string") => "string".to_string(),
        s if s.contains("number") => "number".to_string(),
        s if s.contains("boolean") => "boolean".to_string(),
        s if s.contains("{") && s.contains("}") => "object".to_string(),
        _ => "other".to_string(),
    }
}

/// 文字列の類似性を計算
fn calculate_string_similarity(s1: &str, s2: &str) -> f64 {
    if s1 == s2 {
        return 1.0;
    }
    
    let len1 = s1.len();
    let len2 = s2.len();
    let max_len = len1.max(len2) as f64;
    
    if max_len == 0.0 {
        return 1.0;
    }
    
    // 簡単なレーベンシュタイン距離の近似
    let common_prefix = s1.chars().zip(s2.chars()).take_while(|(a, b)| a == b).count();
    let common_suffix = s1.chars().rev().zip(s2.chars().rev()).take_while(|(a, b)| a == b).count();
    let common = (common_prefix + common_suffix).min(len1.min(len2));
    
    common as f64 / max_len
}

/// 型の類似性を計算
fn calculate_type_similarity(t1: &str, t2: &str) -> f64 {
    if t1 == t2 {
        return 1.0;
    }
    
    let norm1 = normalize_type(t1);
    let norm2 = normalize_type(t2);
    
    if norm1 == norm2 {
        0.8  // 正規化後に一致
    } else {
        0.0
    }
}

/// 修飾子の類似性を計算
fn calculate_modifier_similarity(m1: &[String], m2: &[String]) -> f64 {
    if m1.is_empty() && m2.is_empty() {
        return 1.0;
    }
    
    let set1: HashMap<_, _> = m1.iter().map(|s| (s.as_str(), true)).collect();
    let set2: HashMap<_, _> = m2.iter().map(|s| (s.as_str(), true)).collect();
    
    let intersection = set1.keys().filter(|k| set2.contains_key(*k)).count();
    let union = (set1.len() + set2.len() - intersection).max(1);
    
    intersection as f64 / union as f64
}