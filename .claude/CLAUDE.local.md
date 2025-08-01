# Git-Setup Rust Project Instructions

## Project Context
You are working on git-setup-rs, a Rust reimplementation of a Git configuration management tool. Comprehensive documentation exists in the `.claude/` directory. Read and understand these files thoroughly before proceeding with any task.

## Core Development Principles

### 1. Test-Driven Development (TDD) is MANDATORY - STRICT ENFORCEMENT

**VIOLATIONS WILL RESULT IN CHECKPOINT FAILURE**

#### TDD Enforcement Rules:
1. **PROVE THE RED PHASE**: You MUST paste the failing test output showing the test fails
2. **NO IMPLEMENTATION WITHOUT RED**: Writing implementation code before showing RED test output = IMMEDIATE CHECKPOINT FAILURE
3. **COMMIT THE RED TEST**: You MUST commit the failing test with message `test: [RED] <description>`
4. **MINIMAL GREEN**: Only write enough code to make the test pass, nothing more
5. **COMMIT THE GREEN**: Commit passing implementation with message `feat: [GREEN] <description>`
6. **REFACTOR ONLY WITH GREEN**: Any refactoring must maintain passing tests

#### TDD Audit Trail:
For EVERY feature implementation, you MUST provide:
```bash
# 1. Show the test code
cat src/module/tests.rs

# 2. Run and show RED output
cargo test test_name -- --nocapture
# MUST show "test result: FAILED"

# 3. Commit the failing test
git add . && git commit -m "test: [RED] test for feature X"

# 4. Show implementation code
cat src/module/implementation.rs

# 5. Run and show GREEN output
cargo test test_name -- --nocapture
# MUST show "test result: ok"

# 6. Commit the implementation
git add . && git commit -m "feat: [GREEN] implement feature X"
```

**CHECKPOINT REVIEWERS WILL VERIFY**:
- Git history shows RED commits before GREEN commits
- No implementation code exists without corresponding tests
- Test coverage meets or exceeds requirements
- Tests actually test the requirements, not just exist

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

## CHECKPOINT REVIEW PROCESS - MANDATORY PROCEDURES

### Directory Structure for Reviews
Each phase checkpoint MUST have the following structure:
```
.claude/.reviews/
├── phase-1/
│   ├── index.md                    # Links to all checkpoint contents with line numbers
│   ├── checkpoint-1/
│   │   ├── FEEDBACK.md            # Reviewer feedback (append-only)
│   │   ├── QUESTIONS.md           # Q&A between junior dev and reviewer
│   │   ├── CHALLENGES.md          # Challenges faced by junior dev (append-only)
│   │   ├── RESEARCH.md            # Research plans and findings (append-only)
│   │   └── LESSONS.md             # Lessons learned (append-only)
│   └── checkpoint-2/
│       └── ... (same structure)
└── phase-2/
    └── ... (same structure)
```

### REVIEWER RESPONSIBILITIES

1. **Update Index File**
   - Path: `/Users/asmith/code/public/git-setup-rs/.claude/.reviews/phase-X/index.md`
   - MUST contain links to each checkpoint's contents
   - MUST include specific line numbers where updates were added
   - Example:
     ```markdown
     ## Checkpoint 1: Secure File System
     - [FEEDBACK.md](checkpoint-1/FEEDBACK.md) - Lines 1-45 (initial), 46-72 (revision 1)
     - [QUESTIONS.md](checkpoint-1/QUESTIONS.md) - Q1 (lines 1-15), Q2 (lines 16-28)
     - [CHALLENGES.md](checkpoint-1/CHALLENGES.md) - Lines 1-23
     ```

2. **Write Feedback**
   - Path: `/Users/asmith/code/public/git-setup-rs/.claude/.reviews/phase-X/checkpoint-Y/FEEDBACK.md`
   - This is APPEND-ONLY - add new feedback below previous
   - Include timestamp and review iteration number
   - Must cover: TDD compliance, code quality, security, performance, completeness
   - Example:
     ```markdown
     ## Review Iteration 1 - 2025-07-30 14:23:00
     
     ### TDD Compliance: FAILED
     - No RED test output provided for atomic operations
     - Implementation found without corresponding test-first evidence
     
     ### Requirements:
     1. Show RED test output for atomic_write function
     2. Add security tests for permission validation
     ```

3. **Monitor Questions**
   - Check `/Users/asmith/code/public/git-setup-rs/.claude/.reviews/phase-X/checkpoint-Y/QUESTIONS.md`
   - MUST check BEFORE starting review
   - MUST check every 30 seconds for 4 minutes AFTER review
   - Answer directly next to each question
   - Example:
     ```markdown
     Q1: Should atomic writes use tempfile or manual implementation?
     A1: Use tempfile crate - it's battle-tested and handles edge cases.
     
     Q2: What permission should backup files have?
     A2: Same as original file (preserve permissions).
     ```

4. **Approval Criteria**
   - **For Checkpoints**: 
     - All checkpoint deliverables complete
     - All TDD evidence MUST be provided
     - No security vulnerabilities
     - Code meets quality standards
     - Tests passing with required coverage
   - **For Phase Completion Only**:
     - Score MUST be >95/100
     - Phase criteria MUST be 100% complete
     - All checkpoints approved
     - Performance targets met

### JUNIOR DEVELOPER RESPONSIBILITIES

1. **Ask Questions IMMEDIATELY**
   - Path: `/Users/asmith/code/public/git-setup-rs/.claude/.reviews/phase-X/checkpoint-Y/QUESTIONS.md`
   - Write questions and WAIT 3 minutes
   - Check every 30 seconds until answered
   - Ask clarifications about feedback IMMEDIATELY after reading

2. **Document Challenges**
   - Path: `/Users/asmith/code/public/git-setup-rs/.claude/.reviews/phase-X/checkpoint-Y/CHALLENGES.md`
   - APPEND-ONLY file
   - Document with timestamp
   - Include: what you tried, what failed, current blockers

3. **Record Research**
   - Path: `/Users/asmith/code/public/git-setup-rs/.claude/.reviews/phase-X/checkpoint-Y/RESEARCH.md`
   - APPEND-ONLY file
   - Include: research question, sources consulted, findings, decision made

4. **Capture Lessons**
   - Path: `/Users/asmith/code/public/git-setup-rs/.claude/.reviews/phase-X/checkpoint-Y/LESSONS.md`
   - APPEND-ONLY file
   - Include: what you learned, what you'd do differently, key insights

5. **Complete ALL Feedback**
   - Read ALL feedback in FEEDBACK.md
   - Complete ALL requirements before continuing
   - Ask clarifying questions if needed
   - DO NOT proceed until feedback is addressed

### WORKFLOW ENFORCEMENT

1. **Junior Dev reaches checkpoint** → Creates review directory structure
2. **Junior Dev writes any pre-review questions** → Waits for answers
3. **Reviewer checks questions** → Answers all questions
4. **Reviewer performs review** → Writes to FEEDBACK.md
5. **Reviewer monitors for questions** → 4 minutes post-review
6. **Junior Dev reads feedback** → Asks clarifications immediately
7. **Junior Dev implements feedback** → Documents challenges/lessons
8. **Junior Dev requests re-review** → Process repeats until approved

### FAILURE CONDITIONS

**Checkpoint FAILS if**:
- TDD evidence not provided (RED → GREEN commits)
- Questions not answered within review window
- Feedback not fully addressed before continuing
- Checkpoint deliverables incomplete
- Review files not properly maintained
- Junior dev proceeds without approval

**Phase Completion FAILS if**:
- Any checkpoint not approved
- Score <95/100
- Phase criteria <100% complete
- Performance targets not met
- Missing documentation
- Security vulnerabilities found

This process ensures knowledge transfer, quality control, and proper documentation throughout the project.
