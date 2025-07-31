# Alignment Plan: git-setup-rs to SPEC Compliance

## Executive Summary

The current implementation of git-setup-rs is fundamentally broken and requires significant rework to meet the SPEC requirements. The most critical issue is the complete lack of persistence - profiles exist only in memory and are lost when the application exits. Additionally, tests don't compile, and core functionality is missing.

---

## Critical Path to Working Software

### Phase 1: Fix Compilation and Tests (1-2 days)
**Goal**: Get the project to compile and pass tests

1. **Fix ProfileManager Interface**
   - Change `write` calls to `create` in commands/add.rs
   - Fix all compilation errors in tests
   - Remove or fix async/await issues
   - Fix borrow checker errors in TUI tests

2. **Remove Unnecessary Async**
   - Remove tokio dependency
   - Convert all async functions to sync
   - No actual async I/O is being performed

3. **Fix Test Compilation**
   - Update test APIs to match actual implementations
   - Fix all 94 compilation errors
   - Ensure all tests can at least run (even if failing)

### Phase 2: Implement Persistence Layer (2-3 days)
**Goal**: Make profiles persist to disk per SPEC

1. **Create FileProfileManager**
   ```rust
   pub struct FileProfileManager {
       config_loader: Arc<dyn ConfigLoaderTrait>,
       cache: Arc<Mutex<Option<Config>>>,
   }
   ```

2. **Integrate with ConfigLoader**
   - Load profiles from config.toml on startup
   - Save profiles to config.toml on create/update/delete
   - Handle config file creation if missing

3. **Update Main.rs**
   - Create ConfigLoader instance
   - Pass it to FileProfileManager
   - Wire up proper initialization

4. **Write Integration Tests**
   - Test actual file I/O
   - Test profile persistence across restarts
   - Test config file format compliance

### Phase 3: Implement Core Commands (3-4 days)
**Goal**: Make all CLI commands functional

1. **Apply Command**
   - Actually apply git configuration
   - Handle scope properly (local/global/system)
   - Configure signing based on key type
   - Add includeIf configuration

2. **Add Command (Interactive)**
   - Replace placeholder implementation
   - Add proper interactive prompts
   - Validate input properly
   - Integrate with 1Password for key creation

3. **Import Command**
   - Implement agent.toml import
   - Parse 1Password agent configuration
   - Create profiles from agent data

4. **Edit/Delete Commands**
   - Implement profile editing
   - Ensure changes persist
   - Handle edge cases (default profile, etc.)

### Phase 4: TUI Implementation (4-5 days)
**Goal**: Create working TUI per SPEC

1. **Fix TUI Framework**
   - Remove over-engineered abstractions
   - Focus on working implementation
   - Fix all compilation errors

2. **Implement Required Screens**
   - Main Menu (with number key navigation)
   - Profile List (with table view)
   - Add Profile Wizard (multi-step)
   - Edit Profile Screen
   - Profile Selector (with fuzzy search)

3. **Navigation and Events**
   - Implement proper event handling
   - Add keyboard shortcuts per SPEC
   - Ensure back navigation works

### Phase 5: Advanced Features (2-3 days)
**Goal**: Complete remaining SPEC requirements

1. **Auto-Detection**
   - Wire up detection to actual use
   - Test with real repositories
   - Add detection hints to UI

2. **1Password Integration**
   - Create SSH keys in 1Password
   - Create GPG items with template
   - List and select vaults
   - Handle authentication properly

3. **GPG Support**
   - Implement GPG key generation
   - Import existing keys
   - Configure git for GPG signing

---

## Technical Debt to Address

### Immediate Fixes:
1. Remove all unused imports
2. Run `cargo fmt` on entire codebase
3. Fix all clippy warnings
4. Remove dead code
5. Add proper documentation

### Architectural Changes:
1. Remove unnecessary abstractions
2. Simplify trait hierarchies
3. Remove async where not needed
4. Consolidate duplicate code

### Testing Strategy:
1. Write integration tests FIRST
2. Test actual git operations
3. Test file persistence
4. Test 1Password integration
5. Add end-to-end tests

---

## Resource Requirements

### Time Estimate: 12-17 days
- Phase 1: 1-2 days
- Phase 2: 2-3 days
- Phase 3: 3-4 days
- Phase 4: 4-5 days
- Phase 5: 2-3 days

### Skills Needed:
- Strong Rust knowledge (borrow checker, traits)
- TUI development experience (ratatui)
- Git internals knowledge
- Testing expertise (TDD approach)

---

## Definition of Done

The project will be considered aligned with SPEC when:

1. **All tests pass** with >80% coverage
2. **Profiles persist** to ~/.config/git/setup/config.toml
3. **All CLI commands work** as specified
4. **TUI is functional** with all required screens
5. **1Password integration** works for SSH/GPG keys
6. **Git configuration** is actually applied
7. **Cross-platform** functionality verified
8. **Performance** meets requirements (<100ms TUI start)
9. **No clippy warnings** or formatting issues
10. **Documentation** is complete

---

## Recommendations

1. **Start Over**: Given the fundamental issues, consider starting fresh with TDD
2. **Incremental Delivery**: Focus on CLI first, then add TUI
3. **Pair Programming**: Junior dev needs mentoring on TDD and Rust patterns
4. **Code Reviews**: Implement PR reviews before merging
5. **CI/CD**: Set up automated testing to catch issues early

---

## Risk Assessment

### High Risk:
- Current architecture may require significant refactoring
- Junior developer may need extensive mentoring
- 1Password integration complexity

### Medium Risk:
- TUI implementation timeline
- Cross-platform testing resources
- GPG implementation complexity

### Mitigation:
- Daily stand-ups to track progress
- Pair programming for complex features
- Incremental testing and validation
- Clear acceptance criteria for each phase