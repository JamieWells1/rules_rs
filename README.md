# Rules

A comprehensive rules engine for evaluating rules against items.

## Rule DSL (Domain-Specific Language)

A simple, concise syntax for writing matching rules.

### Operators

- `=` - equals
- `!` - not equals
- `&` - logical AND
- `|` - logical OR
- `()` - grouping for precedence
- `,` - shorthand for OR within the same field (e.g. `color=red | color=blue` becomes `color=red,blue`)

### Syntax Rules

- No quotes needed around values
- Spaces are optional and ignored
- Parentheses control evaluation order

### Examples

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

## Config Files

The rules engine uses three configuration files in the `config/` directory:

### 1. Tags File (`.tags`)

Defines available tags (fields) and their possible values.

**File:** `config/my_tags.tags`
```
# Define the tags and their possible values to be used in rules

- Colour: Blue, Green, Red
- Shape: Circle, Rectangle, Square
- Size: Small, Medium, Large
```

### 2. Rules File (`.rules`)

Contains the actual matching rules written in the DSL syntax.

**File:** `config/my_rules.rules`
```
# Define rules

- (colour=blue,red) & shape!circle
- (colour=green) | shape=rectangle
```

### 3. Objects File (`.yaml`)

Contains items to be evaluated against the rules.

**File:** `config/objects.yaml`
```yaml
# Objects to be evaluated against defined rules

items:
  - colour: [red, green]
    shape: rectangle
    size: large

  - colour: green
    shape: circle
    size: small
```

## Parsing Rules

- **Comments:** Use `#` for comments in all config files
- **Case-insensitive:** All parsing is case-insensitive
- **No quotes:** Values don't require quotes
- **Spaces:** Optional and ignored in rules

---

## Engine Design

1. Parse each rule into disjunct normal form (DNF)
   - E.g. (ccy == "USD" || ccy == "GBP") && assCl == "BOND" --> (ccy == "USD" && assCl == "BOND") || (ccy == "GBP" && assCl == "BOND")
2. Iterate through each subrule and add mapped subrule to map of subrules, which contains how many times the subrule is present, and the operator used in the subrule
   - E.g. { "SR1": { "expected_count": 2, "actual_count": 2 (initialised at 0), "operator": "eq" } }
3. Iterate through each subrule (each DNF rule separated by the OR operator) and create a map for each tag found in subrules
   - E.g. ccy map, where key = ccy (e.g. "USD") and value is which subrules it appears in --> ccy map: { "USD": ["SR1"], "GBP": ["SR2"] }
4. Check operator and tag against item being matched against
   - E.g. { "assCl": "EQTY", "ccy": "USD", "tkr": "AAPL" } --> if operator + tag corresponds with item, add to map. Else, don't add
5. Check number of expected sub-rule counts with number of actual sub-rule counts, if equal, match the item with the rules. Else, item doesn't match
