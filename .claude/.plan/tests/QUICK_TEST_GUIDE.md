# Quick Test Guide for git-setup-rs

A concise guide for rapid testing across platforms. Use this for smoke tests and quick validation.

## 🚀 5-Minute Smoke Test

Run these commands on any platform to verify basic functionality:

```bash
# 1. Check installation
git-setup --version

# 2. Create a test profile
git-setup profile create --name test --email test@example.com --user-name "Test User"

# 3. List profiles
git-setup profile list

# 4. Apply profile
mkdir test-repo && cd test-repo
git init
git-setup profile use test

# 5. Verify Git config
git config user.email
# Should show: test@example.com

# 6. Launch TUI (press 'q' to quit)
git-setup tui

# 7. Clean up
cd .. && rm -rf test-repo
git-setup profile delete test --force
```

## 🖥️ Platform-Specific Quick Tests

### Windows (PowerShell)
```powershell
# Path handling
git-setup config set test.path "C:\Program Files\Test"
git-setup config get test.path

# Long path
$long = "C:\Test\" + ("Long\" * 30) + "Path"
git-setup config set test.long $long
```

### macOS
```bash
# Universal binary check
file $(which git-setup)
# Should show: "universal binary with 2 architectures"

# Keychain test (if configured)
git-setup signing test
```

### Linux
```bash
# XDG compliance
echo "Config at: ${XDG_CONFIG_HOME:-$HOME/.config}/git-setup"
ls -la ${XDG_CONFIG_HOME:-$HOME/.config}/git-setup/

# Permission check
stat -c "%a" ~/.config/git-setup/config.toml
# Should be 644 or 600
```

## 🧪 Feature Quick Tests

### Profile Inheritance (2 min)
```bash
# Create base
git-setup profile create --name base --email base@company.com

# Create child
git-setup profile create --name feature --extends base --user-name "Feature Dev"

# Verify inheritance
git-setup profile show feature
# Should have email from base, name from feature
```

### 1Password Integration (3 min)
```bash
# Check if available
git-setup signing providers

# If 1Password listed:
git-setup signing setup --method ssh --provider 1password
```

### Auto-Detection (2 min)
```bash
# Set up rule
git-setup detect add --path "~/work/*" --profile work

# Test detection
cd ~/work/project
git-setup detect current
# Should suggest "work" profile
```

## 🎯 Critical Path Test

The most important user journey - setting up for work:

```bash
# 1. First-time setup
git-setup init

# 2. Create work profile
git-setup profile create --interactive

# 3. Go to work project
cd ~/work/important-project

# 4. Apply profile
git-setup profile use work

# 5. Make a commit
echo "test" > test.txt
git add test.txt
git commit -m "Test commit"

# 6. Verify author
git log -1 --pretty=format:"%an <%ae>"
# Should match work profile
```

## 🐛 Common Issues Quick Check

### Issue: "Profile not found"
```bash
git-setup profile list  # Check available profiles
ls ~/.config/git-setup/profiles/  # Check files exist
```

### Issue: "Permission denied"  
```bash
ls -la ~/.config/git-setup/  # Check permissions
whoami  # Check current user
```

### Issue: TUI not working
```bash
echo $TERM  # Check terminal type
tput colors  # Check color support
```

### Issue: Git config not updating
```bash
git config --list --show-origin  # See all configs
git-setup profile show work  # Check profile values
```

## ⚡ Performance Quick Check

```bash
# Measure key operations
time git-setup --version          # Should be <100ms
time git-setup profile list       # Should be <20ms
time git-setup profile show work  # Should be <20ms

# Memory check (Linux/macOS)
/usr/bin/time -l git-setup profile list 2>&1 | grep "maximum resident"
# Should be <50MB
```

## 📋 Release Smoke Test Checklist

Run before any release:

- [ ] **Install**: Fresh install works
- [ ] **Upgrade**: Upgrade preserves profiles  
- [ ] **Create**: Can create new profile
- [ ] **List**: Shows all profiles
- [ ] **Apply**: Git config updated
- [ ] **TUI**: Launches and navigates
- [ ] **Help**: `--help` works for all commands
- [ ] **Errors**: Bad input shows clear errors
- [ ] **Performance**: Feels instant
- [ ] **Clean**: Uninstall removes everything

## 🔄 Regression Test Points

After any major change, verify:

1. **Profiles persist** after upgrade
2. **Paths work** on all platforms  
3. **Git integration** still functions
4. **TUI renders** correctly
5. **No credentials** in logs
6. **Performance** hasn't degraded

## 💡 Testing Tips

1. **Always test as regular user** (not admin/root)
2. **Test with real Git repos** not just empty ones
3. **Try non-ASCII** in names and paths
4. **Test offline** for good error messages
5. **Resize terminal** while TUI is running

---

*Quick tests catch most issues. When in doubt, run the full test matrix.*