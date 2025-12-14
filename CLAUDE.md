## 🔧 PUNCH Quick Commands
**Instant Analysis:** `punch go .` | `punch quality .` | `punch stats` | Zero setup required.

---

# Claude Code Configuration - CrabCamera Project

## 🚨 CRITICAL DEVELOPMENT PRINCIPLE

**READ BEFORE WRITE RULE**: ALWAYS use the Read tool to examine a file before using Edit, MultiEdit, or Write tools. This prevents compilation errors and maintains code integrity.

**EXAMPLE WORKFLOW**:
```
❌ WRONG: Edit file without reading
✅ CORRECT: Read → Understand → Edit
```

---

## 🦀 CrabCamera Project Specifics

### Current Status: v0.2.0 DEVELOPMENT COMPLETE
- **Version**: 0.2.0 (major release with advanced camera controls)
- **Features**: Professional camera controls, plant photography optimization, performance improvements
- **Architecture**: Rust + Tauri 2.0 plugin system
- **Demo**: Plant Photography Studio (HTML/JavaScript demo)

### Project Structure
```
crabcamera/
├── src/
│   ├── commands/           # Tauri command handlers
│   │   ├── advanced.rs     # v0.2.0: Advanced camera controls
│   │   ├── capture.rs      # Photo capture operations
│   │   └── init.rs         # Camera initialization
│   ├── platform/           # Platform-specific implementations
│   ├── types.rs           # Core data structures
│   └── lib.rs             # Main library entry point
├── demo/                  # Plant Photography Studio demo
├── CHANGELOG.md           # v0.2.0 comprehensive changelog
└── Cargo.toml            # Dependencies and metadata
```

### Key Dependencies
- **tauri = "2.0"** - Desktop app framework
- **nokhwa = "0.10"** - Cross-platform camera backend
- **tokio = "1.40"** - Async runtime
- **uuid = "1.10"** - Frame identification
- **chrono = "0.4"** - Timestamp handling
- **image = "0.25"** - Image processing

---

## 🛠️ Development Commands

### Build & Test
```powershell
# Check compilation (ALWAYS run before committing)
cargo check --all-features

# Run full test suite
cargo test --all-features

# Build for release
cargo build --release
```

### Git Operations (IMPORTANT)
```powershell
# Check current status BEFORE any commits
git status

# Check what's staged
git diff --staged

# Check commit history
git log --oneline -10
```

---

## 🚨 NEVER COMMIT WITHOUT HUMAN APPROVAL

**CRITICAL RULE**: All code changes, especially for v0.2.0 release, must be reviewed by human before any git commits or pushes.

**🚫 NEVER ADD CLAUDE AS CO-AUTHOR**: Never use "Co-Authored-By: Claude" or "Generated with Claude Code" in commit messages. This makes Claude appear as a contributor on GitHub.

### Pre-Commit Checklist
- [ ] Code compiles without errors (`cargo check --all-features`)
- [ ] Tests pass (`cargo test --all-features`)
- [ ] Read all changed files to verify correctness
- [ ] Human approval for commit message and changes
- [ ] Verify no sensitive information in commits
- [ ] **VERIFY NO CLAUDE CO-AUTHOR REFERENCES** in commit message

---

## 📦 v0.2.0 Feature Summary

### Major Additions
1. **Advanced Camera Controls** (`src/commands/advanced.rs`)
   - Manual focus, exposure, white balance
   - Plant photography optimization
   - HDR and focus stacking

2. **Performance Improvements** (`src/commands/capture.rs`)
   - Async-friendly locking with RwLock
   - Zero-copy memory management
   - Non-blocking file I/O

3. **Enhanced Type System** (`src/types.rs`)
   - CameraControls, BurstConfig, FrameMetadata
   - Professional camera capabilities detection
   - Comprehensive validation

4. **Plant Photography Demo** (`demo/plant-photography-studio.html`)
   - Professional camera interface
   - Real-time performance monitoring
   - Interactive control demonstration

### Testing
- **100+ comprehensive tests** in `src/tests/`
- **Performance benchmarks** for burst capture
- **Mock system** for hardware-independent testing
- **Edge case validation** for all user inputs

---

## 🌿 Plant Photography Specialization

### Why Plants?
- **High-value market**: Agricultural technology and botanical research
- **Technical requirements**: Deep DOF, color accuracy, macro capabilities
- **Differentiation**: First camera library optimized for botanical documentation

### Specialized Features
- **One-click optimization**: `optimize_for_plants()` command
- **Botanical settings**: f/8 aperture, enhanced greens, high contrast
- **Quality assessment**: Real-time sharpness and color analysis
- **Metadata capture**: Full botanical documentation support

---

## 🔧 Troubleshooting

### Common Issues
1. **Compilation Errors**: Always read files before editing
2. **Test Failures**: Check async/await patterns in new code
3. **Type Mismatches**: Verify CameraInitParams.controls structure
4. **Missing Metadata**: Ensure FrameMetadata::default() in CameraFrame

### Debug Commands
```powershell
# Detailed error output
RUST_BACKTRACE=1 cargo test

# Check specific feature compilation
cargo check --features contextlite

# Lint and format
cargo clippy --all-features
cargo fmt
```

---

## 📝 Documentation Standards

### Code Comments
- **No emojis in source code** (only in markdown files)
- **Clear function documentation** with examples
- **Error context** in all Result types
- **Performance notes** for optimization decisions

### API Documentation
- **Comprehensive examples** for all public functions
- **Platform compatibility** notes
- **Performance characteristics** descriptions
- **Error conditions** documentation

---

## 🎯 Next Steps (Post v0.2.0)

### Immediate Priorities
1. **Human review** of all v0.2.0 changes
2. **Git status check** and commit approval
3. **Demo testing** in multiple browsers
4. **Documentation review** for accuracy

### Future Development
1. **AI integration** for plant health analysis
2. **Cloud storage** for botanical databases
3. **Advanced processing** algorithms
4. **Platform-specific optimizations**

---

**Remember**: READ BEFORE WRITE. Get human approval before commits. Focus on plant photography excellence. 🦀🌿📷