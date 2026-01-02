# KNIGHT AGE PROJECT - CONTEXT & ROADMAP

> **Dự án**: Chuyển đổi backend game Knight Age từ Java sang Rust  
> **Mục tiêu**: Vừa học Rust vừa xây dựng lại server game online  
> **Ngày bắt đầu**: 02/01/2026

---

## 📋 TỔNG QUAN DỰ ÁN

### Giới thiệu
Knight Age là một game online 2D MMORPG kiểu Java Mobile đã có client hoàn chỉnh được viết bằng Unity (C#). Backend cũ được viết bằng Java, và chúng ta đang chuyển đổi sang Rust để:
- Học và thực hành Rust trong dự án thực tế
- Cải thiện hiệu năng và độ an toàn của server
- Tận dụng các ưu điểm của Rust (memory safety, concurrency, zero-cost abstractions)

### Cấu trúc dự án

```
KnightAgeProject/
├── knight-age-client/          # Unity C# Client (Hoàn thiện)
│   └── Assets/
│       └── Scripts/
│           └── Assembly-CSharp/
│               ├── Session_ME.cs              # Socket & Network
│               ├── Message.cs                  # Binary Protocol
│               ├── GlobalMessageHandler.cs     # Message Router
│               ├── GlobalService.cs            # API Calls
│               ├── myReader.cs / myWriter.cs   # Binary I/O
│               └── ...
│
├── knight-age-server/          # Rust Server (Đang phát triển)
│   └── src/
│       ├── main.rs
│       └── network/
│           ├── mod.rs              # Network module
│           ├── session.rs          # Session management
│           ├── crypto.rs           # XOR encryption
│           ├── packet.rs           # Packet structures
│           ├── opcode.rs           # Command codes
│           ├── worker.rs           # Worker pool
│           ├── handler/            # Packet handlers
│           ├── packet/             # Packet definitions
│           └── server_message/     # Server responses
│
└── log/                        # Documentation
    ├── CONTEXT.md                  # File này - Tổng quan dự án
    ├── PROTOCOL_ANALYSIS.md        # Phân tích giao thức (sẵn có)
    ├── CLIENT_ARCHITECTURE.md      # Kiến trúc client (mới tạo)
    ├── SERVER_ARCHITECTURE.md      # Kiến trúc server (mới tạo)
    ├── PACKET_REFERENCE.md         # Tham chiếu các packet (mới tạo)
    └── RUST_LEARNING_NOTES.md     # Ghi chú học Rust (mới tạo)
```

---

## 🎯 CÁC GIAI ĐOẠN PHÁT TRIỂN

### ✅ Giai đoạn 0: Chuẩn bị (Hoàn thành)
- [x] Thiết lập môi trường Rust
- [x] Tạo cấu trúc project cơ bản
- [x] Tạo tài liệu phân tích giao thức (PROTOCOL_ANALYSIS.md)

### 🚧 Giai đoạn 1: Core Network Layer (Đang làm)
- [x] TCP Socket Listener
- [x] Session Management
- [x] XOR Encryption/Decryption
- [x] Binary Packet Reader/Writer
- [x] Worker Pool cho xử lý packet
- [ ] **Hoàn thiện Key Exchange** (Đang test)
- [ ] **Message Queue System**

### 📋 Giai đoạn 2: Authentication System
- [ ] Login Handler (CMD 1)
- [ ] Server Info Handler (CMD 37, CMD 61)
- [ ] Character List (CMD 13)
- [ ] Character Selection
- [ ] Character Creation
- [ ] Database Integration (SQLite/PostgreSQL)

### 📋 Giai đoạn 3: Game World Foundation
- [ ] Map System
- [ ] Player Movement (CMD 4)
- [ ] Monster System (CMD 7, 9, 10, 17)
- [ ] NPC System (CMD 23, -44)
- [ ] Item System (CMD 19, 20, 25, 28)

### 📋 Giai đoạn 4: Gameplay Features
- [ ] Combat System (Fire Monster, Skills)
- [ ] Inventory System (CMD 16)
- [ ] Equipment System (CMD 15)
- [ ] Quest System (CMD 52)
- [ ] Chat System (CMD 27, 34, 35)
- [ ] Party System
- [ ] Clan System

### 📋 Giai đoạn 5: Advanced Features
- [ ] Shop System
- [ ] Trading System
- [ ] Arena/PvP
- [ ] Event System
- [ ] Admin Tools
- [ ] Anti-cheat

### 📋 Giai đoạn 6: Optimization & Deployment
- [ ] Performance Optimization
- [ ] Load Testing
- [ ] Security Hardening
- [ ] Deployment Setup
- [ ] Monitoring & Logging

---

## 📚 TÀI LIỆU THAM KHẢO QUAN TRỌNG

### File Client cần đọc kỹ
1. **[Session_ME.cs](../knight-age-client/Assets/Scripts/Assembly-CSharp/Session_ME.cs)** - 547 dòng
   - Quản lý kết nối TCP
   - XOR encryption logic
   - Thread quản lý send/receive
   - Key exchange protocol

2. **[Message.cs](../knight-age-client/Assets/Scripts/Assembly-CSharp/Message.cs)** - 47 dòng
   - Cấu trúc message packet
   - Command + Data
   - Reader/Writer interface

3. **[GlobalMessageHandler.cs](../knight-age-client/Assets/Scripts/Assembly-CSharp/GlobalMessageHandler.cs)** - 444 dòng
   - Switch-case cho ~100 command types
   - Router tất cả messages đến handlers tương ứng
   - Điểm vào chính cho xử lý logic

4. **[GlobalService.cs](../knight-age-client/Assets/Scripts/Assembly-CSharp/GlobalService.cs)** - 1361 dòng
   - API calls từ client → server
   - Tất cả các request được gửi đi
   - Tham chiếu cách serialize data

5. **[myReader.cs](../knight-age-client/Assets/Scripts/Assembly-CSharp/myReader.cs)** & **[myWriter.cs](../knight-age-client/Assets/Scripts/Assembly-CSharp/myWriter.cs)**
   - Binary I/O utilities
   - Read/Write primitive types
   - String encoding (UTF-8)

### File Server đã triển khai
1. **[main.rs](../knight-age-server/src/main.rs)** - Entry point
2. **[network/mod.rs](../knight-age-server/src/network/mod.rs)** - Network module
3. **[network/session.rs](../knight-age-server/src/network/session.rs)** - Session management
4. **[network/crypto.rs](../knight-age-server/src/network/crypto.rs)** - XOR cipher
5. **[network/packet.rs](../knight-age-server/src/network/packet.rs)** - Packet structures
6. **[network/opcode.rs](../knight-age-server/src/network/opcode.rs)** - Command codes

---

## 🔑 KIẾN THỨC RUST QUAN TRỌNG

### Concepts đã sử dụng
- ✅ `async/await` với Tokio runtime
- ✅ `Arc<T>` - Atomic Reference Counting cho multi-threading
- ✅ `Mutex<T>` & `RwLock<T>` - Thread-safe shared state
- ✅ `tokio::net::TcpListener` & `TcpStream`
- ✅ `tokio::sync::mpsc` - Multi-producer single-consumer channels
- ✅ Pattern matching với `match`
- ✅ Error handling với `Result<T, E>`
- ✅ Traits & Generics
- ✅ Module system

### Concepts cần học tiếp
- [ ] `unsafe` code (nếu cần tối ưu)
- [ ] Macro programming
- [ ] Advanced async patterns
- [ ] Database ORM (diesel/sqlx)
- [ ] Serialization với `serde`
- [ ] Configuration management

---

## 🚀 WORKFLOW LÀM VIỆC

### Khi implement một feature mới:

1. **Đọc code client**
   - Tìm file xử lý trong `GlobalMessageHandler.cs`
   - Xem `GlobalService.cs` để biết client gửi gì
   - Xem `ReadMessenge.cs` để biết client nhận gì

2. **Phân tích packet**
   - Xác định command code
   - Xác định cấu trúc data (reader/writer)
   - Ghi chú vào `PACKET_REFERENCE.md`

3. **Implement trong Rust**
   - Tạo packet struct trong `network/packet/`
   - Tạo handler trong `network/handler/`
   - Register handler vào `PacketRegistry`
   - Test với client

4. **Testing**
   - Chạy server Rust
   - Chạy client Unity kết nối
   - Kiểm tra log
   - Debug nếu cần

5. **Documentation**
   - Update tài liệu
   - Ghi chú vấn đề đã gặp
   - Ghi chú giải pháp

---

## 💡 GHI CHÚ QUAN TRỌNG

### Protocol Insights
1. **Binary Format**: Tất cả packets đều là binary, không phải text/JSON
2. **Encryption**: XOR cipher với rotating key
3. **Message Structure**: `[command: i8][length: u16][data: Vec<u8>]`
4. **Endianness**: Big-endian cho integers
5. **String Encoding**: UTF-8 với length prefix

### Client Behavior
1. **Key Exchange**: Luôn là bước đầu tiên (CMD -40)
2. **Login Flow**: CMD 1 gọi 2 lần (initial + real login)
3. **Async Loading**: Client load data song song (images, templates)
4. **Reconnection**: Client tự động reconnect khi bị disconnect

### Common Pitfalls
1. ⚠️ **XOR Key**: Phải transform key trước khi gửi cho client
2. ⚠️ **Thread Safety**: Phải dùng Arc/Mutex cho shared state
3. ⚠️ **Buffer Size**: Client có buffer lớn (128KB), server cũng cần tương ứng
4. ⚠️ **Byte Order**: Big-endian vs Little-endian
5. ⚠️ **String Encoding**: UTF-8, có length prefix

---

## 📞 DEBUGGING TIPS

### Khi gặp lỗi:
1. Kiểm tra log file `connect-taidulieu-login.txt`
2. So sánh với `PROTOCOL_ANALYSIS.md`
3. Dùng Wireshark để capture packet
4. In hex dump của packet
5. Kiểm tra XOR encryption có đúng không

### Tools hữu ích:
- `hexdump -C` - Xem binary data
- Wireshark - Capture network traffic
- Unity Debug.Log - Client logging
- Rust `println!` & `dbg!` - Server logging

---

## 🎓 TÀI NGUYÊN HỌC TẬP

### Rust Resources
- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [Async Book](https://rust-lang.github.io/async-book/)

### Game Server Resources
- Game networking patterns
- Binary protocol design
- Anti-cheat techniques
- Server architecture patterns

---

## ✨ PROGRESS TRACKING

**Hiện tại đang ở**: Giai đoạn 1 - Core Network Layer

**Công việc tiếp theo**:
1. Hoàn thiện và test key exchange
2. Implement login handler (CMD 1)
3. Implement server info handler (CMD 37, 61)
4. Setup database cho authentication

**Vấn đề cần giải quyết**:
- [ ] Key exchange protocol cần test kỹ hơn
- [ ] Message queue system chưa có
- [ ] Database chưa được setup
- [ ] Logging system cần cải thiện

---

**Cập nhật lần cuối**: 02/01/2026  
**Người maintain**: Bạn & GitHub Copilot  
**Mục tiêu**: Hoàn thành MVP trong Q1 2026
