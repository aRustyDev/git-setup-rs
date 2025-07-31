---
name: Checkpoint Review
about: Request review for a phase checkpoint
title: "CHECKPOINT: Phase X.Y - [Checkpoint Name]"
labels: checkpoint, needs-review
assignees: tech-lead, qa-engineer

---

## Checkpoint Information

**Phase**: [e.g., 1 - Foundation & Security]
**Checkpoint**: [e.g., 1.1 - Secure File System Complete]
**Work Plan Link**: [Link to specific checkpoint in WORK_PLAN.md]

## Pre-Checkpoint Checklist

I have completed ALL of the following before requesting review:

- [ ] All code committed and pushed to this branch
- [ ] All tests passing locally (`cargo test --all`)
- [ ] Code coverage meets minimum requirement: ___% (attach report)
- [ ] No compiler warnings: `cargo build --all-targets`
- [ ] No clippy warnings: `cargo clippy -- -D warnings`
- [ ] Documentation updated for all public APIs
- [ ] CHANGELOG.md updated with changes
- [ ] Self-review completed using criteria below

## Summary of Completed Work

### What Was Built
<!-- Provide a clear summary of what you implemented -->

### Key Decisions Made
<!-- Document any important technical decisions and why -->

### Deviations from Plan
<!-- List any deviations from the work plan with justification -->

## Testing

### Test Coverage
- Current coverage: ___% 
- Coverage report: [Link to HTML report]
- Key test scenarios covered:
  - [ ] [Scenario 1]
  - [ ] [Scenario 2]
  - [ ] [Scenario 3]

### Platform Testing
- [ ] Linux: Tested on [distro/version]
- [ ] macOS: Tested on [version]
- [ ] Windows: Tested on [version]

## Performance

### Benchmarks
<!-- Include relevant benchmark results -->
```
Operation       Target    Actual    Status
----------      ------    ------    ------
[Operation 1]   <10ms     Xms       ✅/❌
[Operation 2]   <5ms      Xms       ✅/❌
```

### Resource Usage
- Memory usage: ___ MB
- Binary size impact: +___ KB

## Security Considerations

### Security Checklist
- [ ] No hardcoded secrets or credentials
- [ ] Input validation on all external data
- [ ] No unsafe code without justification
- [ ] Sensitive data properly protected (zeroized)
- [ ] File permissions correctly set

### Security Notes
<!-- Any specific security considerations for reviewers -->

## Known Issues

### Technical Debt
<!-- List any technical debt introduced with plan to address -->

### Pending Items
<!-- List any items deferred to next phase with justification -->

## Screenshots/Demos

<!-- If UI-related, include screenshots or recordings -->

## Review Checklist

### For Tech Lead
- [ ] Architecture aligns with overall design
- [ ] Code follows project style guide
- [ ] No obvious performance issues
- [ ] Proper error handling throughout
- [ ] No code smells or anti-patterns

### For QA Engineer
- [ ] Test coverage adequate for new code
- [ ] Unit tests for all public functions
- [ ] Integration tests for key workflows
- [ ] Edge cases and error paths tested
- [ ] Tests are maintainable and clear

### For Security Engineer
- [ ] Security checklist items verified
- [ ] No new vulnerabilities introduced
- [ ] Threat model considerations addressed
- [ ] Secure coding practices followed

## Definition of Done

This checkpoint is complete when:
- [ ] All reviewers have approved
- [ ] All CI checks are passing
- [ ] Any requested changes have been made
- [ ] PR has been merged to main
- [ ] Checkpoint tag has been created

## Notes for Reviewers

<!-- Any additional context that would help reviewers -->

---

**Reminder**: This is a MANDATORY checkpoint. Work cannot proceed to the next phase until this review is complete and approved.