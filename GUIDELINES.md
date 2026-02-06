# Project Guidelines: TUI Money Manager

## 1. Project Overview
**TUI Money** is a terminal-based personal finance manager written in Rust. It allows users to track expenses and incomes using a fast, keyboard-centric interface.

### Key Technologies
- **Language**: Rust (2024 edition)
- **UI Framework**: [Ratatui](https://github.com/ratatui-org/ratatui)
- **Database**: SQLite (via `rusqlite`)
- **Architecture**: Modular workspace (Clean Architecture inspired)

---

## 2. Workspace Structure & Architecture
The project is organized as a Cargo workspace with distinct crates to separate concerns:

### `crates/domain` (Core Business Logic)
- **Purpose**: Defines pure business entities and interfaces. Dependency-free (except for standard types).
- **Key Entities**:
  - `Entry`: Represents a financial transaction (Expense or Income).
  - `EntryKind`: Enum (`Expense`, `Income`).
  - `EntryFilter`: Struct for querying entries (date range, category).
- **Interfaces**:
  - `EntryRepository`: Trait defining storage operations (`add`, `list`).
- **Validation**: Enforced in `NewEntry::validate()` (e.g., amount must be positive).

### `crates/storage` (Persistence Layer)
- **Purpose**: Implements the `EntryRepository` trait using SQLite.
- **Repository**: `SqliteRepository`.
- **Database Schema**:
  - Table `entries`:
    - `id` (INTEGER PRIMARY KEY)
    - `kind` (TEXT: "expense" or "income")
    - `amount_cents` (INTEGER: monetary value in cents)
    - `category` (TEXT)
    - `note` (TEXT NULLable)
    - `occurred_on` (TEXT: ISO-8601 Date)
  - Table `schema_migrations`: Tracks applied migrations.
- **Migrations**: Embedded in binary (e.g., `001_init.sql`) and applied automatically on startup.

### `crates/ui` (Presentation Layer)
- **Purpose**: Handles rendering and user input.
- **Screens**:
  - `Login`: Authentication entry point (currently schematic).
  - `Dashboard`: Main view for viewing/adding entries.
  - `CreateUser`: Registration screen.
- **State Management**:
  - `App` struct manages the active screen (`ScreenId` enum).
  - `Action` enum defines UI events (`Quit`, `Go(ScreenId)`, `InputChar`, `Nav*`).
  - Event loop handles keyboard inputs via `crossterm`.

### `crates/app` (Application Composition)
- **Purpose**: The binary crate (`main.rs`).
- **Responsibility**: Wires the `SqliteRepository`, initializes the TUI `App`, and starts the event loop.

---

## 3. Data Flow
1. **User Action**: User types in the TUI (e.g., adds an expense).
2. **UI Layer**: `crates/ui` captures input, validates format, and invokes domain logic.
3. **Domain Layer**: `crates/domain` defines the `NewEntry` struct and validation rules.
4. **Storage Layer**: `crates/storage` converts `NewEntry` to SQL and persists it to `tui-money.db`.

---

## 4. Development Guidelines

### Build & Run
- **Run**: `cargo run -p tui-money`
- **Check**: `cargo check`
- **Test**: `cargo test` (Runs unit tests in `domain` and integration tests in `storage` using temp DBs).

### Code Style
- Follow standard Rust naming conventions (snake_case for functions/vars, PascalCase for types).
- **Formatting**: Run `cargo fmt` before committing.
- **Linting**: Run `cargo clippy` to ensure code quality.

### Database Handling
- The application creates a `tui-money.db` file in the working directory.
- **Do not commit** `tui-money.db`.
- Tests use temporary database files in the system temp directory to ensure isolation.

### Adding New Features
1. **Domain**: Define new types or methods in `crates/domain`.
2. **Storage**: Implement persistence in `crates/storage` (add migrations if needed).
3. **UI**: Create or update widgets/screens in `crates/ui` to expose the feature.
4. **Wiring**: Update `crates/app` if new dependencies or initialization logic is required.

---

## 5. Security Note
- Secrets and tokens should not be committed.
- Input sanitization logic resides in the `domain` layer (validation) and `storage` layer (parameterized SQL queries).

---

## 6. AI Agent Guidelines
**Mandatory Workflow**:
1. After **any** code modification.
2. Run `cargo clippy --fix --workspace --allow-dirty --allow-staged` to catch errors and apply automatic fixes.
3. Run `cargo fmt --all` to ensure formatting compliance.
4. Only proceed if both commands succeed.
