# Rule Engine

Rule engine written in Rust for parsing and evaluating rules, with customisable tags and objects to evaluate against.

## Contents

- [Rule Engine](#rule-engine)
  - [Contents](#contents)
- [Rule DSL (Domain-Specific Language)](#rule-dsl-domain-specific-language)
  - [Operators](#operators)
  - [Examples](#examples)
- [Config Files](#config-files)
  - [1. Tags File (`.tags`)](#1-tags-file-tags)
  - [2. Rules File (`.rules`)](#2-rules-file-rules)
  - [3. Objects File (`.yaml`)](#3-objects-file-yaml)
- [Parsing Rules](#parsing-rules)
- [Engine Design](#engine-design)
  - [Step 1: Index and Validate Tags (Parser)](#step-1-index-and-validate-tags-parser)
  - [Step 2: Validate and Convert Rules to Disjunctive Normal Form (DNF) (Parser)](#step-2-validate-and-convert-rules-to-disjunctive-normal-form-dnf-parser)
  - [Step 3: Validate and Build Map of Objects (Parser)](#step-3-validate-and-build-map-of-objects-parser)
  - [Step 4: Match Objects Against Rules (Engine)](#step-4-match-objects-against-rules-engine)
  - [Step 5: Determine Match Result (Engine)](#step-5-determine-match-result-engine)

---

> ### **\*Note**: Easy to understand config files are already present if you'd like to get started right away.\*

---

# Rule DSL (Domain-Specific Language)

A simple, concise syntax for writing matching rules.

## Operators

- `=` - equals
- `!` - not equals
- `&` - logical AND
- `|` - logical OR
- `()` - grouping for precedence
- `,` - shorthand for OR within the same field (e.g. `color=red | color=blue` becomes `color=red,blue`)

## Examples

**Simple equality:**

```
color=red
```

**AND condition:**

```
color=red & size=large
```

**OR condition:**

```
color=red | color=blue
```

**OR shorthand (comma):**

```
color=red,blue
```

Equivalent to: `color=red | color=blue`

**NOT condition:**

```
color!red
```

**Complex grouping:**

```
(color=red,blue) & size!small
```

Matches: color is red OR blue, AND size is NOT small

**Nested logic:**

```
status=active & (priority=high | type=urgent)
```

Matches: status is active AND (priority is high OR type is urgent)

**Multiple field conditions:**

```
(type=admin,moderator) & status=active & role!guest
```

Matches: type is admin OR moderator, AND status is active, AND role is NOT guest

---

# Config Files

The rules engine uses three configuration files in the `config/` directory:

## 1. Tags File (`.tags`)

Defines available tags (fields) and their possible values.

**File:** `config/my_tags.tags`

```
- Colour: Blue, Green, Red
- Shape: Circle, Rectangle, Square
- Size: Small, Medium, Large
```

## 2. Rules File (`.rules`)

Contains the actual matching rules written in the DSL syntax.

**File:** `config/my_rules.rules`

```
- (colour=blue,red) & shape!circle
- (colour=green) | shape=rectangle
```

## 3. Objects File (`.yaml`)

Contains objects to be evaluated against the rules. Objects are grouped by type for flexibility.

**File:** `config/my_objects.yaml`

```yaml
objects:
  shapes:
    - colour: [red, green]
      shape: rectangle
      size: large

    - colour: green
      shape: circle
      size: small

  cars:
    - colour: grey
      size: small
      doors: 3

    - colour: black
      size: large
      doors: 5
```

**Adding new object types:**

Simply add a new key under `objects` with a list of items. Each type can have completely different attributes:

```yaml
objects:
  your_type_name:
    - attribute1: value1
      attribute2: value2

    - attribute1: value3
      attribute2: value4
```

The type name (e.g., `shapes`, `cars`) is automatically assigned to each object in that group.

---

# Parsing Rules

- **Comments:** Use `#` for comments in all config files
- **Case-insensitive:** All parsing is case-insensitive
- **No quotes:** Values don't require quotes
- **Spaces:** Optional and ignored in rules

---

# Engine Design

The matching engine uses a DNF-based approach for efficient rule evaluation.

## Step 1: Index and Validate Tags (Parser)

Parse the tags file and build an index of all available tags and their valid values. Validate the format and ensure each tag has a unique name and at least one value.

## Step 2: Validate and Convert Rules to Disjunctive Normal Form (DNF) (Parser)

Parse each rule, validate syntax, convert to OR-of-ANDs format (each AND group is a "subrule"), build subrule objects, and create tag-to-subrule maps. For each subrule, track the expected clause count, actual match count (initialized to 0), comparison operators, and tag key-value pairs.

**Example:**

```
Original: (colour=blue,red) & shape!circle
DNF:      (colour=blue & shape!circle) | (colour=red & shape!circle)
          └────────── SR1 ───────────┘   └────────── SR2 ──────────┘
```

```
Subrule Objects:
SR1: {
  expected_count: 2,
  actual_count: 0,
  comparison_ops: [ISEQ, NOEQ],
  tag_kvs: {"colour": "blue", "shape": "circle"}
}
SR2: {
  expected_count: 2,
  actual_count: 0,
  comparison_ops: [ISEQ, NOEQ],
  tag_kvs: {"colour": "red", "shape": "circle"}
}

Tag-to-Subrule Maps:
colour map:
  "blue" → [SR1]
  "red"  → [SR2]

shape map:
  "circle" → [SR1, SR2]  (appears in both with NOEQ operator)
```

## Step 3: Validate and Build Map of Objects (Parser)

Parse the objects YAML file and build a map of all objects to evaluate. Validate that each object has valid structure and assign object types based on their grouping in the YAML file.

## Step 4: Match Objects Against Rules (Engine)

For each object, check which clauses match and increment the `actual_count` for matching subrules.

**Example object:**

```yaml
colour: blue
shape: square
size: large
```

Matching process:

- `colour=blue` matches → increment `SR1.actual_count` to 1
- `shape!circle` matches (square ≠ circle) → increment `SR1.actual_count` to 2
- `shape!circle` matches → increment `SR2.actual_count` to 1
- `colour=red` doesn't match → `SR2.actual_count` stays at 1

## Step 5: Determine Match Result (Engine)

A rule matches if **any subrule** has `actual_count == expected_count`.

**Example:**

```
SR1: actual_count = 2, expected_count = 2 → MATCH ✓
SR2: actual_count = 1, expected_count = 2 → no match

Result: MATCH
```
