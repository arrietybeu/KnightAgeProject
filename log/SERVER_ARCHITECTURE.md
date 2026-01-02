# SERVER ARCHITECTURE - Knight Age Rust Server

> Tài liệu này mô tả kiến trúc server Rust đang được xây dựng để thay thế backend Java cũ.

---

## 📐 KIẾN TRÚC TỔNG QUAN

### Architecture Layers

```
┌────────────────────────────────────────────────────────────────┐
│                     APPLICATION LAYER                           │
│  (Game Logic, Business Rules - TODO)                           │
├────────────────────────────────────────────────────────────────┤
│                     HANDLER LAYER                               │
│  (Packet Handlers, Request Processing)                         │
├────────────────────────────────────────────────────────────────┤
│                     WORKER POOL LAYER                           │
│  (Multi-threaded Packet Processing)                            │
├────────────────────────────────────────────────────────────────┤
│                     SESSION LAYER                               │
│  (Session Management, State Management)                        │
├────────────────────────────────────────────────────────────────┤
│                     NETWORK LAYER                               │
│  (Packet I/O, Encryption, Binary Protocol)                     │
├────────────────────────────────────────────────────────────────┤
│                     TRANSPORT LAYER                             │
│  (TCP Socket, Tokio Runtime)                                   │
└────────────────────────────────────────────────────────────────┘
```

---

## 🏗️ MODULE STRUCTURE

### Project Layout

```
knight-age-server/
├── Cargo.toml
└── src/
    ├── main.rs                     # Entry point
    └── network/
        ├── mod.rs                  # Network module exports
        │
        ├── connection.rs           # TCP connection I/O
        ├── crypto.rs               # XOR encryption
        ├── session.rs              # Session management
        ├── state.rs                # Connection states
        ├── context.rs              # Packet context
        ├── worker.rs               # Worker pool
        ├── opcode.rs               # Command constants
        │
        ├── packet/
        │   ├── mod.rs              # Packet struct
        │   ├── reader.rs           # Binary reader
        │   └── writer.rs           # Binary writer
        │
        ├── handler/
        │   ├── mod.rs              # Handler exports
        │   ├── packet_handler.rs   # Handler trait
        │   ├── registry.rs         # Handler registry
        │   ├── cm_login.rs         # Login handler
        │   └── cm_server_info.rs   # Server info handler
        │
        └── server_message/
            ├── mod.rs              # Message exports
            ├── traits.rs           # ServerMessage trait
            └── messages.rs         # Message implementations
```

---

## 🔧 CORE COMPONENTS

### 1. Main Entry Point (main.rs)

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // 1. Start network module
    network::start_net_work().await?;
    Ok(())
}
```

**Vai trò**: 
- Khởi động Tokio async runtime
- Gọi network module để start server

### 2. Network Module (network/mod.rs)

```rust
pub async fn start_net_work() -> Result<(), Box<dyn Error>> {
    // 1. Create packet registry
    let registry = create_packet_registry();
    let registry = Arc::new(registry);
    
    // 2. Create server config
    let config = ServerConfig::new()
        .with_workers(4)
        .with_bind_address("127.0.0.1:19129");
    
    // 3. Create worker pool
    let worker_pool = WorkerPool::new(
        config.num_workers, 
        registry.clone()
    );
    
    // 4. Create session manager
    let session_manager = Arc::new(SessionManager::new());
    
    // 5. Start game server
    start_game_server(
        config,
        session_manager,
        worker_pool.get_sender()
    ).await?;
}
```

**Luồng khởi động**:
```
┌────────────────────────────────────────────────────────────┐
│                  SERVER STARTUP FLOW                        │
└────────────────────────────────────────────────────────────┘

1. main()
   ├── start_net_work()
   │   ├── Create PacketRegistry
   │   │   └── Register all handlers
   │   │
   │   ├── Create ServerConfig
   │   │   └── Workers: 4, Address: 127.0.0.1:19129
   │   │
   │   ├── Create WorkerPool
   │   │   └── Spawn 4 worker tasks
   │   │
   │   ├── Create SessionManager
   │   │   └── Thread-safe session storage
   │   │
   │   └── start_game_server()
   │       └── TcpListener::bind()
   │           └── Accept loop
   │               ├── Accept connection
   │               ├── Create Session
   │               ├── Register session
   │               └── Spawn session handler
   │
   └── Wait forever (server running)
```

---

## 🌐 SESSION MANAGEMENT

### Session Struct (session.rs)

```rust
pub struct Session {
    pub id: u64,                            // Unique session ID
    pub addr: SocketAddr,                   // Client address
    reader: ConnectionReader,               // Reads packets
    ctx: PacketContext,                     // Session context
    worker_sender: Option<Sender<WorkerTask>>, // Worker pool
}
```

**Lifecycle**:

```
┌────────────────────────────────────────────────────────────┐
│                    SESSION LIFECYCLE                        │
└────────────────────────────────────────────────────────────┘

1. Connection Accepted
   ├── Create Session::new()
   │   ├── Generate unique session_id
   │   ├── Split TcpStream into (read, write)
   │   ├── Create ConnectionReader
   │   ├── Create ConnectionWriter (wrapped in Arc<Mutex>)
   │   └── Create PacketContext
   │
2. Session Registration
   ├── SessionManager::register()
   │   ├── Store writer for sending
   │   └── Store context for state
   │
3. Session Running
   ├── Session::run() loop
   │   ├── Key Exchange (CMD -40)
   │   │   ├── Generate XOR key
   │   │   ├── Send to client
   │   │   └── Set cipher for read/write
   │   │
   │   └── Packet Processing Loop
   │       ├── reader.read_packet()
   │       ├── Create WorkerTask
   │       └── Send to worker pool
   │
4. Session Cleanup
   ├── On error/disconnect
   │   ├── SessionManager::unregister()
   │   └── Close connection
   │
└── Session dropped
```

### SessionManager (session.rs)

```rust
pub struct SessionManager {
    sessions: RwLock<HashMap<u64, SessionData>>
}

struct SessionData {
    writer: SharedWriter,       // For sending packets
    ctx: PacketContext,         // Session context
}
```

**Thread Safety**:
- `Arc<SessionManager>` - Shared across threads
- `RwLock<HashMap>` - Multiple readers, single writer
- `Arc<Mutex<ConnectionWriter>>` - Thread-safe writer

---

## 📦 PACKET PROCESSING

### Packet Structure (packet/mod.rs)

```rust
pub struct Packet {
    pub cmd: i8,            // Command code (-128 to 127)
    pub data: Vec<u8>,      // Binary data
}
```

### Binary I/O

#### PacketReader (packet/reader.rs)

```rust
pub struct PacketReader {
    data: Vec<u8>,
    pos: usize,
}

impl PacketReader {
    // Read primitives (Big-Endian)
    pub fn read_byte(&mut self) -> io::Result<u8>
    pub fn read_i8(&mut self) -> io::Result<i8>
    pub fn read_i16(&mut self) -> io::Result<i16>
    pub fn read_i32(&mut self) -> io::Result<i32>
    pub fn read_i64(&mut self) -> io::Result<i64>
    
    // Read UTF-8 string (length-prefixed)
    pub fn read_utf(&mut self) -> io::Result<String>
    
    // Read arrays
    pub fn read_bytes(&mut self, len: usize) -> io::Result<Vec<u8>>
    
    // Utility
    pub fn available(&self) -> usize
}
```

#### PacketWriter (packet/writer.rs)

```rust
pub struct PacketWriter {
    data: Vec<u8>,
}

impl PacketWriter {
    // Write primitives (Big-Endian)
    pub fn write_byte(&mut self, value: u8)
    pub fn write_i8(&mut self, value: i8)
    pub fn write_i16(&mut self, value: i16)
    pub fn write_i32(&mut self, value: i32)
    pub fn write_i64(&mut self, value: i64)
    
    // Write UTF-8 string (length-prefixed)
    pub fn write_utf(&mut self, s: &str)
    
    // Write arrays
    pub fn write_bytes(&mut self, bytes: &[u8])
    
    // Get result
    pub fn into_packet(self, cmd: i8) -> Packet
}
```

**Encoding Rules**:
```
Integers: Big-Endian (Network Byte Order)
  i16:  [high_byte][low_byte]
  i32:  [b3][b2][b1][b0]
  i64:  [b7][b6][b5][b4][b3][b2][b1][b0]

Strings: Length-prefixed UTF-8
  [length: u16][utf8_bytes...]

Arrays: Manual length prefix
  [count: u8/u16][element1][element2]...
```

---

## 🔐 ENCRYPTION - XOR Cipher

### XorCipher (crypto.rs)

```rust
pub struct XorCipher {
    key: Vec<u8>,           // Encryption key
    read_pos: usize,        // Current read position
    write_pos: usize,       // Current write position
}
```

**Key Operations**:

```rust
// 1. Generate random key
let key = XorCipher::generate_key(16);
let cipher = XorCipher::new(key);

// 2. Transform for client
let client_key = cipher.get_key_for_client();
// Server: [k0, k1, k2, k3]
// Client will receive and transform to same key

// 3. Encrypt/Decrypt
let encrypted = cipher.encrypt_byte(byte);
let decrypted = cipher.decrypt_byte(byte);

// 4. Buffer operations
cipher.encrypt_buffer(&mut data);
cipher.decrypt_buffer(&mut data);
```

**Algorithm**:
```rust
// XOR with rotating key
pub fn encrypt_byte(&mut self, byte: u8) -> u8 {
    let result = byte ^ self.key[self.write_pos];
    self.write_pos = (self.write_pos + 1) % self.key.len();
    result
}

pub fn decrypt_byte(&mut self, byte: u8) -> u8 {
    let result = byte ^ self.key[self.read_pos];
    self.read_pos = (self.read_pos + 1) % self.key.len();
    result
}
```

**Key Transformation** (để khớp với client):
```rust
// Client code (C#):
// for (int j = 0; j < key.Length - 1; j++)
//     key[j + 1] ^= key[j];

// Server must send inverse:
pub fn get_key_for_client(&self) -> Vec<u8> {
    let mut client_key = self.key.clone();
    for i in (1..client_key.len()).rev() {
        client_key[i] ^= client_key[i - 1];
    }
    client_key
}
```

---

## 🛠️ WORKER POOL - Concurrent Processing

### WorkerPool (worker.rs)

```rust
pub struct WorkerPool {
    sender: Sender<WorkerTask>,
}

pub struct WorkerTask {
    pub packet: Packet,
    pub ctx: PacketContext,
}
```

**Architecture**:

```
┌──────────────────────────────────────────────────────────┐
│                    WORKER POOL PATTERN                    │
└──────────────────────────────────────────────────────────┘

Session Threads                  Worker Threads
    │                                 │
    │                            ┌────┴────┐
    ├─► WorkerTask ────────────► │ Worker 0 │
    │        ↓                    └──────────┘
    │   [Packet, Context]              │
    │        ↓                          │
    │   mpsc::channel             ┌────┴────┐
    │   (10000 capacity)          │ Worker 1 │
    │                             └──────────┘
    ├─► WorkerTask ────────────►      │
    │                                  │
    │                             ┌────┴────┐
    ├─► WorkerTask ────────────► │ Worker 2 │
    │                             └──────────┘
    │                                  │
    │                             ┌────┴────┐
    └─► WorkerTask ────────────► │ Worker 3 │
                                 └──────────┘
                                      │
                                      ↓
                              PacketRegistry
                                      ↓
                                 Handler
```

**Workflow**:
```rust
// 1. Session reads packet
let packet = reader.read_packet().await?;

// 2. Create task
let task = WorkerTask {
    packet,
    ctx: self.ctx.clone(),
};

// 3. Submit to worker pool
worker_sender.send(task).await?;

// 4. Worker processes
async fn process_task(worker_id, registry, task) {
    let result = registry.handle_packet(
        &task.packet,
        &task.ctx
    ).await;
    // Handle result...
}
```

**Benefits**:
- ✅ Non-blocking packet reading
- ✅ Concurrent packet processing
- ✅ Better CPU utilization
- ✅ Scalable to many CCU

---

## 🎯 HANDLER SYSTEM

### PacketHandler Trait (handler/packet_handler.rs)

```rust
pub enum PacketHandlerResult {
    Ok,                         // Success
    CloseConnection,            // Close session
    Error(String),              // Handler error
}

#[async_trait]
pub trait PacketHandler: Send + Sync {
    async fn handle(
        &self,
        packet: &Packet,
        ctx: &PacketContext,
    ) -> io::Result<PacketHandlerResult>;
}
```

### PacketRegistry (handler/registry.rs)

```rust
pub struct PacketRegistry {
    handlers: HashMap<i8, Arc<dyn PacketHandler>>,
}

impl PacketRegistry {
    pub fn register(&mut self, cmd: i8, handler: impl PacketHandler + 'static) {
        self.handlers.insert(cmd, Arc::new(handler));
    }
    
    pub async fn handle_packet(
        &self,
        packet: &Packet,
        ctx: &PacketContext,
    ) -> io::Result<PacketHandlerResult> {
        if let Some(handler) = self.handlers.get(&packet.cmd) {
            handler.handle(packet, ctx).await
        } else {
            // Unhandled packet
            Ok(PacketHandlerResult::Ok)
        }
    }
}
```

### Example Handler - CmLogin (handler/cm_login.rs)

```rust
pub struct CmLogin;

#[async_trait]
impl PacketHandler for CmLogin {
    async fn handle(
        &self,
        packet: &Packet,
        ctx: &PacketContext,
    ) -> io::Result<PacketHandlerResult> {
        // 1. Parse packet
        let mut reader = PacketReader::new(packet.data.clone());
        let username = reader.read_utf()?;
        let password = reader.read_utf()?;
        // ... read more fields
        
        // 2. Validate credentials
        // TODO: Database query
        
        // 3. Send response
        let mut writer = PacketWriter::new();
        writer.write_i8(1); // Success
        writer.write_utf(&session_token);
        
        let response = writer.into_packet(1); // CMD 1
        ctx.send_packet(response).await?;
        
        Ok(PacketHandlerResult::Ok)
    }
}
```

**Handler Registration**:
```rust
fn create_packet_registry() -> PacketRegistry {
    let mut registry = PacketRegistry::default();
    
    // Register handlers
    registry.register(1, CmLogin);           // Login
    registry.register(37, CmServerInfo);     // Server info
    // ... more handlers
    
    registry
}
```

---

## 📡 CONNECTION I/O

### ConnectionReader (connection.rs)

```rust
pub struct ConnectionReader {
    reader: BufReader<OwnedReadHalf>,
    cipher: Option<XorCipher>,
    key_exchanged: bool,
}
```

**Read Flow**:
```
┌─────────────────────────────────────────────────────┐
│              PACKET READING PROCESS                  │
└─────────────────────────────────────────────────────┘

1. Read command byte
   ├── read_u8()
   └── Decrypt if cipher set

2. Determine length size
   ├── If key_exchanged:
   │   ├── Check SPECIAL_CMDS [-51, -52, -54, 126]
   │   │   ├── YES: Read 4 bytes (big-endian)
   │   │   └── NO:  Read 2 bytes (big-endian)
   │   └── Decrypt each byte
   └── Else: Read 2 bytes (no decrypt)

3. Read data
   ├── Read exact length bytes
   └── Decrypt buffer if cipher set

4. Return Packet { cmd, data }
```

### ConnectionWriter (connection.rs)

```rust
pub struct ConnectionWriter {
    writer: BufWriter<OwnedWriteHalf>,
    cipher: Option<XorCipher>,
    key_exchanged: bool,
}
```

**Write Flow**:
```
1. Write command byte
   └── Encrypt if cipher set

2. Write length (2 bytes, big-endian)
   ├── Encrypt high byte
   └── Encrypt low byte

3. Write data bytes
   └── Encrypt each byte

4. Flush buffer
```

**Special: Key Exchange Response**:
```rust
pub async fn write_key_exchange(&mut self, key: &[u8]) -> io::Result<()> {
    // Unencrypted write
    self.writer.write_i8(CMD_KEY_EXCHANGE).await?;
    self.writer.write_u16(key.len() as u16).await?;
    self.writer.write_all(key).await?;
    self.writer.flush().await?;
    Ok(())
}
```

---

## 🔄 PACKET CONTEXT

### PacketContext (context.rs)

```rust
pub struct PacketContext {
    pub session_id: u64,
    pub addr: SocketAddr,
    writer: SharedWriter,           // Thread-safe writer
    state: Arc<RwLock<ConnectionState>>,
    // TODO: Add user_id, character_id, etc.
}
```

**Vai trò**:
- Lưu thông tin session
- Cung cấp writer để send packets
- Quản lý connection state
- Sẽ chứa game state (player, character, etc.)

**Methods**:
```rust
impl PacketContext {
    // Send packet to client
    pub async fn send_packet(&self, packet: Packet) -> io::Result<()>
    
    // Get/Set state
    pub async fn get_state(&self) -> ConnectionState
    pub async fn set_state(&self, state: ConnectionState)
    
    // Get writer
    pub fn get_writer(&self) -> SharedWriter
}
```

---

## 📊 CONNECTION STATE

### ConnectionState (state.rs)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Connected,          // Just connected
    KeyExchanged,       // Key exchange done
    Authenticated,      // Login successful
    InGame,            // Playing game
    Disconnected,      // Disconnected
}
```

**State Transitions**:
```
Connected
    │
    ├─ Key Exchange (CMD -40)
    ↓
KeyExchanged
    │
    ├─ Login (CMD 1)
    ↓
Authenticated
    │
    ├─ Enter Game
    ↓
InGame
    │
    ├─ Disconnect
    ↓
Disconnected
```

---

## 🚀 CONCURRENCY MODEL

### Async Tasks

```
┌──────────────────────────────────────────────────┐
│              TOKIO TASK STRUCTURE                 │
└──────────────────────────────────────────────────┘

Main Task (tokio::main)
    │
    ├── Accept Loop Task
    │   └── For each connection:
    │       └── Spawn Session Task
    │           ├── Read packets
    │           └── Submit to workers
    │
    ├── Worker Task 0
    │   └── Loop: Process WorkerTasks
    │
    ├── Worker Task 1
    │   └── Loop: Process WorkerTasks
    │
    ├── Worker Task 2
    │   └── Loop: Process WorkerTasks
    │
    └── Worker Task 3
        └── Loop: Process WorkerTasks
```

### Thread Safety

**Shared State**:
```rust
// SessionManager - shared across all tasks
Arc<SessionManager>
    └── RwLock<HashMap<u64, SessionData>>
        └── Each SessionData contains:
            ├── SharedWriter = Arc<Mutex<ConnectionWriter>>
            └── PacketContext (contains Arc<RwLock<State>>)

// PacketRegistry - shared with all workers
Arc<PacketRegistry>
    └── HashMap<i8, Arc<dyn PacketHandler>>
```

**Channel Communication**:
```rust
// Worker Pool Channel
mpsc::channel<WorkerTask>(capacity: 10000)
    ├── Sender: Cloned to each Session
    └── Receiver: Shared among Workers
```

---

## 💡 DESIGN PATTERNS

### 1. **Singleton Pattern**
```rust
// SessionManager - one instance shared via Arc
let session_manager = Arc::new(SessionManager::new());
```

### 2. **Registry Pattern**
```rust
// PacketRegistry - maps commands to handlers
registry.register(cmd, handler);
registry.handle_packet(packet, ctx);
```

### 3. **Worker Pool Pattern**
```rust
// Multiple workers share task queue
WorkerPool::new(num_workers, registry);
```

### 4. **Context Object Pattern**
```rust
// PacketContext carries session info
async fn handle(&self, packet: &Packet, ctx: &PacketContext)
```

### 5. **Builder Pattern** (in config)
```rust
ServerConfig::new()
    .with_workers(4)
    .with_bind_address("127.0.0.1:19129");
```

---

## 🔍 KEY RUST CONCEPTS USED

### Ownership & Borrowing
```rust
// Ownership transfer
let packet = Packet::new(1);
worker_sender.send(packet).await;  // packet moved

// Borrowing
async fn handle(&self, packet: &Packet)  // immutable borrow
```

### Smart Pointers
```rust
Arc<T>          // Atomic Reference Counting (thread-safe)
Box<T>          // Heap allocation
Rc<T>           // Reference Counting (single-threaded)
```

### Interior Mutability
```rust
Mutex<T>        // Mutual exclusion lock
RwLock<T>       // Read-write lock
```

### Async/Await
```rust
#[tokio::main]
async fn main() { }

async fn handle(&self) -> io::Result<()> {
    let data = reader.read().await?;
    writer.write(data).await?;
}
```

### Error Handling
```rust
Result<T, E>    // Success or error
Option<T>       // Some or None
?               // Propagate error
```

### Traits
```rust
#[async_trait]
trait PacketHandler {
    async fn handle(&self, ...) -> io::Result<...>;
}
```

---

## 📈 PERFORMANCE CONSIDERATIONS

### Scalability
- **Worker Pool**: 4 workers xử lý packets song song
- **Non-blocking I/O**: Tokio async runtime
- **Zero-copy**: Direct buffer operations
- **Connection pooling**: SessionManager với RwLock

### Memory Management
- **Arc**: Share data without copying
- **Mutex**: Minimal lock contention
- **Buffer reuse**: BufReader/BufWriter

### Future Optimizations
- [ ] Object pool cho Packet allocation
- [ ] Custom allocator
- [ ] Zero-copy serialization với `bytes` crate
- [ ] Connection keep-alive tuning

---

## 🐛 DEBUGGING & LOGGING

### Current Logging
```rust
println!("[GameServer] Listening on {}", addr);
println!("[Session {}] Key exchange completed", id);
eprintln!("[Worker {}] Error: {}", worker_id, e);
```

### TODO: Better Logging
```rust
// Using `tracing` crate
use tracing::{info, warn, error, debug};

info!(session_id = %id, "Key exchange completed");
error!(session_id = %id, error = %e, "Handler error");
debug!("Packet received: {:?}", packet);
```

---

## 📋 TODO LIST

### Core Network (Giai đoạn 1)
- [x] TCP Listener
- [x] Session Management
- [x] XOR Encryption
- [x] Binary Packet I/O
- [x] Worker Pool
- [ ] **Message Queue (per session)**
- [ ] **Heartbeat/Keep-alive**
- [ ] **Better error handling**

### Authentication (Giai đoạn 2)
- [ ] Complete Login Handler
- [ ] Server Info Handler
- [ ] Database integration
- [ ] Character list/creation

### Game Features (Giai đoạn 3+)
- [ ] Map system
- [ ] Movement
- [ ] Combat
- [ ] Inventory
- [ ] Chat
- [ ] ... nhiều features khác

---

**Cập nhật lần cuối**: 02/01/2026  
**Trạng thái**: Giai đoạn 1 - Core Network Layer (90% complete)  
**Tiếp theo**: Hoàn thiện Login Handler & Database
