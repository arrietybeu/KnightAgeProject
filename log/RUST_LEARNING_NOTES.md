# RUST LEARNING NOTES - Knight Age Development Journey

> Ghi chú học Rust trong quá trình phát triển Knight Age Server. Tổng hợp kiến thức, patterns, và bài học đã học được.

---

## 📚 MỤC LỤC

- [Rust Fundamentals](#rust-fundamentals)
- [Ownership & Borrowing](#ownership--borrowing)
- [Smart Pointers](#smart-pointers)
- [Async Programming](#async-programming)
- [Error Handling](#error-handling)
- [Pattern Matching](#pattern-matching)
- [Traits & Generics](#traits--generics)
- [Module System](#module-system)
- [Common Patterns](#common-patterns)
- [Pitfalls & Solutions](#pitfalls--solutions)

---

## 🎓 RUST FUNDAMENTALS

### Variables & Mutability

```rust
// Immutable by default
let x = 5;
// x = 6;  // ❌ ERROR: cannot assign twice to immutable variable

// Mutable variable
let mut y = 5;
y = 6;  // ✅ OK

// Constants (always immutable, must have type)
const MAX_PLAYERS: u32 = 1000;
```

**Lesson**: Rust forces you to be explicit about mutability. This prevents bugs.

---

### Data Types

```rust
// Integers
let small: i8 = -127;        // 8-bit signed
let medium: i32 = 1000;      // 32-bit signed (default)
let large: i64 = 1000000;    // 64-bit signed

let unsigned: u8 = 255;      // 8-bit unsigned
let big: u64 = 18446744073709551615;  // 64-bit unsigned

// Floating point
let f32_val: f32 = 3.14;
let f64_val: f64 = 3.141592653589793;  // default

// Boolean
let is_alive: bool = true;

// Character (4 bytes, Unicode)
let c: char = '😀';

// String types
let string_slice: &str = "Hello";        // String slice (borrowed)
let owned_string: String = String::from("World");  // Owned string
```

**Trong Knight Age**:
- `i8` cho command codes (-128 to 127)
- `u8` cho bytes
- `i16`, `i32`, `i64` cho game values
- `String` cho player names, messages

---

### Collections

```rust
// Vec<T> - Growable array
let mut vec = Vec::new();
vec.push(1);
vec.push(2);
let v = vec![1, 2, 3];  // Macro

// HashMap<K, V> - Key-value store
use std::collections::HashMap;
let mut map = HashMap::new();
map.insert("key", "value");

// Access
let value = map.get("key");  // Returns Option<&V>
```

**Trong Knight Age**:
```rust
// Session storage
HashMap<u64, SessionData>

// Packet handlers
HashMap<i8, Arc<dyn PacketHandler>>

// Player inventory
Vec<Item>
```

---

## 🔑 OWNERSHIP & BORROWING

### Ownership Rules

1. **Each value has an owner**
2. **Only one owner at a time**
3. **Value is dropped when owner goes out of scope**

```rust
fn main() {
    let s1 = String::from("hello");
    let s2 = s1;  // s1 moved to s2
    
    // println!("{}", s1);  // ❌ ERROR: value borrowed after move
    println!("{}", s2);  // ✅ OK
}
```

**Solution: Clone or Borrow**

```rust
// Clone (copy data)
let s1 = String::from("hello");
let s2 = s1.clone();
println!("{} {}", s1, s2);  // ✅ Both valid

// Borrow (reference)
let s1 = String::from("hello");
let s2 = &s1;  // Immutable borrow
println!("{} {}", s1, s2);  // ✅ Both valid
```

---

### Borrowing Rules

1. **Any number of immutable references** OR **one mutable reference**
2. **References must be valid** (no dangling references)

```rust
// Multiple immutable borrows ✅
let s = String::from("hello");
let r1 = &s;
let r2 = &s;
println!("{} {}", r1, r2);

// Mutable borrow ✅
let mut s = String::from("hello");
let r1 = &mut s;
r1.push_str(" world");

// ERROR: Cannot have multiple mutable borrows ❌
let mut s = String::from("hello");
let r1 = &mut s;
let r2 &mut s;  // ❌ ERROR
```

**Trong Knight Age**:
```rust
// Immutable borrow to read
async fn handle(&self, packet: &Packet, ctx: &PacketContext) { }

// Mutable borrow to modify
async fn write_packet(&mut self, packet: &Packet) -> io::Result<()> { }
```

---

### Lifetimes

```rust
// Lifetime annotation
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}

// Struct with lifetimes
struct ImportantExcerpt<'a> {
    part: &'a str,
}

// Often elided by compiler
fn first_word(s: &str) -> &str {
    // Same as: fn first_word<'a>(s: &'a str) -> &'a str
    &s[..1]
}
```

**Trong Knight Age**: Hầu hết lifetimes được compiler tự suy ra.

---

## 🧩 SMART POINTERS

### Box<T> - Heap Allocation

```rust
// Stack data
let x = 5;

// Heap data
let b = Box::new(5);

// Recursive type (requires Box)
enum List {
    Cons(i32, Box<List>),
    Nil,
}
```

**Khi nào dùng**:
- Recursive types
- Large data (avoid stack overflow)
- Transfer ownership (trait objects)

---

### Rc<T> - Reference Counting (Single-threaded)

```rust
use std::rc::Rc;

let a = Rc::new(String::from("hello"));
let b = Rc::clone(&a);  // Increment count
let c = Rc::clone(&a);

println!("Count: {}", Rc::strong_count(&a));  // 3
```

**⚠️ NOT thread-safe!** Use `Arc<T>` for multi-threading.

---

### Arc<T> - Atomic Reference Counting (Thread-safe)

```rust
use std::sync::Arc;
use std::thread;

let data = Arc::new(vec![1, 2, 3]);

let data1 = Arc::clone(&data);
let handle1 = thread::spawn(move || {
    println!("{:?}", data1);
});

let data2 = Arc::clone(&data);
let handle2 = thread::spawn(move || {
    println!("{:?}", data2);
});

handle1.join().unwrap();
handle2.join().unwrap();
```

**Trong Knight Age**:
```rust
// Shared across threads
Arc<SessionManager>
Arc<PacketRegistry>

// Each session has Arc to shared writer
Arc<Mutex<ConnectionWriter>>
```

---

### RefCell<T> / Mutex<T> - Interior Mutability

```rust
// Single-threaded: RefCell<T>
use std::cell::RefCell;

let data = RefCell::new(5);
*data.borrow_mut() += 1;  // Runtime borrow check

// Multi-threaded: Mutex<T>
use std::sync::Mutex;

let data = Mutex::new(5);
{
    let mut num = data.lock().unwrap();
    *num += 1;
}  // Lock automatically released
```

**Trong Knight Age**:
```rust
// Thread-safe mutable access
Arc<Mutex<ConnectionWriter>>
Arc<RwLock<HashMap<...>>>
```

---

### RwLock<T> - Read-Write Lock

```rust
use std::sync::RwLock;

let lock = RwLock::new(5);

// Multiple readers
{
    let r1 = lock.read().unwrap();
    let r2 = lock.read().unwrap();
    println!("{} {}", *r1, *r2);
}

// Single writer
{
    let mut w = lock.write().unwrap();
    *w += 1;
}
```

**Performance**: Better than `Mutex` when reads >> writes

**Trong Knight Age**:
```rust
// SessionManager: Many reads, few writes
RwLock<HashMap<u64, SessionData>>

// ConnectionState
Arc<RwLock<ConnectionState>>
```

---

## ⚡ ASYNC PROGRAMMING

### Tokio Runtime

```rust
// Main entry point
#[tokio::main]
async fn main() {
    // Async code here
}

// Equivalent to:
fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        // Async code
    });
}
```

---

### Async/Await Syntax

```rust
// Async function
async fn fetch_data() -> Result<String, Error> {
    let data = some_async_operation().await?;
    Ok(data)
}

// Calling async function
async fn main_async() {
    let result = fetch_data().await;
    match result {
        Ok(data) => println!("{}", data),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

**Key points**:
- `async fn` returns `Future`
- `.await` waits for future to complete
- Must be called from async context

---

### Spawning Tasks

```rust
use tokio::task;

#[tokio::main]
async fn main() {
    let handle = tokio::spawn(async {
        // This runs on a separate task
        println!("Hello from spawned task");
    });
    
    // Wait for task to complete
    handle.await.unwrap();
}
```

**Trong Knight Age**:
```rust
// Spawn session handler
tokio::spawn(async move {
    if let Err(e) = session.run().await {
        eprintln!("Session error: {}", e);
    }
});
```

---

### Channels - Communication Between Tasks

```rust
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(100);
    
    // Sender task
    tokio::spawn(async move {
        tx.send("hello").await.unwrap();
    });
    
    // Receiver
    while let Some(msg) = rx.recv().await {
        println!("Received: {}", msg);
    }
}
```

**Trong Knight Age**:
```rust
// Worker pool channel
let (sender, receiver) = mpsc::channel::<WorkerTask>(10000);

// Submit task
sender.send(task).await?;

// Receive task
let task = receiver.recv().await;
```

---

### Select - Wait for Multiple Futures

```rust
use tokio::select;

#[tokio::main]
async fn main() {
    let mut interval = tokio::time::interval(Duration::from_secs(1));
    let (tx, mut rx) = mpsc::channel(10);
    
    loop {
        select! {
            _ = interval.tick() => {
                println!("Tick");
            }
            msg = rx.recv() => {
                if let Some(msg) = msg {
                    println!("Received: {}", msg);
                }
            }
        }
    }
}
```

---

### Common Async I/O

```rust
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

async fn example() -> io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:8080").await?;
    
    // Write
    stream.write_all(b"hello").await?;
    
    // Read
    let mut buf = [0; 1024];
    let n = stream.read(&mut buf).await?;
    
    Ok(())
}
```

**Trong Knight Age**:
```rust
// BufReader/BufWriter
use tokio::io::{BufReader, BufWriter};

let reader = BufReader::new(read_half);
let writer = BufWriter::new(write_half);

// Read byte
let byte = reader.read_u8().await?;

// Write byte
writer.write_u8(byte).await?;
writer.flush().await?;
```

---

## 🛡️ ERROR HANDLING

### Result<T, E>

```rust
// Function that can fail
fn divide(a: i32, b: i32) -> Result<i32, String> {
    if b == 0 {
        Err(String::from("Division by zero"))
    } else {
        Ok(a / b)
    }
}

// Using Result
match divide(10, 2) {
    Ok(result) => println!("Result: {}", result),
    Err(e) => eprintln!("Error: {}", e),
}
```

---

### The ? Operator

```rust
use std::fs::File;
use std::io::{self, Read};

fn read_file() -> io::Result<String> {
    let mut file = File::open("file.txt")?;  // Returns early if error
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

// Equivalent to:
fn read_file_manual() -> io::Result<String> {
    let mut file = match File::open("file.txt") {
        Ok(f) => f,
        Err(e) => return Err(e),
    };
    // ... similar for read_to_string
}
```

**Lesson**: `?` makes error handling concise and readable.

---

### Option<T>

```rust
fn find_user(id: u64) -> Option<User> {
    // Return Some(user) if found, None if not
}

// Using Option
match find_user(123) {
    Some(user) => println!("Found: {}", user.name),
    None => println!("User not found"),
}

// Or with if let
if let Some(user) = find_user(123) {
    println!("Found: {}", user.name);
}
```

---

### Custom Error Types

```rust
use std::fmt;

#[derive(Debug)]
enum MyError {
    IoError(io::Error),
    ParseError(String),
    NotFound,
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MyError::IoError(e) => write!(f, "IO error: {}", e),
            MyError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            MyError::NotFound => write!(f, "Not found"),
        }
    }
}

impl std::error::Error for MyError {}

// Convert from io::Error
impl From<io::Error> for MyError {
    fn from(err: io::Error) -> Self {
        MyError::IoError(err)
    }
}
```

**Trong Knight Age**:
```rust
// PacketHandlerResult
pub enum PacketHandlerResult {
    Ok,
    CloseConnection,
    Error(String),
}
```

---

## 🎯 PATTERN MATCHING

### Match Expression

```rust
let number = 7;

match number {
    1 => println!("One"),
    2 | 3 | 5 | 7 | 11 => println!("Prime"),
    13..=19 => println!("Teen"),
    _ => println!("Other"),
}
```

**Trong Knight Age**:
```rust
match msg.command {
    1 => handle_login(msg),
    4 => handle_move(msg),
    9 => handle_fire_monster(msg),
    _ => {} // Unhandled
}
```

---

### If Let

```rust
let some_option = Some(5);

// Instead of:
match some_option {
    Some(val) => println!("{}", val),
    None => {}
}

// Use:
if let Some(val) = some_option {
    println!("{}", val);
}
```

---

### While Let

```rust
let mut stack = vec![1, 2, 3];

while let Some(top) = stack.pop() {
    println!("{}", top);
}
```

**Trong Knight Age**:
```rust
// Worker loop
while let Some(task) = receiver.recv().await {
    process_task(task).await;
}
```

---

## 🧬 TRAITS & GENERICS

### Traits - Interface Definition

```rust
trait Summary {
    fn summarize(&self) -> String;
    
    // Default implementation
    fn read(&self) -> String {
        format!("Read more: {}", self.summarize())
    }
}

struct Article {
    title: String,
    content: String,
}

impl Summary for Article {
    fn summarize(&self) -> String {
        format!("{}: {}", self.title, self.content)
    }
}
```

**Trong Knight Age**:
```rust
#[async_trait]
pub trait PacketHandler: Send + Sync {
    async fn handle(
        &self,
        packet: &Packet,
        ctx: &PacketContext,
    ) -> io::Result<PacketHandlerResult>;
}
```

---

### Trait Bounds

```rust
// Function with trait bound
fn notify<T: Summary>(item: &T) {
    println!("{}", item.summarize());
}

// Multiple trait bounds
fn notify<T: Summary + Display>(item: &T) { }

// Where clause (cleaner)
fn notify<T>(item: &T)
where
    T: Summary + Display,
{ }
```

---

### Generics

```rust
// Generic struct
struct Point<T> {
    x: T,
    y: T,
}

impl<T> Point<T> {
    fn new(x: T, y: T) -> Self {
        Point { x, y }
    }
}

// Generic function
fn largest<T: PartialOrd>(list: &[T]) -> &T {
    let mut largest = &list[0];
    for item in list {
        if item > largest {
            largest = item;
        }
    }
    largest
}
```

**Trong Knight Age**:
```rust
// Arc<Mutex<T>> is generic
Arc<Mutex<ConnectionWriter>>

// HashMap<K, V> is generic
HashMap<u64, SessionData>
HashMap<i8, Arc<dyn PacketHandler>>
```

---

### Trait Objects - Dynamic Dispatch

```rust
trait Draw {
    fn draw(&self);
}

struct Circle;
impl Draw for Circle {
    fn draw(&self) { println!("Circle"); }
}

struct Square;
impl Draw for Square {
    fn draw(&self) { println!("Square"); }
}

// Vec of trait objects
let shapes: Vec<Box<dyn Draw>> = vec![
    Box::new(Circle),
    Box::new(Square),
];

for shape in shapes {
    shape.draw();
}
```

**Trong Knight Age**:
```rust
// PacketRegistry stores handlers as trait objects
HashMap<i8, Arc<dyn PacketHandler>>
```

---

## 📦 MODULE SYSTEM

### Module Declaration

```rust
// In lib.rs or main.rs
mod network;  // Declares module

// In network.rs or network/mod.rs
pub mod connection;
pub mod session;
pub mod crypto;
```

---

### Use Statement

```rust
// Bring into scope
use std::collections::HashMap;
use std::io::{self, Write};

// Rename
use std::io::Result as IoResult;

// Re-export
pub use crate::network::Session;
```

**Trong Knight Age**:
```rust
// In network/mod.rs
pub mod connection;
pub mod session;

pub use session::{Session, SessionManager};
pub use connection::{ConnectionReader, ConnectionWriter};
```

---

### Visibility

```rust
mod my_mod {
    // Private by default
    fn private_fn() {}
    
    // Public
    pub fn public_fn() {}
    
    // Public within crate
    pub(crate) fn crate_visible() {}
    
    // Public within parent module
    pub(super) fn parent_visible() {}
}
```

---

## 🎨 COMMON PATTERNS

### Builder Pattern

```rust
struct ServerConfig {
    num_workers: usize,
    bind_address: String,
}

impl ServerConfig {
    pub fn new() -> Self {
        Self {
            num_workers: 4,
            bind_address: String::from("127.0.0.1:8080"),
        }
    }
    
    pub fn with_workers(mut self, n: usize) -> Self {
        self.num_workers = n;
        self
    }
    
    pub fn with_bind_address(mut self, addr: &str) -> Self {
        self.bind_address = addr.to_string();
        self
    }
}

// Usage
let config = ServerConfig::new()
    .with_workers(8)
    .with_bind_address("0.0.0.0:9000");
```

---

### Newtype Pattern

```rust
struct SessionId(u64);
struct PlayerId(u64);

// Type safety: can't mix up IDs
fn get_session(id: SessionId) { }
// get_session(PlayerId(123));  // ❌ Type error
```

---

### RAII (Resource Acquisition Is Initialization)

```rust
{
    let file = File::open("data.txt")?;
    // Use file...
}  // file automatically closed (Drop trait)

{
    let guard = mutex.lock().unwrap();
    // Use guard...
}  // lock automatically released (Drop trait)
```

---

### Type State Pattern

```rust
struct Locked;
struct Unlocked;

struct Door<State> {
    _state: PhantomData<State>,
}

impl Door<Locked> {
    fn unlock(self) -> Door<Unlocked> {
        Door { _state: PhantomData }
    }
}

impl Door<Unlocked> {
    fn open(&self) { }
    fn lock(self) -> Door<Locked> {
        Door { _state: PhantomData }
    }
}

// Type-safe API
let door = Door::<Locked>::new();
// door.open();  // ❌ Error: Locked door can't open
let door = door.unlock();
door.open();  // ✅ OK
```

---

## ⚠️ PITFALLS & SOLUTIONS

### 1. Moving Values in Loops

**Problem**:
```rust
let data = vec![1, 2, 3];

for item in data {
    println!("{}", item);
}

// println!("{:?}", data);  // ❌ ERROR: value moved
```

**Solution**:
```rust
// Borrow instead
for item in &data {
    println!("{}", item);
}
println!("{:?}", data);  // ✅ OK
```

---

### 2. Mutex Deadlock

**Problem**:
```rust
let m1 = Arc::new(Mutex::new(0));
let m2 = Arc::new(Mutex::new(0));

// Thread 1: lock m1, then m2
// Thread 2: lock m2, then m1
// → Deadlock!
```

**Solution**:
- Always lock in same order
- Use `try_lock()` with timeout
- Use single lock for related data
- Use `RwLock` for read-heavy workloads

---

### 3. Async Closure Issues

**Problem**:
```rust
tokio::spawn(async {
    let data = get_data();  // Not async
    process(data).await;    // data might be moved
});
```

**Solution**:
```rust
tokio::spawn(async move {
    let data = get_data().await;
    process(data).await;
});
```

---

### 4. String vs &str Confusion

```rust
// &str - String slice (borrowed)
fn print_str(s: &str) {
    println!("{}", s);
}

// String - Owned string
fn take_string(s: String) {
    println!("{}", s);
}

// Usage
let owned = String::from("hello");
print_str(&owned);        // ✅ Borrow
// take_string(owned);    // ✅ Move
// print_str(&owned);     // ❌ ERROR: moved

let slice: &str = "world";
print_str(slice);         // ✅ OK
// take_string(slice);    // ❌ ERROR: expected String
take_string(slice.to_string());  // ✅ OK: convert to String
```

---

### 5. Lifetime Elision Confusion

```rust
// This works (lifetime elided):
fn first_word(s: &str) -> &str {
    &s[..1]
}

// This doesn't (multiple inputs):
fn longest(x: &str, y: &str) -> &str {  // ❌ ERROR
    if x.len() > y.len() { x } else { y }
}

// Need explicit lifetime:
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {  // ✅ OK
    if x.len() > y.len() { x } else { y }
}
```

---

## 💡 BEST PRACTICES

### 1. Use Type Aliases

```rust
type SessionId = u64;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

async fn get_session(id: SessionId) -> Result<Session> {
    // ...
}
```

---

### 2. Implement Debug and Display

```rust
#[derive(Debug)]
struct User {
    id: u64,
    name: String,
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "User({}: {})", self.id, self.name)
    }
}
```

---

### 3. Use Enums for State

```rust
enum ConnectionState {
    Connected,
    Authenticated { user_id: u64 },
    InGame { char_id: u64 },
    Disconnected,
}

match state {
    ConnectionState::Authenticated { user_id } => {
        // ...
    }
    _ => {}
}
```

---

### 4. Prefer Iterators

```rust
// Instead of:
let mut sum = 0;
for i in 0..10 {
    sum += i;
}

// Use:
let sum: i32 = (0..10).sum();

// More complex:
let even_squares: Vec<i32> = (0..10)
    .filter(|x| x % 2 == 0)
    .map(|x| x * x)
    .collect();
```

---

### 5. Use ? for Error Propagation

```rust
// Instead of:
let file = match File::open("file.txt") {
    Ok(f) => f,
    Err(e) => return Err(e),
};

// Use:
let file = File::open("file.txt")?;
```

---

## 📖 RECOMMENDED RESOURCES

### Official Resources
- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Rustlings](https://github.com/rust-lang/rustlings) - Interactive exercises

### Async Rust
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [Async Book](https://rust-lang.github.io/async-book/)

### Advanced Topics
- [Rust Design Patterns](https://rust-unofficial.github.io/patterns/)
- [Rust Cookbook](https://rust-lang-nursery.github.io/rust-cookbook/)
- [Too Many Lists](https://rust-unofficial.github.io/too-many-lists/) - Learn linked lists

### Tools
- `cargo fmt` - Code formatter
- `cargo clippy` - Linter
- `cargo check` - Fast compile check
- `cargo test` - Run tests
- `cargo doc --open` - Generate and open docs

---

## 🎯 NEXT STEPS IN LEARNING

### Currently Mastered ✅
- Basic syntax
- Ownership & borrowing
- Smart pointers (Arc, Mutex, RwLock)
- Async/await with Tokio
- Traits & trait objects
- Error handling
- Module system

### To Learn Next 📚
- [ ] Macros (declarative & procedural)
- [ ] Unsafe Rust (when needed)
- [ ] Advanced type system features
- [ ] Benchmark & profiling tools
- [ ] Database integration (diesel/sqlx)
- [ ] Serialization (serde)
- [ ] Testing strategies
- [ ] Documentation best practices

---

**Cập nhật lần cuối**: 02/01/2026  
**Tiến độ học**: Intermediate Level  
**Dự án thực hành**: Knight Age Server
