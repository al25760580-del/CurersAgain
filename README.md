# CurersAgain

 > HoloCure reimplementation. Trust me bro, 100% real.

 CurersAgain is an independent reimplementation of **HoloCure** written in Rust.

 The goal is not to reproduce the original source code, but to recreate the behavior of the game and its underlying runtime as accurately as possible, while providing a clean, understandable and highly moddable codebase.

 No weird DLLs. No praying to Aurie. Just Rust.

 ## What is this?

 CurersAgain is being built through reverse engineering and behavioral analysis of the original game, including:

 - x86 disassembly and decompilation
- Runtime analysis
- Debugging and instrumentation
- Comparing observed behavior against the reimplementation
- Reimplementing game and engine functionality in Rust

 The result is intended to be a standalone implementation that users can understand, modify and extend without having to rely on external injection frameworks or opaque runtime patches.

 ## Modding

 One of the main goals of CurersAgain is to make modding as straightforward as possible.

 Ideally, creating a mod should be as simple as putting it in a folder and writing understandable code. The project should expose the systems that mods actually need instead of requiring users to reverse engineer the reimplementation themselves.

 The codebase is therefore intended to be:

 - Readable
- Extensible
- Well documented where behavior is non-obvious
- Friendly to mod authors
- Free from unnecessary dependencies on the original runtime

 ## Found Something Wrong?

 If you find behavior that is:

 - Not implemented yet
- Different from the original game
- Only partially implemented
- Implemented but behaving differently in edge cases

 **Please open an issue.**

 When possible, include:

 1. **What the original game does**
2. **What CurersAgain does instead**
3. Steps to reproduce the behavior
4. The relevant function or system
5. A decompiled/disassembled snippet showing the original behavior, if you have one
6. Any additional observations that might help reproduce it

 A small example is much more useful than simply saying that something is "wrong".

 ### Example

```
Original:
    When X happens, Y is incremented before Z is called.

CurersAgain:
    Y is incremented after Z.

Relevant function:
    <function name / address if known>

Decompilation:
    <snippet>

Expected behavior:
    Y should already contain the incremented value when Z executes.
```

 You don't need to be certain about your analysis. **An incomplete but reproducible observation is still useful.**

 ## AI-Assisted Development

 **AI-assisted development is allowed and encouraged.**

 This project prioritizes getting behavioral parity with the original game quickly. AI can be useful for accelerating the initial port, translating decompiled logic, exploring unfamiliar systems, generating tests, and filling in missing implementations.

 That does **not** mean generated code is expected to be high quality.

 The general workflow is:

 > **Get it working → verify behavior → understand it → rewrite it properly.**

 AI-generated or heavily assisted code may therefore exist in intermediate stages of the project. Code quality, architecture and maintainability can be improved after the behavior has been validated against the original.

 Please don't avoid submitting an issue or PR because you think the existing implementation is "too messy to touch". If you can help establish what the original does, that's already valuable.

 ## Accuracy

 CurersAgain aims for **behavioral compatibility**, not source-level equivalence.

 The original implementation and CurersAgain may use completely different internal structures while still producing the same observable behavior.

 When behavior differs, the original game is the reference.

 Some systems may initially be approximations until enough information is available to reproduce their behavior accurately.

 ## Contributing

 Contributions are welcome, especially:

 - Reverse engineering findings
- Behavioral documentation
- Missing implementations
- Tests comparing original and reimplemented behavior
- Bug fixes
- Engine improvements
- Modding APIs
- Code cleanup and refactoring

 If you reverse engineer a function or system and discover something useful, **please document it**, even if you don't have time to implement it yourself.

 A good observation can save someone else hours of staring at assembly.

 ## Disclaimer

 CurersAgain is an independent project and is not affiliated with or endorsed by the original developers of HoloCure.

 The project is intended as a clean-room-style reimplementation based on observed behavior and reverse engineering research.

 Do not submit proprietary source code or other material that you are not permitted to redistribute.

---

 **CurersAgain**

 _We read the assembly so you don't have to._
