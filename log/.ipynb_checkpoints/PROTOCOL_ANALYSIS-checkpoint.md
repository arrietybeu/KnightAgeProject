# Knight Age Protocol Analysis

This document analyzes the client-server communication protocol based on the log file `connect-taidulieu-login.txt`.

## Table of Contents
1. [Connection Flow Overview](#connection-flow-overview)
2. [Phase 1: Key Exchange](#phase-1-key-exchange)
3. [Phase 2: Initial Login & Server Info](#phase-2-initial-login--server-info)
4. [Phase 3: Real Login](#phase-3-real-login)
5. [Phase 4: Game Data Loading](#phase-4-game-data-loading)
6. [Command Reference](#command-reference)
7. [Rust Implementation Guide](#rust-implementation-guide)

---

## Connection Flow Overview

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                          Knight Age Login Sequence                               │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                  │
│  CLIENT                                              SERVER                      │
│    │                                                    │                        │
│    │ ─────────── TCP Connect ─────────────────────────► │                        │
│    │                                                    │                        │
│    │ ══════════════════════════════════════════════════════════════════════════ │
│    │                     PHASE 1: KEY EXCHANGE                                   │
│    │ ══════════════════════════════════════════════════════════════════════════ │
│    │                                                    │                        │
│    │ ─────────── CMD -40 (0 bytes) ──────────────────► │  Key Exchange Request   │
│    │                                                    │                        │
│    │ ◄─────────── CMD -40 (2 bytes) ────────────────── │  Key Exchange Response  │
│    │                                                    │  [key_byte1, key_byte2] │
│    │                                                    │                        │
│    │ ══════════════════════════════════════════════════════════════════════════ │
│    │              PHASE 2: INITIAL LOGIN & SERVER INFO                           │
│    │ ══════════════════════════════════════════════════════════════════════════ │
│    │                                                    │                        │
│    │ ─────────── CMD 1 LOGIN (~37 bytes) ────────────► │  Login with placeholder │
│    │             (encrypted from now on)               │  data or minimal info   │
│    │                                                    │                        │
│    │ ─────────── CMD 61 NAME_SERVER (0 bytes) ───────► │  Request server names   │
│    │                                                    │                        │
│    │ ◄─────────── CMD 37 INFO_FROM_SERVER (82 bytes) ── │  Server info message   │
│    │                                                    │                        │
│    │ ◄─────────── CMD 61 NAME_SERVER (~3191 bytes) ──── │  World names, items,   │
│    │                                                    │  rebuild data          │
│    │                                                    │                        │
│    │ ══════════════════════════════════════════════════════════════════════════ │
│    │                   PHASE 3: REAL LOGIN                                       │
│    │ ══════════════════════════════════════════════════════════════════════════ │
│    │                                                    │                        │
│    │ ─────────── CMD 1 LOGIN (~55 bytes) ────────────► │  Login with real       │
│    │                                                    │  credentials           │
│    │                                                    │                        │
│    │ ◄─────────── CMD 63 DELETE_RMS (1 byte) ────────── │  Clear client cache    │
│    │                                                    │                        │
│    │ ══════════════════════════════════════════════════════════════════════════ │
│    │                PHASE 4: GAME DATA LOADING                                   │
│    │ ══════════════════════════════════════════════════════════════════════════ │
│    │                                                    │                        │
│    │ ◄─────────── CMD 26 CATALOG_MONSTER (26844 bytes)  │  Monster catalog       │
│    │                                                    │                        │
│    │ ◄─────────── CMD 25 ITEM_TEMPLATE (94954 bytes) ── │  Item templates        │
│    │                                                    │                        │
│    │ ◄─────────── CMD -57 UPDATE_DATA (4 bytes) ─────── │  Data version/checksum │
│    │                                                    │                        │
│    │ ◄─────────── CMD -52 LOAD_IMAGE_DATA_PART_CHAR ─── │  Character images      │
│    │              (multiple packets)                   │  (PNG data)            │
│    │                                                    │                        │
│    │ ══════════════════════════════════════════════════════════════════════════ │
│    │                    GAME SESSION BEGINS                                      │
│    │ ══════════════════════════════════════════════════════════════════════════ │
│    │                                                    │                        │
└─────────────────────────────────────────────────────────────────────────────────┘
```

---

## Phase 1: Key Exchange

### Client → Server: CMD -40 (Key Exchange Request)
- **Direction**: Client → Server (WRITE)
- **Command**: `-40` (0xD8)
- **Size**: 0 bytes (no data)
- **When**: Immediately after TCP connection established
- **Purpose**: Request encryption key from server

### Server → Client: CMD -40 (Key Exchange Response)
- **Direction**: Server → Client (READ)
- **Command**: `-40` (0xD8)
- **Size**: 2 bytes
- **Data Format**:
  ```
  [0]: Key type (0x01)
  [1]: Key seed (0xD1 in example)
  ```
- **Purpose**: Provide XOR encryption key to client

### Encryption Key Generation (from client code)
```csharp
// Client code reference from Session_ME.cs
private static void setKey(sbyte[] b)
{
    byte d = (byte)(b[1] < 0 ? b[1] + 256 : b[1]);
    for (int i = 0; i < 256; i++)
    {
        curR[i] = (sbyte)(i & 0xFF);
        curW[i] = (sbyte)(i & 0xFF);
    }
    for (int i = 0; i < 256; i++)
    {
        d = (byte)((curR[i] + d) & 0xFF);
        // Swap curR[i] and curR[d]
        sbyte tmp = curR[i];
        curR[i] = curR[d];
        curR[d] = tmp;
    }
    // curW is the original order, used for writing
    // curR is the shuffled order, used for reading
}
```

---

## Phase 2: Initial Login & Server Info

### Client → Server: CMD 1 (LOGIN)
- **Direction**: Client → Server (WRITE)
- **Command**: `1`
- **Size**: ~37 bytes (first login with placeholder)
- **Data Format** (from GlobalService.cs):
  ```
  | Field            | Type   | Example Value        | Description                    |
  |------------------|--------|----------------------|--------------------------------|
  | username         | String | "1"                  | User name (placeholder)        |
  | password         | String | "1"                  | Password (placeholder)         |
  | version          | String | "3.0.2"              | Client version                 |
  | clientProtocol   | String | "0"                  | Client protocol                |
  | pro              | String | "0"                  | Pro info                       |
  | agent            | String | "0"                  | Agent info                     |
  | zoomLevel        | i8     | 2                    | Graphics zoom level            |
  | device           | i8     | 4                    | Device type                    |
  | characterId      | i32    | -1                   | Character ID (-1 = new login)  |
  | area             | i8     | -1                   | Area ID (-1 = any)             |
  | isPC             | i8     | 1                    | 1 if PC, 0 if mobile           |
  | resolutionIndex  | i8     | -1                   | Resolution index               |
  | loginInfoIndex   | i8     | 1                    | Login info index               |
  | reserved         | i8     | 0                    | Reserved field                 |
  | charPartIndex    | i16    | varies               | Character part index           |
  | packageName      | String | "com.arriety.hso"    | Package name                   |
  ```

### Client → Server: CMD 61 (NAME_SERVER Request)
- **Direction**: Client → Server (WRITE)
- **Command**: `61`
- **Size**: 0 bytes
- **Purpose**: Request world/server name data

### Server → Client: CMD 37 (INFO_FROM_SERVER)
- **Direction**: Server → Client (READ)
- **Command**: `37`
- **Size**: ~82 bytes
- **Data Format** (from ReadMessenge.InfoServer_Download):
  ```
  | Field     | Type   | Description                           |
  |-----------|--------|---------------------------------------|
  | message   | String | Info/welcome message (UTF-8)          |
  | link      | String | Download link (empty if none)         |
  | type      | i8     | Message type (15 = special dialog)    |
  ```
- **Purpose**: Display server info/announcement to player

### Server → Client: CMD 61 (NAME_SERVER Response)
- **Direction**: Server → Client (READ)
- **Command**: `61`
- **Size**: ~3191 bytes
- **Data Format** (from ReadMessenge.name_server):
  ```
  | Field                      | Type         | Description                     |
  |----------------------------|--------------|----------------------------------|
  | numWorldNames              | u8           | Number of world/map names        |
  | worldNames[numWorldNames]  | String[]     | Array of world/map names         |
  | numQuestItems              | i8           | Number of quest item names       |
  | questItems[numQuestItems]  | String[]     | Array of quest item names        |
  | numMaterials               | i8           | Number of rebuild materials      |
  | materials[numMaterials]    | short[]      | Material IDs                     |
  | numRebuildData             | i8           | Number of rebuild data entries   |
  | rebuildData[numRebuild]    | struct[]     | Rebuild item data (see below)    |
  ```
  
  **RebuildData Structure**:
  ```
  | Field      | Type   | Description           |
  |------------|--------|-----------------------|
  | level      | i8     | Level requirement     |
  | priceCoin  | i32    | Coin price            |
  | priceGold  | i16    | Gold price            |
  | mValue[4]  | i8[4]  | 4 value bytes         |
  ```

---

## Phase 3: Real Login

### Client → Server: CMD 1 (LOGIN with credentials)
- **Direction**: Client → Server (WRITE)
- **Command**: `1`
- **Size**: ~55 bytes (real credentials)
- **Same format as Phase 2**, but with real username/password
- **Example Data Preview**: `00 0A 30 33 32 37 30 36 38 35 39 33 00 0A...`
  - `00 0A`: String length 10
  - `30 33 32 37...`: "0327068593" (username)

### Server → Client: CMD 63 (DELETE_RMS)
- **Direction**: Server → Client (READ)
- **Command**: `63`
- **Size**: 1 byte
- **Data Format**:
  ```
  | Field       | Type | Description                    |
  |-------------|------|--------------------------------|
  | resIndex    | i8   | Resource index for cache clear |
  ```
- **Purpose**: Tell client to clear cached data

---

## Phase 4: Game Data Loading

### Server → Client: CMD 26 (CATALOG_MONSTER)
- **Direction**: Server → Client (READ)
- **Command**: `26`
- **Size**: ~26844 bytes
- **Purpose**: Send complete monster catalog to client
- **Contains**: Monster definitions, stats, images

### Server → Client: CMD 25 (ITEM_TEMPLATE)
- **Direction**: Server → Client (READ)
- **Command**: `25`
- **Size**: ~94954 bytes
- **Purpose**: Send complete item template database to client
- **Contains**: All item definitions, stats, descriptions

### Server → Client: CMD -57 (UPDATE_DATA)
- **Direction**: Server → Client (READ)
- **Command**: `-57`
- **Size**: 4 bytes
- **Data Format**:
  ```
  | Field    | Type | Description              |
  |----------|------|--------------------------|
  | version  | i32  | Data version/checksum    |
  ```
- **Purpose**: Data version check for client caching

### Server → Client: CMD -52 (LOAD_IMAGE_DATA_PART_CHAR)
- **Direction**: Server → Client (READ)
- **Command**: `-52`
- **Size**: Variable (typically 3000-60000 bytes)
- **Data Format**:
  ```
  | Field       | Type    | Description                    |
  |-------------|---------|--------------------------------|
  | partType    | u8      | Character part type            |
  | partIndex   | u8      | Part index within type         |
  | unknown     | u8      | Unknown                        |
  | dataSize    | u16     | Size of image data             |
  | imageData   | byte[]  | PNG image data                 |
  ```
- **Purpose**: Send character sprite images to client
- **Note**: Sent multiple times for each character part

---

## Command Reference

### Client → Server Commands (WRITE)

| CMD  | Name              | Size     | Description                    | Required State    |
|------|-------------------|----------|--------------------------------|-------------------|
| -40  | KEY_EXCHANGE      | 0        | Request encryption key         | Connected         |
| 1    | LOGIN             | ~37-55   | Login request                  | KeyExchanged      |
| 61   | NAME_SERVER       | 0        | Request server/world names     | KeyExchanged+     |
| 4    | PLAYER_MOVE       | 4        | Player movement                | InGame            |
| 5    | CHAR_INFO         | 2        | Request character info         | InGame            |

### Server → Client Commands (READ)

| CMD  | Name                        | Size        | Description                    |
|------|-----------------------------|-------------|--------------------------------|
| -40  | KEY_EXCHANGE                | 2           | Encryption key response        |
| 1    | LOGIN_OK                    | Variable    | Login success + map table      |
| 2    | LOGIN_FAIL                  | Variable    | Login failure + message        |
| 25   | ITEM_TEMPLATE               | ~95000      | Item database                  |
| 26   | CATALOG_MONSTER             | ~27000      | Monster database               |
| 37   | INFO_FROM_SERVER            | ~82         | Server info/announcement       |
| 61   | NAME_SERVER                 | ~3200       | World/server names data        |
| 63   | DELETE_RMS                  | 1           | Clear client cache             |
| -52  | LOAD_IMAGE_DATA_PART_CHAR   | Variable    | Character sprite images        |
| -57  | UPDATE_DATA                 | 4           | Data version check             |

---

## Rust Implementation Guide

### 1. State Machine

```rust
pub enum ConnectionState {
    Connected,       // TCP connected, waiting for key exchange
    KeyExchanged,    // Key exchanged, ready for login
    Authed,          // Logged in, loading game data
    InGame,          // Fully loaded, in game
    Disconnected,    // Connection closed
}
```

### 2. Required Handlers (Priority Order)

```rust
// Phase 1: Key Exchange
- [✓] CMD -40: CmKeyExchange (already implemented in session.rs)

// Phase 2: Initial Login
- [✓] CMD 1:  CmLogin (parse login packet - already implemented)
- [ ] CMD 61: CmNameServer (client requests server names)

// Server Response Handlers (need to implement sending)
- [ ] CMD 37: SmInfoFromServer (send welcome message)
- [ ] CMD 61: SmNameServer (send world names, quest items, rebuild data)
- [ ] CMD 63: SmDeleteRms (send cache clear instruction)

// Phase 4: Game Data (later implementation)
- [ ] CMD 26: SmCatalogMonster (send monster database)
- [ ] CMD 25: SmItemTemplate (send item database)
- [ ] CMD -57: SmUpdateData (send data version)
- [ ] CMD -52: SmLoadImageDataPartChar (send character sprites)
```

### 3. Packet Handler Implementation Order

1. **First Priority**: Handle CMD 61 request and respond with CMD 61 + CMD 37
2. **Second Priority**: Handle real login (CMD 1 with credentials) and respond with CMD 63
3. **Third Priority**: Send game data (CMD 26, 25, -57, -52)
4. **Fourth Priority**: Handle in-game commands

### 4. Example Response Implementation

```rust
// Respond to CMD 61 NAME_SERVER request
pub fn send_name_server_response(ctx: &PacketContext) -> io::Result<()> {
    // First send INFO_FROM_SERVER (CMD 37)
    ctx.send_with(cmd::INFO_FROM_SERVER, |w| {
        w.write_string("Welcome to Knight Age!");  // message
        w.write_string("");                         // download link (empty)
        w.write_i8(0);                              // type
    }).await?;
    
    // Then send NAME_SERVER response (CMD 61)
    ctx.send_with(cmd::NAME_SERVER, |w| {
        // World names
        w.write_u8(world_names.len() as u8);
        for name in &world_names {
            w.write_string(name);
        }
        
        // Quest item names
        w.write_i8(quest_items.len() as i8);
        for item in &quest_items {
            w.write_string(item);
        }
        
        // Materials
        w.write_i8(materials.len() as i8);
        for material_id in &materials {
            w.write_short(*material_id);
        }
        
        // Rebuild data
        w.write_i8(rebuild_data.len() as i8);
        for data in &rebuild_data {
            w.write_i8(data.level);
            w.write_int(data.price_coin);
            w.write_short(data.price_gold);
            for val in &data.m_value {
                w.write_i8(*val);
            }
        }
    }).await?;
    
    Ok(())
}
```

### 5. Timing from Log Analysis

| Event | Timestamp | Delta |
|-------|-----------|-------|
| Key Exchange Request | 17:49:09.354 | - |
| Key Exchange Response | 17:49:09.380 | 26ms |
| Initial Login | 17:49:09.386 | 6ms |
| NAME_SERVER Request | 17:49:09.386 | 0ms |
| INFO_FROM_SERVER | 17:49:09.407 | 21ms |
| NAME_SERVER Response | 17:49:09.414 | 7ms |
| Real Login | 17:49:45.278 | ~36s (user input) |
| DELETE_RMS | 17:49:45.301 | 23ms |
| CATALOG_MONSTER | 17:49:45.311 | 10ms |
| ITEM_TEMPLATE | 17:49:45.436 | 125ms |
| UPDATE_DATA | 17:49:45.631 | 195ms |
| First Image Data | 17:49:45.636 | 5ms |

---

## Notes

1. **Encryption**: All packets after key exchange are XOR encrypted
2. **String Format**: Java-style modified UTF-8 (2-byte length prefix, big-endian)
3. **Byte Order**: Big-endian for all multi-byte values
4. **Character ID -1**: Indicates new login (no character selected yet)
5. **Area -1**: Indicates any area is acceptable

---

## Files Modified for Rust Server

1. `knight-age-server/src/network/handler/cm_login.rs` - Login packet parsing
2. `knight-age-server/src/network/handler/cm_server_info.rs` - NAME_SERVER handling (needs update)
3. `knight-age-server/src/network/session.rs` - Key exchange handling
4. `knight-age-server/src/network/opcode.rs` - Command constants

## Next Steps for Implementation

1. [ ] Implement CMD 61 (NAME_SERVER) request handler
2. [ ] Create response packet for CMD 37 (INFO_FROM_SERVER)
3. [ ] Create response packet for CMD 61 (NAME_SERVER response)
4. [ ] Implement CMD 63 (DELETE_RMS) response after successful login
5. [ ] Create placeholder data for monsters/items/images
6. [ ] Test full login flow with Unity client

