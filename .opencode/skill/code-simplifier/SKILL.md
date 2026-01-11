---
name: code-simplifier
description: Analyze and simplify code by extracting reusable functions, removing redundancies, and improving overall code quality through refactoring techniques
license: MIT
compatibility: opencode
metadata:
  audience: developers
  workflow: code-refactoring
---

## What I do

I analyze code and automatically simplify it by:
- Extracting repetitive code patterns into reusable functions
- Removing redundant code, dead code, and unnecessary complexity
- Applying idiomatic language patterns and best practices
- Optimizing control flow and data structures
- Improving readability while preserving functionality

## When to use me

Use this skill when you need to:
- Clean up legacy code with high complexity
- Extract common patterns into helper functions
- Reduce code duplication across files
- Modernize code to follow current best practices
- Simplify nested conditionals and loops
- Improve variable naming and code structure

## My workflow

1. **Code Analysis**
   - Parse the code into abstract syntax tree (AST)
   - Identify code clones and repeated patterns
   - Measure cyclomatic complexity
   - Detect dead code and unreachable branches
   - Find anti-patterns and suboptimal constructs

2. **Simplification Strategies**
   - Extract methods/functions from duplicated code
   - Replace nested conditionals with guard clauses
   - Simplify boolean expressions
   - Merge similar switch/if branches
   - Convert loops to iterator methods where applicable
   - Remove unused imports, variables, and functions

3. **Refactoring Execution**
   - Apply changes incrementally, preserving git history
   - Maintain function signatures and behavior
   - Add minimal, focused unit tests for extracted functions
   - Update documentation for changed public APIs

4. **Validation**
   - Ensure all existing tests still pass
   - Verify no semantic changes were introduced
   - Check that complexity metrics improved

## Usage

```bash
/simplify --file src/utils/helper.ts
/simplify --dir src/components --recursive
/simplify --pattern "**/*.java" --dry-run
/simplify --aggressive --max-complexity 10
```

### Options

| Option | Description |
|--------|-------------|
| `--file <path>` | Simplify a single file |
| `--dir <path>` | Simplify all files in a directory |
| `--recursive` | Include subdirectories (use with --dir) |
| `--pattern <glob>` | Files matching glob pattern |
| `--dry-run` | Show changes without applying them |
| `--aggressive` | Apply more aggressive simplifications |
| `--max-complexity <n>` | Target complexity threshold |
| `--extract-methods` | Extract repeated code into methods |
| `--remove-dead-code` | Remove unreachable code and unused items |
| `--fix-idioms` | Apply language-specific idiomatic patterns |

## What I won't do

- Change public API signatures without explicit approval
- Modify code that doesn't have test coverage
- Introduce behavioral changes (only refactoring)
- Remove code comments (only unused imports/variables)
- Force a specific coding style beyond simplification

## Simplification Examples

### 1. Code Extraction

**Before:**
```typescript
// Duplicate code in multiple places
const calculateTax = (amount: number) => amount * 0.08;
const calculateShipping = (weight: number) => weight * 0.5;
const calculateDiscount = (price: number) => price * 0.1;

// Later in the file...
const calculateTax = (amount: number) => amount * 0.08;
const calculateShipping = (weight: number) => weight * 0.5;
const calculateDiscount = (price: number) => price * 0.1;
```

**After:**
```typescript
const calculateTax = (amount: number) => amount * 0.08;
const calculateShipping = (weight: number) => weight * 0.5;
const calculateDiscount = (price: number) => price * 0.1;

// Used where needed
```

### 2. Guard Clauses

**Before:**
```typescript
function processUser(user: User): Result {
    if (user !== null) {
        if (user.isActive) {
            if (user.hasPermission) {
                // Main logic
                return performAction(user);
            }
        }
    }
    return defaultResult;
}
```

**After:**
```typescript
function processUser(user: User): Result {
    if (!user || !user.isActive || !user.hasPermission) {
        return defaultResult;
    }
    return performAction(user);
}
```

### 3. Boolean Simplification

**Before:**
```typescript
const isValid = (status: string) => {
    if (status === 'active' || status === 'pending') {
        return true;
    } else {
        return false;
    }
};
```

**After:**
```typescript
const isValid = (status: string) => status === 'active' || status === 'pending';
```

### 4. Loop to Iterator

**Before:**
```typescript
const names: string[] = [];
for (let i = 0; i < users.length; i++) {
    names.push(users[i].name);
}
```

**After:**
```typescript
const names = users.map(user => user.name);
```

### 5. Optional Chaining & Nullish Coalescing

**Before:**
```typescript
const city = user && user.address && user.address.city !== null
    ? user.address.city
    : 'Unknown';
```

**After:**
```typescript
const city = user?.address?.city ?? 'Unknown';
```

---

## Complexity Metrics

| Metric | Before | After | Target |
|--------|--------|-------|--------|
| Cyclomatic Complexity | 15 | 8 | ≤ 10 |
| Lines of Code | 450 | 320 | - |
| Code Duplication | 12% | 0% | 0% |
| Nesting Depth | 5 | 2 | ≤ 3 |

## Best Practices

1. **Extract early, extract often**: Don't wait for duplication to grow
2. **Single Responsibility**: Each function should do one thing well
3. **Small functions**: Aim for < 20 lines per function
4. **Descriptive names**: `calculateTotalWithTax()` > `process()`
5. **Avoid side effects**: Pure functions are easier to test and reason about

## Limitations

- Cannot simplify code without clear structure (e.g., generated code)
- May require human review for domain-specific optimizations
- Some patterns are intentionally complex for performance reasons
- Tests must exist for behavioral verification

## Output Format

### Summary Report

```
# Code Simplification Report

## Changes Applied

### Extracted Methods
- `extractUserData()` from 3 locations
- `validateInput()` from 2 locations

### Removed Dead Code
- Unused function: `legacyProcess()`
- Unreachable branch in `handleEvent()`

### Simplified Conditionals
- Replaced nested ifs with guard clause (5 occurrences)
- Simplified boolean expressions (3 occurrences)

## Metrics Improvement

| Metric | Before | After | Change |
|--------|--------|-------|--------|
| LOC | 1,250 | 980 | -22% |
| Complexity | 18 avg | 11 avg | -39% |
| Functions | 45 | 52 | +16% |

## Files Modified

- src/utils/helper.ts (3 changes)
- src/services/user.ts (2 changes)
- src/components/form.tsx (1 change)
```

---

## Interaction style

- Provide clear before/after code comparisons
- Explain the rationale for each simplification
- Suggest additional manual improvements when applicable
- Warn about potential risks before aggressive changes
- Preserve code comments and documentation
- Group related changes into logical commits

## Quick Reference

| Pattern | Simplification |
|---------|----------------|
| Duplicate code | Extract to function |
| Nested conditionals | Guard clauses |
| Long boolean expressions | Simplify with early returns |
| Manual iteration | Array methods (map/filter/reduce) |
| Null checks | Optional chaining |
| Default values | Nullish coalescing |
| Switch with 2 cases | Ternary operator |
| Unused code | Remove entirely |
