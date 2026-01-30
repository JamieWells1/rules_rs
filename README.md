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
  - [Step 1: Convert to Disjunctive Normal Form (DNF)](#step-1-convert-to-disjunctive-normal-form-dnf)
  - [Step 2: Build Subrule Metadata](#step-2-build-subrule-metadata)
  - [Step 3: Create Tag-to-Subrule Maps](#step-3-create-tag-to-subrule-maps)
  - [Step 4: Match Objects Against Rules](#step-4-match-objects-against-rules)
  - [Step 5: Determine Match Result](#step-5-determine-match-result)

---

> **Note**: Easy to understand, correctly configured config files are already present if you'd like to get started right away.

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

**File:** `config/objects.yaml`

```yaml
objects:
  shapes:
    - colour: [red, green]
    - shape: rectangle
    - size: large

    - colour: green
    - shape: circle
    - size: small

  cars:
    - colour: grey
    - size: small
    - doors: 3

    - colour: black
    - size: large
    - doors: 5
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

## Step 1: Convert to Disjunctive Normal Form (DNF)

Convert each rule into OR-of-ANDs format (each AND group is a "subrule").

**Example:**

```
Original: (colour=blue,red) & shape!circle
DNF:      (colour=blue & shape!circle) | (colour=red & shape!circle)
          └────────── SR1 ───────────┘   └────────── SR2 ──────────┘
```

## Step 2: Build Subrule Metadata

For each subrule, track how many clauses it contains and what operators are used.

**Example:**

```
SR1: { expected_count: 2, actual_count: 0, operators: [ISEQ, NOEQ] }
SR2: { expected_count: 2, actual_count: 0, clauses: [ISEQ, ISEQ] }
```

## Step 3: Create Tag-to-Subrule Maps

For each tag, map its values to the subrules where they appear.

**Example:**

```
colour map:
  "blue" → [SR1]
  "red"  → [SR2]

shape map:
  "circle" → [SR1, SR2]  (appears in both with ! operator)
```

## Step 4: Match Objects Against Rules

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

## Step 5: Determine Match Result

A rule matches if **any subrule** has `actual_count == expected_count`.

**Example:**

```
SR1: actual_count = 2, expected_count = 2 → MATCH ✓
SR2: actual_count = 1, expected_count = 2 → no match

Overall: MATCH (at least one subrule matched)
```
