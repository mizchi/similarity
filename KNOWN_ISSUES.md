# Known Issues

## Rust Type Similarity Detection

### Enum Similarity Detection
- **Issue**: Enum similarity detection shows lower than expected similarity scores even for structurally identical enums
- **Example**: Two enums with identical variants show only ~43% similarity
- **Cause**: The AST structure for enums includes variant names as values, and the current rename_cost parameter doesn't adequately handle this case
- **Workaround**: Use a lower threshold (0.4-0.5) for enum similarity detection
- **Status**: Under investigation

### Struct Similarity Detection
- **Status**: Working as expected
- Structs with similar field types but different field names correctly show high similarity (90%+)
- Generic structs are properly compared

## TypeScript Type Similarity Detection
- **Status**: Working as expected
- Interfaces, type aliases, and type literals are correctly detected with appropriate similarity scores