# Developer Guide

Developer guide useful for onboarding as a contributor of project.

## Project Layout

```text
.
├── configs             # Configuration files (e.g. network parameter files).
├── docs                # User & developer documentation.
├── scripts             # Custom python/bash scripts used for testing & development.
├── src
│   ├── constants.rs    # Hard-coded constants used throughout project.
│   ├── eoa.rs          # Definition and logic of Externally Owned Accounts (EOA) object used for testing.
│   ├── errors.rs       # Custom network errors type definition.
│   ├── kurtosis.rs     # Kurtosis specific internal utility functions.
│   ├── network.rs      # Implementation of test network object.
│   ├── types.rs        # Custom type definitions.
│   └── utils.rs        # General utility project functions (internal and external).
└── tests               # Integration & E2E tests of crate from user POV.
```
