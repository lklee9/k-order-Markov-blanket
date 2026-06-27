# Project Setup
    
## Prerequisites

- Python 3.8+
- Rust toolchain (for LIMMB)

## Installation

1. **Pull the pyCausalFS submodule:**

   ```bash
   git submodule update --init --recursive
   ```

2. **Install Python dependencies:**

   ```bash
   pip install -r experiments/requirements.txt
   ```

3. **Build the LIMMB Rust extension:**

   The LIMMB package is a local Rust extension and will be built automatically when installed via the `-e` flag in requirements.txt. If you encounter issues, you can build it manually:

   ```bash
   cd LIMMB/
   maturin develop
   cd ..
   ```
