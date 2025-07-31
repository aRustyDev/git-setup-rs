# Junior Developer Guide Template

## How to Use This Template

This template provides standardized components for making phase plans junior-developer friendly. Include these elements throughout work plans.

## Components

### 1. Concept Explanation Box

```markdown
💡 **Junior Dev Concept**: [Concept Name]
**What it is**: Brief explanation in simple terms
**Why we use it**: The problem it solves
**Example**:
```rust
// Simple example showing the concept
```
**Learn more**: [Link to documentation]
```

### 2. Step-by-Step Breakdown

```markdown
#### Task X.X.X: [Task Name] ([Time Estimate])

**Prerequisites**:
- [ ] Read: [Required reading with link]
- [ ] Understand: [Concept from previous phase]
- [ ] Setup: [Any tools or environment needed]

**Implementation Steps**:

1. **[Step Name]** (X minutes)
   - What to do
   - Expected outcome
   - Code snippet or command
   
   💡 **Tip**: [Helpful hint for this step]

2. **[Next Step]** (X minutes)
   - Clear instructions
   - Decision points explained
   
   ⚠️ **Common Mistake**: [What goes wrong]
   ✅ **Instead**: [Correct approach]
```

### 3. Debugging Guide

```markdown
#### When Things Go Wrong

**Symptom**: [Error message or behavior]
**Likely Cause**: [Common reason]
**How to Debug**:
1. Check [specific thing]
2. Run `[debugging command]`
3. Look for [indicators]
**Solution**: [How to fix]
```

### 4. Code Template with Explanations

```markdown
```rust
// Start with this template and fill in the TODOs

use std::path::Path; // Why: We need path manipulation

pub struct YourStruct {
    // TODO: Add fields you need
    // Consider: What data does this need to hold?
}

impl YourStruct {
    pub fn new() -> Self {
        // TODO: Initialize your struct
        // Hint: Use Default::default() for simple cases
        todo!("Implement new")
    }
    
    // TODO: Add methods
    // Think about: What operations does this need?
}

// Tests are required - start here!
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_new() {
        // TODO: Test the constructor
        // Verify: Initial state is correct
    }
}
```
```

### 5. Learning Resources Section

```markdown
#### 📚 Learning Resources

**Before Starting**:
- 🎯 [Concept Overview](link) - 10 min read
- 🎥 [Video Tutorial](link) - 15 min watch
- 🔧 [Interactive Exercise](link) - 20 min practice

**If You Get Stuck**:
- 💬 Ask in #rust-beginners channel
- 👥 Pair with [designated mentor]
- 📖 Review [specific book chapter]
```

### 6. Progress Checkpoint

```markdown
#### ✅ Self-Check Before Moving On

Can you answer these questions?
- [ ] What problem does [concept] solve?
- [ ] Why did we choose [approach] over [alternative]?
- [ ] What would happen if [edge case]?

Can you complete these tasks?
- [ ] Modify the code to [small change]
- [ ] Add a test for [specific case]
- [ ] Explain the code to someone else
```

## Example Usage

Here's how to apply this template to a real task:

```markdown
#### Task 1.2.3: Implement Atomic File Write (3 hours)

💡 **Junior Dev Concept**: Atomic File Operations
**What it is**: Writing files in a way that either completely succeeds or completely fails
**Why we use it**: Prevents corrupted files if the program crashes mid-write
**Example**:
```rust
// Instead of directly writing:
// ❌ fs::write("config.toml", data)?;

// We write to temp then rename:
// ✅ fs::write("config.toml.tmp", data)?;
//    fs::rename("config.toml.tmp", "config.toml")?;
```
**Learn more**: [Atomic Operations in File Systems](https://lwn.net/Articles/457667/)

**Prerequisites**:
- [ ] Read: [std::fs documentation](https://doc.rust-lang.org/std/fs/)
- [ ] Understand: Error handling with Result<T, E>
- [ ] Setup: Create `src/fs/atomic.rs` file

**Implementation Steps**:

1. **Create the atomic write function** (45 minutes)
   ```rust
   use std::fs;
   use std::path::Path;
   
   pub fn write_atomic(path: &Path, contents: &[u8]) -> std::io::Result<()> {
       // TODO: Generate temporary file name
       let temp_path = /* your code here */;
       
       // TODO: Write to temporary file
       
       // TODO: Rename temporary to final
       
       Ok(())
   }
   ```
   
   💡 **Tip**: Use `format!("{}.tmp", path.display())` for temp name

2. **Handle errors properly** (45 minutes)
   
   ⚠️ **Common Mistake**: Not cleaning up temp file on error
   ✅ **Instead**: Use a guard pattern or defer cleanup
```

## Tips for Template Users

1. **Adjust complexity** based on actual junior developer experience level
2. **Include real examples** from the project, not just generic code
3. **Test instructions** with an actual junior developer if possible
4. **Keep explanations concise** - link to detailed resources
5. **Use consistent emoji** for visual scanning:
   - 💡 = Tip/Concept
   - ⚠️ = Warning/Common Mistake
   - ✅ = Correct Approach
   - 📚 = Learning Resource
   - 🛑 = Stop/Checkpoint