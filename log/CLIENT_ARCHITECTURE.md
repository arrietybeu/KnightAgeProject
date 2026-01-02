# CLIENT ARCHITECTURE - Knight Age Unity Client

> Tài liệu này phân tích kiến trúc client Unity C# của Knight Age để hiểu cách client hoạt động và giao tiếp với server.

---

## 📐 KIẾN TRÚC TỔNG QUAN

### Layer Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    PRESENTATION LAYER                        │
│  (GameCanvas, GameScreen, UI Screens)                       │
├─────────────────────────────────────────────────────────────┤
│                    GAME LOGIC LAYER                          │
│  (Player, Monster, NPC, Item, Skill, Quest)                 │
├─────────────────────────────────────────────────────────────┤
│                    SERVICE LAYER                             │
│  (GlobalService, GlobalLogicHandler)                        │
├─────────────────────────────────────────────────────────────┤
│                    MESSAGE LAYER                             │
│  (GlobalMessageHandler, ReadMessenge)                       │
├─────────────────────────────────────────────────────────────┤
│                    NETWORK LAYER                             │
│  (Session_ME, Message, myReader, myWriter)                  │
├─────────────────────────────────────────────────────────────┤
│                    TRANSPORT LAYER                           │
│  (TCP Socket, BinaryReader, BinaryWriter)                   │
└─────────────────────────────────────────────────────────────┘
```

---

## 🌐 NETWORK LAYER - Session Management

### Session_ME.cs - Core Network Manager

**File**: `Session_ME.cs` (547 dòng)  
**Vai trò**: Singleton quản lý kết nối TCP, encryption, và message queue

#### Thành phần chính:

```csharp
public class Session_ME : ISession
{
    // Singleton instance
    protected static Session_ME instance = new Session_ME();
    
    // Network components
    private static TcpClient sc;
    private static NetworkStream dataStream;
    private static BinaryReader dis;
    private static BinaryWriter dos;
    
    // Threading
    private static Sender sender;              // Send thread
    private static Thread initThread;          // Connection thread
    private static Thread collectorThread;     // Receive thread
    
    // Encryption
    private static sbyte[] key = null;
    private static bool getKeyComplete = false;
    private static sbyte curR;  // Read position
    private static sbyte curW;  // Write position
    
    // State
    public static bool connected;
    public static bool connecting;
    
    // Statistics
    public static int sendByteCount;
    public static int recvByteCount;
}
```

#### Luồng hoạt động:

```
┌─────────────────────────────────────────────────────────────┐
│                    SESSION_ME WORKFLOW                       │
└─────────────────────────────────────────────────────────────┘

1. Connection Initialization
   ├── connect(host, port)
   ├── Create initThread
   └── NetworkInit()
       ├── doConnect()
       │   ├── new TcpClient()
       │   ├── Create BinaryReader/Writer
       │   ├── Start Sender thread
       │   ├── Start MessageCollector thread
       │   └── Send CMD -40 (Key Exchange)
       └── onConnectOK()

2. Sending Messages (Sender class)
   ├── sendMessage(Message)
   ├── Add to sendingMessage queue
   └── Sender.run() loop:
       ├── Wait for getKeyComplete
       ├── doSendMessage()
       │   ├── Encrypt command byte
       │   ├── Encrypt length (2 bytes)
       │   └── Encrypt data bytes
       └── Remove from queue

3. Receiving Messages (MessageCollector class)
   ├── MessageCollector.run() loop:
   │   ├── readMessage()
   │   │   ├── Read & decrypt command
   │   │   ├── Read & decrypt length
   │   │   ├── Read & decrypt data
   │   │   └── Create Message object
   │   ├── Check if CMD -40 (Key Exchange)
   │   │   ├── getKey()
   │   │   └── Set getKeyComplete = true
   │   └── onRecieveMsg(message)
   └── On error: onDisconnected()

4. Encryption (XOR Cipher)
   ├── readKey(byte)  - Decrypt với curR++
   └── writeKey(byte) - Encrypt với curW++
```

#### Thread Model:

```
Main Thread (Unity)
    │
    ├── initThread (Connection)
    │   └── Khởi tạo socket, reader/writer
    │
    ├── sendThread (Sender.run)
    │   └── Loop: Lấy message từ queue → Encrypt → Send
    │
    └── collectorThread (MessageCollector.run)
        └── Loop: Read → Decrypt → Dispatch to handler
```

---

## 📦 MESSAGE LAYER - Packet Processing

### Message.cs - Packet Structure

**File**: `Message.cs` (47 dòng)  
**Vai trò**: Đại diện cho một packet

```csharp
public class Message
{
    public sbyte command;           // Command ID (-128 to 127)
    private myReader dis;           // For reading data
    private myWriter dos;           // For writing data
    
    // Constructors
    Message(int command)            // For sending
    Message(sbyte command, sbyte[] data)  // For receiving
    
    // Methods
    sbyte[] getData()               // Get serialized data
    myReader reader()               // Get reader for deserialization
    myWriter writer()               // Get writer for serialization
}
```

### GlobalMessageHandler.cs - Message Router

**File**: `GlobalMessageHandler.cs` (444 dòng)  
**Vai trò**: Route messages đến handlers tương ứng

#### Switch-Case cho ~100+ commands:

```csharp
public void onMessage(Message msg)
{
    switch (msg.command)
    {
        // Authentication & Connection
        case 1:   Login_Ok(msg); break;
        case 2:   Login_Fail(msg); break;
        case -40: Key_Exchange(msg); break;  // Handled in Session_ME
        
        // Character & Player
        case 3:   mainCharInfo(msg); break;
        case 5:   charInfo(msg); break;
        case 8:   playerExit(msg); break;
        case 13:  listChar(msg); break;
        case 15:  charWearing(msg); break;
        case 16:  charInventory(msg); break;
        
        // Movement & Map
        case 4:   objectMove(msg); break;
        case 12:  changeMap(msg); break;
        
        // Monster & Combat
        case 7:   monsterInfo(msg); break;
        case 9:   fireMonster(msg); break;
        case 10:  monsterFire(msg); break;
        case 17:  dieMonster(msg); break;
        
        // Items
        case 19:  ItemDrop(msg); break;
        case 20:  GetItemMap(msg); break;
        case 21:  Item_More_Info(msg); break;
        case 25:  itemTemplate(msg); break;
        case 28:  get_Item_Tem(msg); break;
        
        // NPC & Quest
        case 23:  npcInfo(msg); break;
        case -44: newNPCInfo(msg); break;
        case 52:  onReceiveInfoQuest(msg); break;
        
        // UI & Dialog
        case -30: Dynamic_Menu(msg); break;
        case -31: Dialog_More_server(msg); break;
        case -32: Dialog_server(msg); break;
        
        // Chat & Social
        case 27:  chatPopup(msg); break;
        case 34:  chatTab(msg); break;
        case 35:  Friend(msg); break;
        
        // Skills
        case 29:  Skill_List(msg); break;
        case 30:  Set_XP(msg); break;
        case 33:  Level_Up(msg); break;
        
        // Server Info & Resources
        case 37:  InfoServer_Download(msg); break;
        case 61:  NAME_SERVER(msg); break;
        case -51: loadImage(msg); break;
        case -52: loadImageDataCharPart(msg); break;
        case -57: UpdateDataAndroid(msg); break;
        
        // ... và nhiều commands khác
    }
}
```

### GlobalService.cs - API Calls (Client → Server)

**File**: `GlobalService.cs` (1361 dòng)  
**Vai trò**: Tạo và gửi requests đến server

#### Ví dụ các API calls:

```csharp
public class GlobalService
{
    protected Message m;  // Current message being built
    
    // 1. LOGIN (CMD 1)
    public void login(string user, string pass, string version, ...)
    {
        init(1);  // Start new message with CMD 1
        m.writer().writeUTF(user);
        m.writer().writeUTF(pass);
        m.writer().writeUTF(version);
        m.writer().writeByte(zoomLevel);
        m.writer().writeByte(device);
        // ... more fields
        send();  // Send to Session_ME
    }
    
    // 2. PLAYER MOVE (CMD 4)
    public void player_move(short x, short y)
    {
        init(4);
        m.writer().writeShort(x);
        m.writer().writeShort(y);
        send();
    }
    
    // 3. FIRE MONSTER (CMD 9)
    public void fire_monster(mVector targets, sbyte typekill)
    {
        init(9);
        m.writer().writeByte(typekill);
        m.writer().writeByte(targets.size());
        for (int i = 0; i < targets.size(); i++)
        {
            Object_Effect_Skill obj = targets.elementAt(i);
            m.writer().writeShort(obj.ID);
        }
        send();
    }
    
    // 4. CHAR INFO (CMD 5)
    public void char_info(short id)
    {
        init(5);
        m.writer().writeShort(id);
        send();
    }
    
    // ... 100+ methods tương tự
}
```

---

## 🔧 BINARY I/O - myReader & myWriter

### myReader.cs - Deserialization

**File**: `myReader.cs` (245 dòng)

```csharp
public class myReader
{
    public sbyte[] buffer;      // Data buffer
    private int posRead;        // Current read position
    
    // Primitive types
    public sbyte readSByte()    // 1 byte signed
    public byte readByte()      // 1 byte unsigned
    public short readShort()    // 2 bytes big-endian
    public int readInt()        // 4 bytes big-endian
    public long readLong()      // 8 bytes big-endian
    
    // Strings
    public string readUTF()     // Length-prefixed UTF-8
    
    // Arrays
    public void read(ref sbyte[] data)
    
    // Utility
    public int available()      // Bytes remaining
    public void mark()          // Save position
    public void reset()         // Restore position
}
```

### myWriter.cs - Serialization

**File**: `myWriter.cs` (238 dòng)

```csharp
public class myWriter
{
    public sbyte[] buffer = new sbyte[2048];
    private int posWrite;
    
    // Primitive types
    public void writeSByte(sbyte value)
    public void writeByte(sbyte value)
    public void writeShort(short value)     // Big-endian
    public void writeInt(int value)         // Big-endian
    public void writeLong(long value)       // Big-endian
    
    // Strings
    public void writeUTF(string str)        // Length prefix + UTF-8
    
    // Arrays
    public void write(sbyte[] data)
    
    // Get result
    public sbyte[] getData()                // Get serialized buffer
}
```

#### Encoding Rules:

```
┌─────────────────────────────────────────────────────────────┐
│                    BINARY ENCODING FORMAT                    │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  1. Integers: Big-Endian (Network Byte Order)               │
│     short:  [high_byte][low_byte]                           │
│     int:    [b3][b2][b1][b0]                                │
│     long:   [b7][b6][b5][b4][b3][b2][b1][b0]               │
│                                                              │
│  2. Strings: UTF-8 with length prefix                       │
│     [length: ushort][utf8_bytes...]                         │
│                                                              │
│  3. Arrays: Length prefix + elements                        │
│     [count: byte/short][element1][element2]...              │
│                                                              │
│  4. Booleans: byte (0 = false, 1 = true)                   │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

---

## 🎮 GAME LOGIC LAYER

### Core Classes

#### Player.cs - Main Character
```csharp
public class Player : MainObject
{
    public int ID;
    public string name;
    public short x, y;              // Position
    public int hp, maxHp;           // Health
    public int mp, maxMp;           // Mana
    public int exp, maxExp;         // Experience
    public int level;
    public MainImage[] imgMain;     // Character sprites
    public mVector inventory;       // Items
    public mVector skills;          // Skills
    // ... nhiều fields khác
}
```

#### MainObject.cs - Base Game Object
```csharp
public class MainObject
{
    public short x, y;              // Position
    public sbyte state;             // State (idle, walk, attack, die)
    public sbyte Direction;         // Facing direction
    public int IDimg;               // Image ID
    // Common methods: paint(), update(), move()
}
```

#### Monster.cs - Enemy Entities
```csharp
public class Monster : MainObject
{
    public int ID;
    public string name;
    public int hp, maxHp;
    public sbyte level;
    public short typeMonster;       // Monster template ID
    // AI logic, attack patterns
}
```

---

## 🖼️ PRESENTATION LAYER

### Screen Hierarchy

```
GameScreen (Base)
    ├── LoginScreen
    ├── SelectCharScreen
    ├── CreateChar
    ├── MapScr (Main game screen)
    ├── Buy_Sell_Screen
    ├── TabScreenNew
    ├── Clan_Screen
    └── ... nhiều screens khác
```

### GameCanvas.cs - Main Controller

```csharp
public class GameCanvas
{
    public static GameScreen currentScreen;
    public static ReadMessenge readMessenge;
    public static GameScreen mapScr;
    
    // Game loop
    public void Update()    // Called every frame
    public void Paint()     // Render
    
    // Dialogs & Popups
    public static void start_Wait_Dialog(string msg, iCommand cmd)
    public static void start_Ok_Dialog(string info)
}
```

---

## 🔐 ENCRYPTION - XOR Cipher

### Key Exchange Flow

```
CLIENT                                  SERVER
  │                                       │
  │──────── CMD -40 (no data) ──────────►│  Request key
  │                                       │
  │                                       │  Generate key [k0, k1]
  │                                       │  Transform for client
  │                                       │
  │◄────── CMD -40 [t0, t1] ─────────────│  Send transformed key
  │                                       │
  │  Receive: [t0, t1]                   │
  │  Process:                             │
  │    key[0] = t0                        │
  │    key[1] = t1 ^ key[0]               │
  │  Result: [k0, k1]                     │
  │                                       │
  │  getKeyComplete = true                │
  │                                       │
  │═══ All subsequent packets encrypted ══│
```

### XOR Algorithm

```csharp
// Client side (Session_ME.cs)
private static sbyte readKey(sbyte b)
{
    sbyte result = (sbyte)(b ^ key[curR]);
    curR = (sbyte)((curR + 1) % key.Length);
    return result;
}

private static sbyte writeKey(sbyte b)
{
    sbyte result = (sbyte)(b ^ key[curW]);
    curW = (sbyte)((curW + 1) % key.Length);
    return result;
}
```

**Note**: 
- Mỗi byte được XOR với key ở vị trí tương ứng
- Vị trí key tăng dần (rotating key)
- Read position và Write position độc lập

---

## 📊 DATA MODELS

### Item System

```csharp
public class MainItem
{
    public short ID;
    public string name;
    public string info;
    public sbyte type;          // Weapon, Armor, Potion, etc.
    public sbyte tier;          // Quality/Rarity
    public short idIcon;        // Icon image ID
    public mImage img;          // Icon image
}

public class MainTemplateItem
{
    public short ID;
    public string name;
    public sbyte type;
    public short idIcon;
    public string info;
    public int price;
    // ... template properties
}
```

### Skill System

```csharp
public class Skill
{
    public short ID;
    public string name;
    public sbyte level;
    public int manaUse;
    public int coolDown;
    public sbyte type;          // Active, Passive, Buff
    public mImage imgIcon;
}
```

---

## 🔄 MESSAGE FLOW EXAMPLES

### Example 1: Login Flow

```
┌──────────────────────────────────────────────────────────────┐
│                        LOGIN SEQUENCE                         │
└──────────────────────────────────────────────────────────────┘

CLIENT                                      SERVER
  │                                           │
  │  User clicks "Login"                     │
  │  ↓                                        │
  │  GlobalService.login(user, pass, ...)    │
  │  ↓                                        │
  │────── CMD 1 LOGIN ──────────────────────►│
  │  Data:                                    │
  │    - username (UTF)                       │
  │    - password (UTF)                       │
  │    - version (UTF)                        │
  │    - platform info                        │
  │                                           │
  │                                           │  Validate credentials
  │                                           │  Load character data
  │                                           │
  │◄────── CMD 1 LOGIN_OK ───────────────────│
  │  Data:                                    │
  │    - session token                        │
  │    - user ID                              │
  │                                           │
  │  GlobalMessageHandler.onMessage()        │
  │  ↓                                        │
  │  case 1: Login_Ok(msg)                   │
  │  ↓                                        │
  │  ReadMessenge.Login_Ok(msg)              │
  │  ↓                                        │
  │  Switch to SelectCharScreen              │
  │                                           │
```

### Example 2: Player Movement

```
CLIENT                                      SERVER
  │                                           │
  │  User clicks on map                       │
  │  ↓                                        │
  │  GlobalService.player_move(x, y)         │
  │  ↓                                        │
  │────── CMD 4 MOVE ───────────────────────►│
  │  Data: [x: short][y: short]              │
  │                                           │
  │                                           │  Validate movement
  │                                           │  Update position
  │                                           │  Broadcast to nearby
  │                                           │
  │◄────── CMD 4 OBJECT_MOVE ────────────────│
  │  Data:                                    │
  │    - object_id                            │
  │    - new_x, new_y                         │
  │                                           │
  │  GlobalMessageHandler.onMessage()        │
  │  ↓                                        │
  │  case 4: objectMove(msg)                 │
  │  ↓                                        │
  │  Update player position on map           │
  │                                           │
```

---

## 🎯 KEY PATTERNS & CONVENTIONS

### Naming Conventions
- `CMD_XXX` hoặc số âm/dương cho command codes
- `Main*` prefix cho core classes (MainObject, MainItem, etc.)
- `Tab*` prefix cho UI tabs
- `*Screen` suffix cho screens
- `m*` prefix cho utility classes (mVector, mImage, mSystem)

### State Management
- Static variables cho global state (GameCanvas.player)
- Singleton pattern cho managers (Session_ME.gI())
- Screen stack cho UI navigation

### Threading Model
- Unity Main Thread cho game logic & rendering
- Network threads cho I/O
- Synchronization qua message queue

---

## 📝 IMPORTANT NOTES FOR RUST IMPLEMENTATION

### 1. Thread Safety
- Client dùng static variables nhiều → Server cần Arc<Mutex<T>>
- Message queue cần thread-safe → Dùng tokio::sync::mpsc

### 2. Binary Protocol
- Big-endian cho integers
- UTF-8 cho strings với length prefix
- XOR encryption với rotating key position

### 3. Message Ordering
- Client assume messages được xử lý theo thứ tự
- Server cần đảm bảo ordering trong session

### 4. Error Handling
- Client ít error handling → Server phải robust hơn
- Client tự reconnect khi disconnect

### 5. State Synchronization
- Client maintain local state
- Server là source of truth
- Cần validation cho mọi client request

---

**Cập nhật lần cuối**: 02/01/2026  
**Mục đích**: Hiểu kiến trúc client để implement server tương thích
