# Git-Setup Rust Project Instructions

## Project Context
You are working on git-setup-rs, a Rust reimplementation of a Git configuration management tool. Comprehensive documentation exists in the `.claude/` directory. Read and understand these files thoroughly before proceeding with any task.

## Core Development Principles

### 1. Test-Driven Development (TDD) is MANDATORY
- **ALWAYS** write tests FIRST, before any implementation
- Follow the Red-Green-Refactor cycle rigorously:
  1. Write a failing test that describes desired behavior
  2. Run the test and verify it fails
  3. Write the MINIMUM code to make the test pass
  4. Refactor while keeping tests green
- Use `just watch-test <test_name>` for rapid feedback
- If you find yourself writing code without a test, STOP and write the test first

### 2. Commit Discipline
- Make small, atomic commits after each TDD cycle
- Commit messages should be clear and describe the change
- Example workflow:
  ```bash
  # After writing test
  git add . && git commit -m "test: add test for profile validation"

  # After making test pass
  git add . && git commit -m "feat: implement profile validation"

  # After refactoring
  git add . && git commit -m "refactor: simplify validation logic"
  ```

### 3. Research and Exploration
Before implementing ANY feature:
1. Read the relevant Go implementation in `../git-setup-go/`
2. Study the corresponding documentation in `.claude/`
3. Understand the full context and implications
4. Research Rust idioms and patterns for the task
5. Check if similar functionality exists in the codebase

### 4. Methodical Thinking Process
For every task:
1. **Understand**: What exactly needs to be done?
2. **Research**: How was it done in Go? What are Rust best practices?
3. **Plan**: Break down into small, testable steps
4. **Implement**: Follow TDD for each step
5. **Verify**: Does it work? Is it idiomatic Rust?
6. **Document**: Update comments and documentation

## Communication Requirements

### Ask Clarifying Questions
You MUST ask questions when:
- The task description is vague or ambiguous
- Multiple interpretations are possible
- Dependencies or prerequisites are unclear
- The expected behavior is not fully specified
- You're unsure about design decisions

Example:
```
"You asked me to 'implement profile management'. I need clarification:
1. Which specific aspect? (Creation, deletion, updating, listing?)
2. Should this include the TUI interface or just the backend logic?
3. Are there specific validation rules I should be aware of?
4. Should I implement all CRUD operations or start with a subset?"
```

### Confirm Understanding
Before starting any implementation:
1. Restate the task in your own words
2. List your assumptions
3. Outline your planned approach
4. Ask for confirmation

Example:
```
"Let me confirm my understanding:
- Task: Implement fuzzy matching for profile names
- This involves: Creating a scoring algorithm that matches partial names
- Approach: I'll implement the algorithm from ADVANCED_FEATURES.md
- I'll start by writing tests for exact matches, prefix matches, and subsequence matches
- Then implement the scoring function to pass these tests
Is this correct?"
```

## Professional Communication

### Be Direct and Honest
- If something is wrong or could be improved, say so directly
- Don't sugarcoat problems or limitations
- Provide constructive criticism with alternatives
- Challenge assumptions when they seem incorrect

### Think Like a Veteran Rust Developer
Ask yourself:
- Is this idiomatic Rust?
- Am I fighting the borrow checker or working with it?
- Could this be more efficient using Rust's zero-cost abstractions?
- Am I handling all error cases properly?
- Is this code maintainable and clear?

## Continuous Learning

### Update Your Understanding
As you work:
1. Document discoveries in comments
2. Update your mental model of the system
3. Identify patterns and abstractions
4. Note any deviations from the original Go implementation

### Analyze Before Implementing
Before writing any code:
1. Review similar code in the project
2. Consider multiple approaches
3. Think about edge cases
4. Consider performance implications
5. Ensure the solution is testable

## Project-Specific Guidelines

### Use the Justfile
The project includes a comprehensive Justfile. Use it:
```bash
just tdd-help          # Learn TDD commands
just new-module <name> # Create new modules
just test-one <test>   # Run specific tests
just watch            # Auto-run tests on changes
just ci               # Run full CI pipeline
```

### Follow Existing Patterns
- Error handling: Use the custom `GitSetupError` type
- External tools: Implement traits for testability
- Configuration: Use the established TOML structure
- Testing: Follow the patterns in existing test files

### Key Documentation Files
Priority reading order:
1. `CONTEXT.md` - Understand the project
2. `SPEC.md` - Know the requirements
3. `TDD_GUIDE.md` - Learn testing approach
4. `IMPLEMENTATION_GUIDE.md` - Follow development patterns
5. `RUST_PATTERNS.md` - Write idiomatic code

## When You Get Stuck

1. **Re-read** the relevant documentation
2. **Check** the Go implementation for clarity
3. **Write** a simpler test to understand the problem
4. **Ask** specific questions about what's unclear
5. **Break** the problem into smaller pieces

## Remember

- You are building a tool that people will depend on
- Quality matters more than speed
- Tests are not optional, they are the foundation
- Clear code is better than clever code
- When in doubt, ask for clarification

Think step by step. Be thorough. Be precise. Build something excellent.
