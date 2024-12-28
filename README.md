# **Embedded Recruitment Task Deliverables**

## Deliverables

1. Updated Server Implementation ---> src/server.rs
   - Fully functional server that adheres to the multithreading requirements.
   - Inline comments in the code to explain significant changes.
2. Test Suite Results ---> Test_Evidence.pdf
   - Evidence (e.g., logs) that your server passes all tests.
3. A brief document outlining: ---> Architectural_Flaws.pdf
   - The identified bugs in the initial implementation.
   - How architectural flaws were addressed.
   
## **Repository Structure**
```plaintext
.
|── proto/
│   └── messages.proto        # IDL with messages server handle.
├── src/
│   ├── server.rs             # Updated server implementation (Multithreaded and robust).
│   └── lib.rs                # Core server logic.
├── tests/
│   ├── client.rs             # Client implementation.
│   └── client_test.rs        # Client test suite (Modified).
├── .gitignore
├── Architectural_Flaws.pdf   # A brief document outlining:
│                               - The identified bugs in the initial implementation.
│                               - How architectural flaws were addressed.
├── build.rs                  # Build script for compiling the Proto file.
├── Cargo.toml                # Rust dependencies and configuration (Modified).
├── README.md                 # Task deliverables instructions.
└── Test_Evidence.pdf         # Test suite results.
```