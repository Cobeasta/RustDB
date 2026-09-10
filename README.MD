
Rust Database

A database implementation written in Rust.

This project uses Fedora Toolbx to isolate the development environment and rustup to manage the Rust toolchain.

Development Environment

The development setup consists of:

Fedora — Host operating system
Toolbx — Isolated development container
Podman — Container runtime used by Toolbx
rustup — Rust toolchain manager
Cargo — Rust package manager and build system
rustc — Rust compiler

The project source code remains in your normal home directory while the development tools live inside the Toolbx environment.

Initial Setup
1. Install Toolbx

Run this on the Fedora host:

sudo dnf install toolbox

What it does

Installs Toolbx, which lets you create persistent development containers.

2. Create the Rust development environment
toolbox create rust-db

What it does

Creates a Toolbx container named rust-db.

You only need to run this command once.

3. Enter the development environment
toolbox enter rust-db

What it does

Opens a shell inside the rust-db Toolbx container.

You'll use this whenever you want to work on the project.

Development Dependencies

Inside the Toolbx container:

sudo dnf install \
    git \
    gcc \
    pkg-config

Packages
git

Version control.

git --version


Used to track changes to the project.

gcc

C compiler.

Rust projects can depend on native C libraries or crates containing C code.

You may not directly use GCC, but some Rust dependencies can require it.

pkg-config

Helps software find native libraries installed on the system.

For example, a Rust crate might need to locate OpenSSL or another system library.

Installing Rust
Install rustup

Inside the Toolbx container:

sudo dnf install rustup


rustup manages Rust versions and toolchains.

Initialize rustup
rustup-init


Follow the prompts and choose the default installation.

Verify Rust
rustc --version


Shows the installed Rust compiler version.

Example:

rustc 1.xx.x

Check Cargo
cargo --version


Shows the installed Cargo version.

Check rustup
rustup show


Shows the installed Rust toolchains and which one is currently active.

Creating the Project

Create a source directory:

mkdir -p ~/src


Move into it:

cd ~/src


Create the Rust project:

cargo new rustdb


Enter the project:

cd rustdb


The initial project looks like:

rustdb/
├── Cargo.toml
└── src/
    └── main.rs

Cargo Commands

Cargo is Rust's build system and package manager.

cargo check
cargo check


Checks whether the project compiles without producing the final executable.

It is faster than a full build.

Use it when

You're actively writing code and want a quick compilation check.

cargo check


Think:

"Did I break the code?"

cargo build
cargo build


Compiles the project.

The debug executable is placed under:

target/debug/

cargo build --release
cargo build --release


Builds an optimized release version.

The executable is placed under:

target/release/


Use this when you want to benchmark or run the database in a more production-like configuration.

cargo run
cargo run


Builds the project if necessary and runs the resulting executable.

For example:

cargo run


might produce:

Hello, world!


You can also pass arguments to your program:

cargo run -- database.db


The -- separates Cargo's arguments from your program's arguments.

Formatting
cargo fmt
cargo fmt


Automatically formats Rust source code according to the standard Rust formatting rules.

For example:

fn main(){let x=5;println!("{}",x);}


becomes:

fn main() {
    let x = 5;
    println!("{}", x);
}


Think:

"Make my code clean and consistently formatted."

Linting
cargo clippy
cargo clippy


Runs Clippy, Rust's linter.

Clippy looks for:

suspicious code
unnecessary complexity
common mistakes
inefficient patterns
non-idiomatic Rust
opportunities to improve the code

For example, it might suggest replacing a verbose piece of code with a simpler Rust idiom.

Think:

"Is there a better or safer way to write this?"

Testing
cargo test
cargo test


Compiles and runs the project's automated tests.

A test can look like:

#[test]
fn test_something() {
    assert_eq!(2 + 2, 4);
}


Running:

cargo test


will report whether the test passed.

For a database, tests will eventually cover things such as:

Storage
├── writing pages
├── reading pages
└── handling invalid pages

B-Tree
├── inserting keys
├── finding keys
├── deleting keys
└── splitting nodes

Transactions
├── commit
├── rollback
└── recovery


Think:

"Does my code actually work?"

Recommended Development Check

Before committing a significant change:

cargo fmt
cargo check
cargo test
cargo clippy


Or run them together:

cargo fmt && cargo check && cargo test && cargo clippy


If all commands succeed, the code is formatted, compiles, passes tests, and has no Clippy errors.

Rust Toolchains

rustup allows different projects to use different Rust versions.

Install the stable toolchain:

rustup toolchain install stable


Use stable for the current project:

rustup override set stable


The override applies to the current project directory.

Check the active toolchain:

rustup show

rust-toolchain.toml

A better way to pin the project's toolchain is to create:

rust-toolchain.toml


with:

[toolchain]
channel = "stable"
components = ["rustfmt", "clippy"]


Now Rust automatically uses the specified toolchain when working inside the project.

This also ensures that rustfmt and clippy are available.

Useful Rustup Commands
List installed toolchains
rustup toolchain list

Install a toolchain
rustup toolchain install stable

Update Rust
rustup update


Updates installed Rust toolchains.

Show active configuration
rustup show

Toolbx Commands
List Toolbx environments
toolbox list


Shows your existing Toolbx containers.

Enter the Rust environment
toolbox enter rust-db

Leave the Rust environment

Inside the container:

exit


This returns you to the normal Fedora shell.

It does not delete the container.

Stop the container

From the Fedora host:

podman stop rust-db


This stops the underlying container.

Your files remain intact.

Start it again
podman start rust-db


Then enter it:

toolbox enter rust-db

Remove the development environment
toolbox rm rust-db


This deletes the Toolbx environment and the software installed inside it.

Your project files in your normal home directory, such as:

~/src/rustdb


remain on the host.

Typical Workflow

Start working:

toolbox enter rust-db


Go to the project:

cd ~/src/rustdb


Check your code:

cargo check


Run tests:

cargo test


Format the code:

cargo fmt


Run Clippy:

cargo clippy


Run the database:

cargo run


Build an optimized version:

cargo build --release


When finished:

exit

Quick Reference
Command	Purpose
toolbox create rust-db	Create development environment
toolbox enter rust-db	Enter development environment
toolbox list	List Toolbx environments
exit	Leave Toolbx
podman stop rust-db	Stop the container
podman start rust-db	Start the container
toolbox rm rust-db	Delete the development environment
rustc --version	Show Rust compiler version
cargo --version	Show Cargo version
rustup show	Show active Rust configuration
rustup update	Update Rust
cargo new rustdb	Create a new Rust project
cargo check	Quickly check compilation
cargo build	Build debug version
cargo build --release	Build optimized version
cargo run	Build and run
cargo test	Run tests
cargo fmt	Format code
cargo clippy	Run Rust linter
rustup toolchain list	List Rust toolchains
rustup toolchain install stable	Install stable Rust
rustup override set stable	Set project-specific toolchain
Recommended Daily Commands

For normal development:

toolbox enter rust-db
cd ~/src/rustdb
cargo check
cargo test
cargo run


Before committing:

cargo fmt
cargo check
cargo test
cargo clippy


For a release build:

cargo build --release

