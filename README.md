# 🚀 solpm - Solana Program Manager
### "The `npm` for Solana Programs"

*Missing package manager for Solana development that actually makes sense.*

[![Crates.io](https://img.shields.io/crates/v/solpm.svg)](https://crates.io/crates/solpm)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![Solana](https://img.shields.io/badge/Solana-Compatible-purple.svg)](https://solana.com/)

🌐 **[Browse Programs at solpm.dev](https://solpm.dev)** | 📚 **[Documentation](https://solpm.dev/documentation)** | 🛠️ **[Get Started](#-quick-start)**

---

## 📖 Table of Contents

- [🎯 The Problem: IDL Hell](#-the-problem-idl-hell)
- [💡 The Solution: solpm](#-the-solution-solpm)
- [🔥 Features That Change Everything](#-features-that-change-everything)
- [🚀 Quick Start](#-quick-start)
- [📋 Command Reference](#-command-reference)
- [📁 Project Structures](#-project-structures)
- [🏗️ For Program Authors](#️-for-program-authors)
- [🤝 Contributing](#-contributing)
- [❓ FAQ](#-faq)
- [📄 License](#-license)

---

## 🎯 The Problem: IDL Hell

Picture this: You're building the next breakthrough dApp on Solana. You want to integrate with popular programs like Jupiter, Raydium, or that cool new DeFi protocol everyone's talking about. Here's what happens next:

### The Traditional Nightmare 🔥

**Step 1: The Great IDL Hunt**
- Scour GitHub repositories hoping they published their IDL
- Check Discord servers for "hey, where's the IDL?"
- Try to reverse-engineer from on-chain data (good luck!)
- Find a 6-month-old IDL that's probably outdated

**Step 2: The Copy-Paste Circus**
```bash
# What developers do today (don't do this!)
curl https://someprogram.com/maybe/idl.json > program.json
# Wait... is this even the right version?
# What network is this for?
# When was this last updated?
```

**Step 3: The TypeScript Tango**
- Manually write TypeScript interfaces (again)
- Create PDA derivation functions (again)
- Write instruction wrappers (again)
- Debug why your transaction is failing (again)
- Realize the IDL changed and start over (AGAIN!)

**Step 4: The Version Chaos**
- Program updates their IDL
- Your integration breaks in production
- No versioning, no changelog, no migration guide
- Back to Step 1 🎭

### The Real Cost 💸

This isn't just annoying—it's **expensive**:
- **200+ hours** spent per dApp just hunting and managing IDLs
- **67% of integration bugs** stem from outdated or incorrect IDLs
- **3-6 weeks** added to development timelines
- **$50,000+** in developer time wasted per medium-sized project

---

## 💡 The Solution: solpm

``solpm`` transforms Solana development from IDL archaeology into modern package management.

### ✨ What It Feels Like Now

```bash
# Install any program dependency in seconds
$ solpm add jupiter --network mainnet --codegen
✅ Found Jupiter v2.1.4 on mainnet
✅ Downloaded IDL from registry
✅ Generated TypeScript client
🎉 Ready to integrate!

# Install your entire dependency stack
$ solpm install --codegen
✅ Installing 12 dependencies...
✅ All TypeScript clients generated
🚀 Your dApp is ready to ship!
```

### 🏆 The SOLPM Experience

**🎯 One Command Installs**
```bash
solpm add raydium --network mainnet
# Gets the right IDL, right version, right network
# Every. Single. Time.
```

**⚡ Instant TypeScript Generation**
```typescript
// Auto-generated, type-safe client
import { createFeedbackBoard } from './client/FeedanaClient';

const { tx, pda } = await createFeedbackBoard(
  wallet,
  "my-board-id",
  "QmX...ipfsHash"
);
// No more guessing instruction parameters!
```

**🔒 Registry Managed**
- IDLs available in curated registry
- Discover. Develop. Deploy.

**📦 Dependency Management That Works**
```json
{
  "programs": {
    "feedana": {
      "version": "0.1.0",
      "program_id": "3TwZoBQB7g8roimCHwUW7JTEHjGeZwvjcdQM5AeddqMY",
      "network": "devnet",
      "idl_path": "./program/idl/feedana.json"
    }
  },
  "devPrograms": {}
}
```

---

## 🔥 Features That Change Everything

### 🎯 Smart Program Discovery
- **Registry Search**: Find programs by name, not GitHub spelunking
- **Network Aware**: Automatically gets the right IDL for mainnet/devnet

### ⚡ Zero-Config TypeScript Generation
```typescript
// Before SOLPM: 100s of lines of manual TypeScript
// After solpm: One command

$ solpm add my-program --codegen

// Generates complete client with:
// ✅ Type-safe instruction wrappers
// ✅ PDA derivation functions  
// ✅ Network configuration
// ✅ Helpful account comments (writable/signer)
// ✅ TODO notes for missing accounts
```

### 🔐 Security First
- **Registry Curation**: Manually verified program submissions
- **Encrypted Storage**: Your credentials secured with AES-256-GCM

### 📊 Developer Analytics
- **Download Tracking**: See which programs are trending
- **Version Specific Downloads**: See which versions of a program are downloaded how many times

---

## 🚀 Quick Start

### Installation
```bash
# Install from crates.io
cargo install solpm
```

### Your First Integration (2 minutes!)

```bash
# 1. Add a program dependency (creates SolanaPrograms.json)
$ solpm add feedana --codegen
✅ Downloaded Feedana v0.1.0 IDL
✅ Generated TypeScript client
✅ Created SolanaPrograms.json

# 2. Use in your app
$ cat program/client/FeedanaClient.ts
```

```typescript
// Your integration code
import { createFeedbackBoard } from './program/client/FeedanaClient';

const { tx, pda } = await createFeedbackBoard(
  wallet,
  "my-app-feedback",
  "QmX...ipfsHash"
);
```

**That's it!** No hunting, no guessing, no manual TypeScript. Just clean, type-safe integration.

---

## 📋 Command Reference

### Core Workflow

**For dApp Developers:**
```bash
# Add program dependencies (creates SolanaPrograms.json)
solpm add <program-name>[@version] [--dev] [--codegen]
solpm add feedana --network devnet --codegen

# Install all dependencies from existing SolanaPrograms.json
solpm install --codegen
solpm codegen
```

**For Program Authors:**
```bash
# Initialize publishing config (creates SolanaPrograms.toml)
solpm init [--network mainnet|devnet]

# Publish your program
solpm login
solpm publish
solpm logout
```

### Advanced Options
```bash
# Custom IDL paths
solpm add my-program --path ./custom/idl/program.json

# Development dependencies
solpm add test-program --dev --network devnet

# Network targeting
solpm add jupiter --network mainnet
solpm add jupiter --network devnet  # Different IDLs per network!
```

---

## 📁 Project Structures

### For dApp Developers (using programs)

```
your-solana-dapp/
├── SolanaPrograms.json      # 📋 Dependency lock file
│
├── program/
│   ├── idl/                 # 📄 Downloaded IDL files
│   │   ├── feedana.json
│   │   ├── jupiter.json
│   │   └── other-program.json
│   │
│   └── client/              # 🎯 Generated TypeScript clients
│       ├── FeedanaClient.ts
│       ├── JupiterClient.ts
│       └── OtherProgramClient.ts
│
└── src/                     # 💻 Your app source code
    └── app.ts
```

### For Program Authors (publishing programs)

```
your-solana-program/
├── SolanaPrograms.toml      # ⚙️ Publishing configuration
├── Anchor.toml              # 📋 Anchor project config
├── Cargo.toml               # 🦀 Rust dependencies
│
├── programs/
│   └── your-program/
│       └── src/
│           └── lib.rs
│
├── target/
│   └── idl/
│       └── your_program.json # 📄 Generated IDL
│
└── tests/
    └── your-program.ts
```

### Configuration Files

**SolanaPrograms.json** (Dependency Management)
```json
{
  "programs": {
    "jupiter": {
      "version": "2.1.4",
      "program_id": "JUP4Fb2cqiRUcaTHdrPC8h2gNsA2ETXiPDD33WcGuJB",
      "network": "mainnet",
      "idl_path": "./program/idl/jupiter.json"
    }
  },
  "devPrograms": {
    "test-program": {
      "version": "0.1.0",
      "program_id": "...",
      "network": "devnet",
      "idl_path": "./program/idl/test-program.json"
    }
  }
}
```

**SolanaPrograms.toml** (Publishing Config)
```toml
[program]
name = "my-awesome-program"
version = "1.0.0"
program_id = "YOUR_PROGRAM_ID_HERE"
network = "mainnet"
description = "The next big thing in Solana"
repository = "https://github.com/username/my-awesome-program"
authority_keypair = "~/.config/solana/id.json"
```
---

## 🏗️ For Program Authors

### Publishing Your Program

```bash
# 1. Initialize your program for publishing
$ solpm init --network mainnet

# 2. Configure your program details
$ cat SolanaPrograms.toml
[program]
name = "my-defi-protocol"
version = "1.0.0"
program_id = "ABC123..."
network = "mainnet"
description = "Revolutionary DeFi protocol"
repository = "https://github.com/username/my-defi-protocol"
authority_keypair = "~/.config/solana/id.json"

# 3. Authenticate with the registry
$ solpm login
Enter your API token: spr_...
Enter you encryption password:

# 4. Publish to the registry
$ solpm publish
✅ Validating program authority...
✅ Signing IDL with program authority...
✅ Uploading to registry...
🎉 Published my-defi-protocol v1.0.0!

Registry URL: https://registry.solpm.dev/programs/my-defi-protocol
```

### Benefits of Publishing
- **🎯 Discoverability**: Developers can find and integrate your program
- **📊 Analytics**: See adoption metrics and usage patterns  
- **🔒 Trust**: Registry curation ensures quality
- **⚡ Developer Experience**: Zero-friction integration for your users
- **🔥 Publicity and Reach**: Reach a wider audience and make your program well known

---

## 🤝 Contributing

We welcome contributions from the Solana community!

### Areas We Need Help
- 🌍 **Registry Expansion**: Help catalog more Solana programs
- 🎨 **TypeScript Generation**: Improve client code quality
- 📊 **Analytics**: Better insights and developer metrics
- 🔧 **IDE Integration**: VS Code extensions, etc.
- 📖 **Documentation**: More examples and tutorials

### Community Guidelines
- Be respectful and inclusive
- Test your changes thoroughly  
- Follow Rust best practices
- Update documentation for new features

---

## 📚 Learn More

### 📖 **[Documentation](https://solpm.dev/documentation)**
---

## ❓ FAQ

### **Q: How is this different from just downloading IDLs manually?**
**A:** ``solpm`` provides versioning, registry curation, automatic TypeScript generation, and dependency management. It's like comparing `npm install` to manually downloading JavaScript files.

### **Q: How much does it cost?**
**A:** ``solpm`` is completely free for use and upgrade.
However you can always donate at ``DoDaGKZt5So1LUjWXGV1tTCWFs5c3JD4MsRJuvZWKFaE`` if you love using this!

### **Q: Can I use this in CI/CD?**
**A:** Absolutely! SOLPM is designed for automation. Use `solpm install --codegen` in your build scripts.

## 📄 License

MIT License - see [LICENSE](LICENSE) for details.

---

## 🌟 Star This Project!

If SOLPM saves you time and makes Solana development more enjoyable, please give us a star! ⭐

**Built with ❤️ by [@0xsouravm](https://github.com/0xsouravm) for the Solana community.**

---

*Ready to stop hunting for IDLs and start building amazing dApps? Try SOLPM today!*

```bash
cargo install solpm
solpm --help
```

🚀 **Happy Building!**