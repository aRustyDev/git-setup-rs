# Planning Guidelines for Rigorous TDD Enforcement

## Purpose
This document provides strict guidelines for creating PLAN and PROMPT documents that force agents to follow Test-Driven Development rigorously and prevent code quality regression.

---

## Core Principles

### 1. NO ESCAPE HATCHES
- Never use phrases like "for now", "minimal", "temporary", or "placeholder"
- Every implementation must be production-ready
- No "quick and dirty" allowed

### 2. EXPLICIT VERIFICATION GATES
Every task must have mandatory checkpoints:
```
CHECKPOINT 1: Tests Written
- Run: `cargo test --lib [module] 2>&1 | grep "error"`
- Expected: Compilation errors for missing implementations
- If tests pass: STOP - You wrote implementation first!

CHECKPOINT 2: Tests Compile but Fail  
- Run: `cargo test --lib [module]`
- Expected: All tests compile but fail
- Paste the failure output here: ___________

CHECKPOINT 3: Implementation Makes Tests Pass
- Write ONLY enough code to pass tests
- Run: `cargo test --lib [module]`
- Expected: All tests pass
- Paste the success output here: ___________
```

### 3. CONCRETE DELIVERABLES
Replace vague goals with specific outputs:
- ❌ "Implement ProfileManager with good test coverage"  
- ✅ "Create ProfileManager with exactly 12 tests that achieve 95% coverage"

### 4. FAILURE TRIGGERS
Define explicit "STOP AND ASK" conditions:
```
IF ANY OF THESE OCCUR, STOP IMMEDIATELY:
- Tests pass on first run (you wrote implementation first)
- Compilation has more than 5 warnings
- Coverage drops below 90%
- You're tempted to write code without a test
- You think "I'll add tests later"
```

---

## Task Structure Template

### 1. OBJECTIVE (Specific and Measurable)
```
Create [COMPONENT] with:
- Exactly [N] unit tests
- Minimum [X]% code coverage
- Zero clippy warnings
- All tests follow AAA pattern (Arrange, Act, Assert)
```

### 2. PRE-TASK VERIFICATION
```
BEFORE STARTING, RUN THESE COMMANDS:
1. `git status` - Ensure clean working directory
2. `cargo test` - Ensure existing tests pass
3. `just lint` - Ensure no existing warnings

PASTE OUTPUT HERE:
```
[Output required]
```
```

### 3. TDD CYCLE (Explicit Steps)
```
CYCLE 1: [Feature Name]

Step 1.1 - Write Failing Test
- Create test: test_[specific_behavior]
- Test must compile but fail
- Run: `cargo test test_[specific_behavior]`
- PASTE FAILURE OUTPUT: ________

Step 1.2 - Write Minimal Implementation  
- Add ONLY code needed to pass test
- No extra methods or fields
- Run: `cargo test test_[specific_behavior]`
- PASTE SUCCESS OUTPUT: ________

Step 1.3 - Refactor (if needed)
- Improve code without changing behavior
- Run all tests: `cargo test --lib [module]`
- PASTE OUTPUT: ________
```

### 4. INCREMENTAL PROGRESS TRACKING
```
Progress Tracker:
□ Test 1: test_create_empty - Written [ ] Failing [ ] Passing [ ]
□ Test 2: test_create_duplicate - Written [ ] Failing [ ] Passing [ ]
□ Test 3: test_validation - Written [ ] Failing [ ] Passing [ ]
[Continue for all tests...]

Coverage after each test:
- After Test 1: ____%
- After Test 2: ____%
- After Test 3: ____%
```

### 5. MANDATORY QUALITY CHECKS
```
AFTER EVERY 3 TESTS, RUN:
1. `cargo fmt --check` - MUST PASS
2. `cargo clippy -- -D warnings` - MUST PASS
3. `cargo test` - ALL MUST PASS
4. `cargo tarpaulin --lib` - MUST SHOW >90%

IF ANY FAIL: Fix immediately before continuing
```

---

## Anti-Patterns to Avoid

### 1. Batching Tests
❌ "Write all tests first, then implement"
✅ "Write one test, make it pass, repeat"

### 2. Vague Instructions  
❌ "Implement proper error handling"
✅ "Add error handling that returns GitSetupError::InvalidProfile when name contains '/'"

### 3. Optional Steps
❌ "Consider adding validation"
✅ "MUST add validation that rejects empty names"

### 4. Implicit Assumptions
❌ "Follow Rust best practices"
✅ "Use Result<T, GitSetupError> for all fallible operations"

---

## Prompt Engineering Rules

### 1. Use Imperative Commands
- "CREATE test_profile_validation that checks..."
- "RUN cargo test and PASTE output"
- "STOP if any test passes without implementation"

### 2. Require Evidence
- "PASTE the test failure output in a code block"
- "SHOW the coverage report after each test"
- "PROVIDE the exact error message"

### 3. Set Time Boxes
- "Spend MAXIMUM 20 minutes on each test cycle"
- "If stuck for >10 minutes, document the issue and move on"

### 4. Define Success Narrowly
- "Success = All 8 tests pass with >95% coverage"
- NOT "Success = ProfileManager works well"

---

## Example: Strict TDD Task

```markdown
TASK: Create Profile Validation

DELIVERABLE: validation.rs with 6 tests achieving 100% coverage

TEST SEQUENCE (Complete in order):

1. Empty Name Test
   - Write: test_validate_empty_name()
   - Expects: Err(InvalidProfile("Name cannot be empty"))
   - Run: cargo test test_validate_empty_name
   - REQUIRED: Paste failure output showing "function not found"

2. Invalid Characters Test  
   - Write: test_validate_invalid_chars()
   - Expects: Err(InvalidProfile("Name contains invalid character: /"))
   - Run: cargo test test_validate_invalid_chars
   - REQUIRED: Paste failure output

[Continue for all 6 tests...]

FINAL VERIFICATION:
- Run: cargo tarpaulin --lib validation
- REQUIRED: Screenshot showing 100% coverage
- Run: cargo clippy -- -D warnings  
- REQUIRED: Output showing "0 warnings"
```

---

## Enforcement Checklist

When reviewing plans, ensure:

- [ ] Every test has a "paste output" requirement
- [ ] Checkpoints exist between test and implementation
- [ ] Failure conditions are explicit
- [ ] Time estimates are realistic (not rushed)
- [ ] No wiggle room for shortcuts
- [ ] Concrete metrics for success
- [ ] Incremental progress tracking
- [ ] Required evidence at each step
- [ ] Clear "STOP" triggers defined
- [ ] No "for now" or "temporary" language