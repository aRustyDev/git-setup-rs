# Checkpoint Enforcement Template

## Purpose
This template ensures that checkpoints are mandatory stop points that cannot be bypassed without proper review and approval.

## Checkpoint Structure

```markdown
---

## 🛑 CHECKPOINT [NUMBER]: [CHECKPOINT NAME]

### ⚠️ MANDATORY STOP POINT ⚠️

**DO NOT PROCEED** past this checkpoint without completing the review process and obtaining all required approvals.

### Pre-Checkpoint Checklist

Before requesting review, ensure ALL items are complete:

- [ ] All code committed and pushed to feature branch
- [ ] All tests passing locally (`cargo test --all`)
- [ ] Code coverage meets minimum (80%): `cargo tarpaulin --out Html`
- [ ] No compiler warnings: `cargo build --all-targets`
- [ ] No clippy warnings: `cargo clippy -- -D warnings`
- [ ] Documentation updated for public APIs
- [ ] CHANGELOG.md updated with your changes
- [ ] Self-review completed using review criteria below

### Review Process

1. **Create Pull Request**
   ```bash
   git checkout -b checkpoint/phase-X-Y-description
   git add .
   git commit -m "CHECKPOINT: Phase X.Y - [Checkpoint Name]"
   git push origin checkpoint/phase-X-Y-description
   ```
   
   PR Title: `CHECKPOINT: Phase X.Y - [Checkpoint Name]`
   
   PR Description MUST include:
   - [ ] Link to this checkpoint in the work plan
   - [ ] Summary of completed work
   - [ ] Any deviations from plan with justification
   - [ ] Known issues or technical debt
   - [ ] Screenshots/demos if UI-related

2. **Run Automated Checks**
   The following CI checks MUST pass:
   - [ ] Build and test pipeline
   - [ ] Code coverage threshold
   - [ ] Security scan (cargo audit)
   - [ ] License check
   - [ ] Documentation build

3. **Request Reviews**
   Tag required reviewers in PR:
   - @[tech-lead] - Architecture and code quality
   - @[qa-engineer] - Test coverage and quality
   - @[security-engineer] - Security implications (if applicable)
   - @[product-owner] - Requirements alignment (if applicable)

### Review Criteria

Reviewers will validate:

#### Code Quality (Tech Lead)
- [ ] Architecture aligns with overall design
- [ ] Code follows project style guide
- [ ] No obvious performance issues
- [ ] Proper error handling throughout
- [ ] No code smells or anti-patterns

#### Test Quality (QA Engineer)
- [ ] Test coverage ≥ 80% for new code
- [ ] Unit tests for all public functions
- [ ] Integration tests for key workflows
- [ ] Edge cases and error paths tested
- [ ] Tests are maintainable and clear

#### Security (Security Engineer - if applicable)
- [ ] No hardcoded secrets or credentials
- [ ] Input validation on all external data
- [ ] No SQL injection or command injection risks
- [ ] Proper authentication/authorization
- [ ] Sensitive data properly protected

### Approval Requirements

The following approvals are REQUIRED before proceeding:

- [ ] **Tech Lead**: Approved ✅ (GitHub approval)
- [ ] **QA Engineer**: Approved ✅ (GitHub approval)
- [ ] **CI Pipeline**: All checks passed ✅
- [ ] **Additional**: [Role] approved ✅ (if required)

### Post-Review Actions

After approval:
1. Merge PR to main branch
2. Tag the commit: `git tag checkpoint-X.Y`
3. Update project board/tracker
4. Notify team in #dev-checkpoints channel
5. Begin next phase of work

### Escalation Process

**If blocked on review for >24 hours**:
1. Post in #dev-blocked channel with:
   - PR link
   - Specific blockers
   - Attempted solutions
2. Tag engineering manager if blocked >48 hours
3. Schedule sync meeting if complex issues

**If review reveals major issues**:
1. Do NOT proceed with next phase
2. Address all feedback in current branch
3. Re-request review after fixes
4. Update time estimates for project

### Consequences of Skipping

**If you proceed without approval**:
- Work will be rejected in later reviews
- You'll need to redo work based on feedback
- Project timeline impacts
- Noted in performance reviews
- Loss of autonomy on future checkpoints

### Checkpoint Approval Record

| Reviewer | Role | Date | Approval | Notes |
|----------|------|------|----------|-------|
| [Name] | Tech Lead | [Date] | ✅/❌ | [Comments] |
| [Name] | QA Engineer | [Date] | ✅/❌ | [Comments] |
| [Name] | [Other] | [Date] | ✅/❌ | [Comments] |

**Checkpoint Completed**: [Date and Time]
**Merged PR**: #[PR Number]
**Commit Hash**: [SHA]

---
```

## Automated Enforcement

### GitHub Actions Workflow

```yaml
# .github/workflows/checkpoint-enforcement.yml
name: Checkpoint Enforcement

on:
  pull_request:
    types: [opened, edited]

jobs:
  check-checkpoint:
    if: startsWith(github.event.pull_request.title, 'CHECKPOINT:')
    runs-on: ubuntu-latest
    steps:
      - name: Check PR Format
        run: |
          # Verify PR title format
          if ! [[ "${{ github.event.pull_request.title }}" =~ ^CHECKPOINT:\ Phase\ [0-9]+\.[0-9]+\ -\ .+ ]]; then
            echo "❌ PR title must match format: CHECKPOINT: Phase X.Y - Description"
            exit 1
          fi
      
      - name: Check Required Sections
        run: |
          # Verify PR body contains required sections
          body="${{ github.event.pull_request.body }}"
          
          required_sections=(
            "Link to this checkpoint"
            "Summary of completed work"
            "Any deviations from plan"
            "Known issues"
          )
          
          for section in "${required_sections[@]}"; do
            if ! echo "$body" | grep -q "$section"; then
              echo "❌ Missing required section: $section"
              exit 1
            fi
          done
      
      - name: Auto-assign Reviewers
        uses: actions/github-script@v6
        with:
          script: |
            github.rest.pulls.requestReviewers({
              owner: context.repo.owner,
              repo: context.repo.repo,
              pull_number: context.issue.number,
              reviewers: ['tech-lead', 'qa-engineer']
            })
      
      - name: Add Checkpoint Label
        uses: actions/github-script@v6
        with:
          script: |
            github.rest.issues.addLabels({
              owner: context.repo.owner,
              repo: context.repo.repo,
              issue_number: context.issue.number,
              labels: ['checkpoint', 'needs-review']
            })
```

### Pre-commit Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit

# Check if we're at a checkpoint
if grep -q "^## 🛑 CHECKPOINT" $(git diff --cached --name-only | grep -E "WORK_PLAN\.md$"); then
    echo "⚠️  WARNING: You're at a checkpoint!"
    echo "Have you completed the review process? (y/n)"
    read -r response
    if [[ ! "$response" =~ ^[Yy]$ ]]; then
        echo "❌ Commit blocked. Complete checkpoint review first."
        exit 1
    fi
fi
```

## Usage Instructions

1. **For Work Plan Authors**: Copy the checkpoint structure into your work plan at each major milestone
2. **For Developers**: Follow the process exactly - no shortcuts
3. **For Reviewers**: Use the criteria checklist during reviews
4. **For Team Leads**: Monitor checkpoint compliance

## Checkpoint Tracking Dashboard

Create a simple dashboard to track checkpoint status:

| Phase | Checkpoint | Developer | Status | PR | Reviewers | Date |
|-------|------------|-----------|--------|----|-----------|----- |
| 1 | Secure File System | @dev1 | ✅ Complete | #42 | @tech-lead, @qa | 2024-01-15 |
| 1 | Memory Safety | @dev1 | 🔄 In Review | #43 | @tech-lead | - |
| 2 | Profile Storage | @dev2 | 📋 Pending | - | - | - |