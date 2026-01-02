# PACKET REFERENCE - Knight Age Protocol Commands

> Tài liệu tham chiếu đầy đủ về tất cả các packet commands trong Knight Age, được trích xuất từ client C#.

---

## 📋 TABLE OF CONTENTS

- [Command Overview](#command-overview)
- [Authentication & Connection](#authentication--connection)
- [Character Management](#character-management)
- [Movement & Map](#movement--map)
- [Combat & Skills](#combat--skills)
- [Items & Inventory](#items--inventory)
- [NPC & Quest](#npc--quest)
- [Chat & Social](#chat--social)
- [Party & Clan](#party--clan)
- [Shop & Trading](#shop--trading)
- [Pet System](#pet-system)
- [Special Features](#special-features)
- [Data Loading](#data-loading)

---

## 📊 COMMAND OVERVIEW

### Command Range Distribution

```
Positive Commands (1 to 127):
  1-20:   Core game mechanics (login, movement, combat)
  21-40:  Items, skills, chat
  41-60:  Party, trading, areas
  61-80:  Clan, effects, special features
  81-90:  Pet, mount, ship

Negative Commands (-1 to -128):
  -30 to -40:  UI & Dialogs
  -44 to -57:  Resource loading
  -90 to -108: Advanced features
```

### Total Commands: ~100+ packets

---

## 🔐 AUTHENTICATION & CONNECTION

### CMD -40: Key Exchange
**Direction**: Bidirectional  
**Client → Server**: Request encryption key  
**Server → Client**: Send encryption key

**Client Request**:
```
Command: -40
Data: [] (empty)
```

**Server Response**:
```
Command: -40
Data: [key_length: byte][key_bytes...]
```

**Implementation**: [Session_ME.cs](../knight-age-client/Assets/Scripts/Assembly-CSharp/Session_ME.cs#L130-L140)

---

### CMD 1: Login
**Direction**: Bidirectional  
**Client → Server**: Login request  
**Server → Client**: Login result

**Client Request** (`GlobalService.login`):
```csharp
Command: 1
Data:
  - username: UTF
  - password: UTF
  - version: UTF
  - clientPro: UTF
  - pro: UTF
  - agent: UTF
  - zoomLevel: byte
  - device: byte
  - id: int
  - area: byte
  - isPC: byte
  - indexRes: byte
  - indexInfoLogin: byte
  - reserved: byte (0)
  - indexCharPar: short
  - packageName: UTF
```

**Server Response**: 
- Success → Character list or game data
- Failure → CMD 2 (Login Fail)

**Handler**: `GlobalMessageHandler.Login_Ok()` / `ReadMessenge.Login_Ok()`

---

### CMD 2: Login Fail
**Direction**: Server → Client  
**Purpose**: Notify login failure

**Server Response**:
```
Command: 2
Data: [error_message: UTF]
```

**Handler**: `GlobalMessageHandler.Login_Fail()` / `ReadMessenge.Login_Fail()`

---

### CMD 37: Server Info / Arena
**Direction**: Bidirectional

**Client Request** (`GlobalService.arena`):
```csharp
Command: 37
Data: [step: byte]
```

**Server Response**:
```
Command: 37
Data: [server_info_data...]  // 82 bytes in log
```

**Handler**: `GlobalMessageHandler.InfoServer_Download()` / `ReadMessenge.InfoServer_Download()`

---

### CMD 61: Server Names
**Direction**: Bidirectional

**Client Request** (`GlobalService.send_cmd_server`):
```csharp
Command: 61
Data: [] (empty)
```

**Server Response**:
```
Command: 61
Data: [world_names_data...]  // ~3191 bytes in log
```

**Handler**: `GlobalMessageHandler.NAME_SERVER()` / `ReadMessenge.name_server()`

---

### CMD 63: Delete RMS (Clear Cache)
**Direction**: Server → Client

**Server Response**:
```
Command: 63
Data: [flag: byte]
```

**Purpose**: Tell client to clear local cache

**Handler**: `ReadMessenge.delete_rms()`

---

## 👤 CHARACTER MANAGEMENT

### CMD 3: Main Character Info
**Direction**: Server → Client

**Server Response**:
```
Command: 3
Data: [character_data...]
```

**Handler**: `ReadMessenge.mainCharInfo()`

---

### CMD 5: Character Info
**Direction**: Bidirectional

**Client Request** (`GlobalService.char_info`):
```csharp
Command: 5
Data: [char_id: short]
```

**Server Response**:
```
Command: 5
Data: [character_details...]
```

**Handler**: `ReadMessenge.charInfo()`

---

### CMD 13: Character List / Select
**Direction**: Bidirectional

**Client Request** (`GlobalService.select_char`):
```csharp
Command: 13
Data:
  - typeSelect: byte
  - charId: int
```

**Server Response**:
```
Command: 13
Data: [character_list...]
```

**Handler**: `ReadMessenge.listChar()`

---

### CMD 14: Create Character
**Direction**: Client → Server

**Client Request** (`GlobalService.create_char`):
```csharp
Command: 14
Data:
  - classId: byte
  - name: UTF
  - hairStyle: byte
  - eyes: byte
  - head: byte
  - index: byte
```

---

### CMD 15: Character Wearing (Equipment)
**Direction**: Server → Client

**Server Response**:
```
Command: 15
Data: [equipment_data...]
```

**Handler**: `ReadMessenge.charWearing()`

---

### CMD 16: Character Inventory
**Direction**: Server → Client

**Server Response**:
```
Command: 16
Data: [inventory_data...]
```

**Handler**: `ReadMessenge.charInventory()`

---

### CMD 33: Level Up
**Direction**: Server → Client

**Server Response**:
```
Command: 33
Data: [level_data...]
```

**Handler**: `ReadMessenge.Level_Up()`

---

### CMD 30: Set XP
**Direction**: Server → Client

**Server Response**:
```
Command: 30
Data: [xp_data...]
```

**Handler**: `ReadMessenge.Set_XP()`

---

### CMD 31: Save Account Info
**Direction**: Server → Client

**Server Response**:
```
Command: 31
Data: [account_data...]
```

**Handler**: `ReadMessenge.writeUserAccountInfoToRMS()`

---

## 🗺️ MOVEMENT & MAP

### CMD 4: Player Move / Object Move
**Direction**: Bidirectional

**Client Request** (`GlobalService.player_move`):
```csharp
Command: 4
Data:
  - x: short
  - y: short
```

**Server Response**:
```
Command: 4
Data: [object_id, x, y...]
```

**Handler**: `ReadMessenge.objectMove()`

---

### CMD 8: Player Exit
**Direction**: Server → Client

**Server Response**:
```
Command: 8
Data: [player_id...]
```

**Handler**: `ReadMessenge.playerExit()`

---

### CMD 12: Change Map
**Direction**: Bidirectional

**Client Request** (`GlobalService.Ok_Change_Map`):
```csharp
Command: 12
Data: [] (empty)
```

**Server Response**:
```
Command: 12
Data: [map_data...]
```

**Handler**: `ReadMessenge.changeMap()`

---

### CMD 51: Change Area
**Direction**: Bidirectional

**Client Request** (`GlobalService.Change_Area`):
```csharp
Command: 51
Data: [area: byte]
```

**Server Response**:
```
Command: 51
Data: [area_data...]
```

**Handler**: `ReadMessenge.changeArea()`

---

### CMD 54: Update Area Status
**Direction**: Bidirectional

**Client Request** (`GlobalService.Request_Area`):
```csharp
Command: 54
Data: [] (empty)
```

**Server Response**:
```
Command: 54
Data: [area_status...]
```

**Handler**: `ReadMessenge.update_Status_Area()`

---

## ⚔️ COMBAT & SKILLS

### CMD 9: Fire Monster (Attack)
**Direction**: Client → Server

**Client Request** (`GlobalService.fire_monster`):
```csharp
Command: 9
Data:
  - typeKill: byte
  - numTargets: byte
  - [target_ids: short[]]
```

**Handler**: `ReadMessenge.fireMonster()`

---

### CMD 10: Monster Fire (Counter Attack)
**Direction**: Server → Client

**Server Response**:
```
Command: 10
Data: [monster_attack_data...]
```

**Handler**: `ReadMessenge.monsterFire()`

---

### CMD 17: Die Monster
**Direction**: Server → Client

**Server Response**:
```
Command: 17
Data: [monster_id...]
```

**Handler**: `ReadMessenge.dieMonster()`

---

### CMD 6: Fire PK (PvP Attack)
**Direction**: Client → Server

**Client Request** (`GlobalService.fire_Pk`):
```csharp
Command: 6
Data:
  - typeKill: byte
  - numTargets: byte
  - [target_ids...]
```

**Handler**: `ReadMessenge.firePK()`

---

### CMD 41: Die Player
**Direction**: Server → Client

**Server Response**:
```
Command: 41
Data: [player_id...]
```

**Handler**: `ReadMessenge.diePlayer()`

---

### CMD 42: PK Status
**Direction**: Bidirectional

**Client Request** (`GlobalService.set_Pk`):
```csharp
Command: 42
Data: [pk_mode: byte]
```

**Server Response**:
```
Command: 42
Data: [pk_data...]
```

**Handler**: `ReadMessenge.pk()`

---

### CMD 29: Skill List
**Direction**: Server → Client

**Server Response**:
```
Command: 29
Data: [skills_data...]
```

**Handler**: `ReadMessenge.Skill_List()`

---

### CMD 22: Add/Upgrade Skill
**Direction**: Bidirectional

**Client Request** (`GlobalService.Add_Base_Skill_Point`):
```csharp
Command: 22
Data:
  - type: byte
  - index: byte
  - value: short (optional)
```

**Server Response**:
```
Command: 22
Data: [skill_result...]
```

**Handler**: `ReadMessenge.onUpSkill()`

---

### CMD 40: Buff
**Direction**: Bidirectional

**Client Request** (`GlobalService.BuffMore`):
```csharp
Command: 40
Data:
  - type: byte
  - template: byte
  - [buff_data...]
```

**Server Response**:
```
Command: 40
Data: [buff_info...]
```

**Handler**: `ReadMessenge.Buff()`

---

### CMD 50: Effect Plus Time
**Direction**: Server → Client

**Server Response**:
```
Command: 50
Data: [effect_data...]
```

**Handler**: `ReadMessenge.eff_plus_time()`

---

### CMD 74: Num Effect
**Direction**: Server → Client

**Server Response**:
```
Command: 74
Data: [effect_num...]
```

**Handler**: `ReadMessenge.Num_Eff()`

---

### CMD 75: Effect From Server
**Direction**: Server → Client

**Server Response**:
```
Command: 75
Data: [effect_data...]
```

**Handler**: `ReadMessenge.EffFormServer()`

---

## 🎒 ITEMS & INVENTORY

### CMD 19: Item Drop
**Direction**: Server → Client

**Server Response**:
```
Command: 19
Data: [dropped_item...]
```

**Handler**: `ReadMessenge.ItemDrop()`

---

### CMD 20: Get Item from Map
**Direction**: Bidirectional

**Client Request** (`GlobalService.Get_Item_Map`):
```csharp
Command: 20
Data:
  - item_id: short
  - type: byte
```

**Server Response**:
```
Command: 20
Data: [pickup_result...]
```

**Handler**: `ReadMessenge.GetItemMap()`

---

### CMD 21: Item More Info
**Direction**: Bidirectional

**Client Request** (`GlobalService.Item_More_Info`):
```csharp
Command: 21
Data:
  - inVenWear: byte
  - id: byte
```

**Server Response**:
```
Command: 21
Data: [item_details...]
```

**Handler**: `ReadMessenge.Item_More_Info()`

---

### CMD 25: Item Template
**Direction**: Server → Client

**Server Response**:
```
Command: 25
Data: [item_templates...]  // ~94954 bytes in log
```

**Handler**: `ReadMessenge.itemTemplate()`

---

### CMD 28: Get Item Template
**Direction**: Bidirectional

**Client Request** (`GlobalService.getItemTem`):
```csharp
Command: 28
Data: [item_id: short]
```

**Server Response**:
```
Command: 28
Data: [item_template...]
```

**Handler**: `ReadMessenge.get_Item_Tem()`

---

### CMD 11: Use Item
**Direction**: Client → Server

**Client Request** (`GlobalService.Use_Item`):
```csharp
Command: 11
Data:
  - id: byte
  - index: byte
```

---

### CMD 32: Use Potion
**Direction**: Bidirectional

**Client Request** (`GlobalService.Use_Potion`):
```csharp
Command: 32
Data: [potion_id: short]
```

**Server Response**:
```
Command: 32
Data: [potion_effect...]
```

**Handler**: `ReadMessenge.use_Potion()`

---

### CMD 18: Delete Item
**Direction**: Client → Server

**Client Request** (`GlobalService.delete_Item`):
```csharp
Command: 18
Data:
  - type: byte
  - id: short
  - typeDelete: byte
```

---

### CMD 67: Rebuild Item
**Direction**: Bidirectional

**Client Request** (`GlobalService.Rebuild_Item`):
```csharp
Command: 67
Data:
  - type: byte
  - id: short
  - template: byte
```

**Server Response**:
```
Command: 67
Data: [rebuild_result...]
```

**Handler**: `ReadMessenge.Rebuild_Item()`

---

### CMD 73: Replace Plus Item
**Direction**: Bidirectional

**Client Request** (`GlobalService.Replace_Item`):
```csharp
Command: 73
Data:
  - type: byte
  - id: short
```

**Server Response**:
```
Command: 73
Data: [replace_result...]
```

**Handler**: `ReadMessenge.ReplacePlusItem()`

---

### CMD 77: Rebuild Wing
**Direction**: Bidirectional

**Client Request** (`GlobalService.Rebuild_Wing`):
```csharp
Command: 77
Data:
  - type: byte
  - wing: int
  - id: short
```

**Server Response**:
```
Command: 77
Data: [wing_result...]
```

**Handler**: `ReadMessenge.Rebuild_Wing()`

---

### CMD 78: Open Box
**Direction**: Server → Client

**Server Response**:
```
Command: 78
Data: [box_contents...]
```

**Handler**: `ReadMessenge.Open_Box()`

---

### CMD -100: Kham Ngoc (Socket Gem)
**Direction**: Bidirectional

**Client Request** (`GlobalService.KhamNgoc`):
```csharp
Command: -100
Data:
  - type: byte
  - idItem: int
  - idGem1: int
  - idGem2: int
  - idGem3: int
```

**Server Response**:
```
Command: -100
Data: [socket_result...]
```

**Handler**: `ReadMessenge.khamNgoc()`

---

### CMD 65: Character Chest
**Direction**: Bidirectional

**Client Request** (`GlobalService.Update_Char_Chest`):
```csharp
Command: 65
Data:
  - type: byte
  - id: short
  - template: byte
  - num: short
```

**Server Response**:
```
Command: 65
Data: [chest_data...]
```

**Handler**: `ReadMessenge.CharChest()`

---

### CMD -106: Material Template
**Direction**: Server → Client

**Server Response**:
```
Command: -106
Data: [material_templates...]
```

**Handler**: `ReadMessenge.Material_Template()`

---

## 🧙 NPC & QUEST

### CMD 23: NPC Info
**Direction**: Bidirectional

**Client Request** (`GlobalService.getlist_from_npc`):
```csharp
Command: 23
Data: [npc_id: byte]
```

**Server Response**:
```
Command: 23
Data: [npc_menu...]
```

**Handler**: `ReadMessenge.npcInfo()`

---

### CMD -44: New NPC Info
**Direction**: Bidirectional

**Client Request** (`GlobalService.new_npc_info`):
```csharp
Command: -44
Data: [npc_id: short]
```

**Server Response**:
```
Command: -44
Data: [npc_data...]
```

**Handler**: `ReadMessenge.newNPCInfo()`

---

### CMD -50: NPC Big (Large NPC Data)
**Direction**: Server → Client

**Server Response**:
```
Command: -50
Data: [npc_big_data...]
```

**Handler**: `ReadMessenge.npcBig()`

---

### CMD 60: Chat with NPC
**Direction**: Bidirectional

**Client Request** (`GlobalService.chat_npc`):
```csharp
Command: 60
Data: [npc_id: byte]
```

**Server Response**:
```
Command: 60
Data: [npc_chat...]
```

**Handler**: `ReadMessenge.chat_npc()`

---

### CMD 70: Update HP NPC
**Direction**: Server → Client

**Server Response**:
```
Command: 70
Data: [npc_hp...]
```

**Handler**: `ReadMessenge.updateHpNPC()`

---

### CMD 52: Quest
**Direction**: Bidirectional

**Client Request** (`GlobalService.quest`):
```csharp
Command: 52
Data:
  - id: short
  - main_sub: byte
  - type: byte
```

**Server Response**:
```
Command: 52
Data: [quest_info...]
```

**Handler**: `ReadMessenge.onReceiveInfoQuest()`

---

### CMD -30: Dynamic Menu
**Direction**: Bidirectional

**Client Request** (`GlobalService.Dynamic_Menu`):
```csharp
Command: -30
Data:
  - idNPC: short
  - idMenu: byte
  - index: byte
```

**Server Response**:
```
Command: -30
Data: [dynamic_menu...]
```

**Handler**: `ReadMessenge.Dynamic_Menu()`

---

### CMD -31: Dialog More Server
**Direction**: Bidirectional

**Client Request** (`GlobalService.sendMoreServerInfo`):
```csharp
Command: -31
Data:
  - idNPC: short
  - idMenu: short
  - [custom_data...]
```

**Server Response**:
```
Command: -31
Data: [dialog_data...]
```

**Handler**: `ReadMessenge.Dialog_More_server()`

---

### CMD -32: Dialog Server
**Direction**: Bidirectional

**Client Request** (`GlobalService.dialog_Server`):
```csharp
Command: -32
Data:
  - id: short
  - type: byte
  - value: byte
```

**Server Response**:
```
Command: -32
Data: [dialog...]
```

**Handler**: `ReadMessenge.Dialog_server()`

---

## 💬 CHAT & SOCIAL

### CMD 27: Chat Popup
**Direction**: Bidirectional

**Client Request** (`GlobalService.chatPopup`):
```csharp
Command: 27
Data: [message: UTF]
```

**Server Response**:
```
Command: 27
Data: [chat_data...]
```

**Handler**: `ReadMessenge.chatPopup()`

---

### CMD 34: Chat Tab
**Direction**: Bidirectional

**Client Request** (`GlobalService.chatTab`):
```csharp
Command: 34
Data:
  - name: UTF
  - message: UTF
```

**Server Response**:
```
Command: 34
Data: [chat_data...]
```

**Handler**: `ReadMessenge.chatTab()`

---

### CMD 35: Friend
**Direction**: Bidirectional

**Client Request** (`GlobalService.Friend`):
```csharp
Command: 35
Data:
  - type: byte
  - name: UTF
```

**Server Response**:
```
Command: 35
Data: [friend_data...]
```

**Handler**: `ReadMessenge.Friend()`

---

### CMD 71: Chat World
**Direction**: Client → Server

**Client Request** (`GlobalService.Chat_World`):
```csharp
Command: 71
Data: [message: UTF]
```

---

### CMD 39: Register
**Direction**: Client → Server

**Client Request** (`GlobalService.Register`):
```csharp
Command: 39
Data:
  - username: UTF
  - password: UTF
```

**Handler**: `ReadMessenge.Register()`

---

### CMD 49: Other Player Info
**Direction**: Bidirectional

**Client Request** (`GlobalService.Re_Info_Other_Object`):
```csharp
Command: 49
Data:
  - name: UTF
  - type: byte
```

**Server Response**:
```
Command: 49
Data: [player_info...]
```

**Handler**: `ReadMessenge.other_player_info()`

---

### CMD 57: List PK
**Direction**: Server → Client

**Server Response**:
```
Command: 57
Data: [pk_list...]
```

**Handler**: `ReadMessenge.List_Pk()`

---

## 👥 PARTY & CLAN

### CMD 48: Party
**Direction**: Bidirectional

**Client Request** (`GlobalService.Party`):
```csharp
Command: 48
Data:
  - type: byte
  - name: UTF
```

**Server Response**:
```
Command: 48
Data: [party_data...]
```

**Handler**: `ReadMessenge.Party()`

---

### CMD 69: Clan
**Direction**: Bidirectional

**Multiple functions**:
- `NextClan(type)` - Navigate clan
- `InvenClan(type)` - Clan inventory
- `Add_And_AnS_MemClan(type, name)` - Add/Answer member
- `ChucNang_Clan(type, id)` - Clan functions
- `gop_Xu_Luong_Clan(type, num)` - Donate money
- `info_Mem_Clan(type, name)` - Member info
- `PhongCap_Clan(type, chucvu, str)` - Promote/Demote
- `Delete_Mem_Clan(type, name)` - Remove member
- `Change_Slo_NoiQuy_Clan(type, name)` - Change rules

**Server Response**:
```
Command: 69
Data: [clan_data...]
```

**Handler**: `ReadMessenge.Clan()`

---

### CMD -104: Clan Chiem Thanh Info
**Direction**: Server → Client

**Server Response**:
```
Command: -104
Data: [territory_data...]
```

**Handler**: `ReadMessenge.infoclanChiemthanh()`

---

## 🛒 SHOP & TRADING

### CMD 24: Buy Item
**Direction**: Client → Server

**Client Request** (`GlobalService.buy_item`):
```csharp
Command: 24
Data:
  - typeBuy: byte
  - idBuy: short
  - quantity: short
```

---

### CMD 36: Buy/Sell
**Direction**: Bidirectional

**Client Request** (`GlobalService.Buy_Sell`):
```csharp
Command: 36
Data:
  - type: byte
  - name: UTF
  - typeItem: byte
  - idItem: short
  - money: int
```

**Server Response**:
```
Command: 36
Data: [trade_data...]
```

**Handler**: `ReadMessenge.Buy_Sell()`

---

### CMD -102: Store Info
**Direction**: Bidirectional

**Client Request** (`GlobalService.do_Buy_Sell_Item`):
```csharp
Command: -102
Data:
  - type: int
  - [items...]
  - slogan: UTF
  - idChar: short
  - idItem: int
  - category: byte
```

**Server Response**:
```
Command: -102
Data: [store_data...]
```

**Handler**: `ReadMessenge.StoreInfo()`

---

### CMD 53: Info Easy From Server
**Direction**: Server → Client

**Server Response**:
```
Command: 53
Data: [easy_info...]
```

**Handler**: `ReadMessenge.InfoEasyFromServer()`

---

## 🐾 PET SYSTEM

### CMD 44: Update Pet Container
**Direction**: Bidirectional

**Client Request** (`GlobalService.Update_Pet_Container`):
```csharp
Command: 44
Data:
  - type: byte
  - id: short
  - template: byte
  - num: short
```

**Server Response**:
```
Command: 44
Data: [pet_data...]
```

**Handler**: `ReadMessenge.UpdatePetContainer()`

---

### CMD 45: Pet Eat
**Direction**: Client → Server

**Client Request** (`GlobalService.Pet_Eat`):
```csharp
Command: 45
Data:
  - idPet: short
  - idItem: short
  - category: byte
  - type: byte
```

---

### CMD 84: Pet Attack
**Direction**: Server → Client

**Server Response**:
```
Command: 84
Data: [pet_attack...]
```

**Handler**: `ReadMessenge.petAttack()`

---

## 🎮 SPECIAL FEATURES

### CMD -97: Use Mount
**Direction**: Bidirectional

**Client Request** (`GlobalService.useMount`):
```csharp
Command: -97
Data: [type: byte]
```

**Server Response**:
```
Command: -97
Data: [mount_data...]
```

**Handler**: `ReadMessenge.useMount()`

---

### CMD -98: Use Ship
**Direction**: Bidirectional

**Client Request** (`GlobalService.useShip`):
```csharp
Command: -98
Data: [index: byte]
```

**Server Response**:
```
Command: -98
Data: [ship_data...]
```

**Handler**: `ReadMessenge.useShip()`

---

### CMD 68: Thach Dau (Challenge/Arena)
**Direction**: Bidirectional

**Client Request** (`GlobalService.Thach_Dau`):
```csharp
Command: 68
Data:
  - type: byte
  - name: UTF
```

**Server Response**:
```
Command: 68
Data: [arena_data...]
```

**Handler**: `ReadMessenge.Thach_Dau()`

---

### CMD -101: Thach Dau (Alt)
**Direction**: Bidirectional

**Client Request** (`GlobalService.doSendThachDau`):
```csharp
Command: -101
Data:
  - type: byte
  - name: UTF
```

**Server Response**:
```
Command: -101
Data: [arena_data...]
```

**Handler**: `ReadMessenge.ThachDau()`

---

### CMD -103: Mi Nuong Info
**Direction**: Bidirectional

**Client Request** (`GlobalService.RequestInfo_MiNuong`):
```csharp
Command: -103
Data:
  - type: byte
  - id: short
```

**Server Response**:
```
Command: -103
Data: [minuong_data...]
```

**Handler**: `ReadMessenge.MiNuongInfo()`

---

### CMD -105: Hop Rac (Recycle Bin)
**Direction**: Server → Client

**Server Response**:
```
Command: -105
Data: [recycle_data...]
```

**Handler**: `ReadMessenge.onHopRac()`

---

### CMD -108: Fill Rec Time
**Direction**: Server → Client

**Server Response**:
```
Command: -108
Data: [time_data...]
```

**Handler**: `ReadMessenge.onFillRec_Time()`

---

### CMD 59: Suc Khoe (Health)
**Direction**: Server → Client

**Server Response**:
```
Command: 59
Data: [health_data...]
```

**Handler**: `ReadMessenge.suckhoe()`

---

### CMD 62: X2 XP (Double Experience)
**Direction**: Server → Client

**Server Response**:
```
Command: 62
Data: [x2_data...]
```

**Handler**: `ReadMessenge.x2_Xp()`

---

### CMD 64: Help From Server
**Direction**: Server → Client

**Server Response**:
```
Command: 64
Data: [help_text...]
```

**Handler**: `ReadMessenge.Help_From_Server()`

---

### CMD 76: Effect Weather
**Direction**: Server → Client

**Server Response**:
```
Command: 76
Data: [weather_effect...]
```

**Handler**: `ReadMessenge.EffWeather()`

---

### CMD 85: Monster Detonate
**Direction**: Server → Client

**Server Response**:
```
Command: 85
Data: [explosion_data...]
```

**Handler**: `ReadMessenge.monsterDetonate()`

---

### CMD 86: Monster Skill Info
**Direction**: Server → Client

**Server Response**:
```
Command: 86
Data: [monster_skill...]
```

**Handler**: `ReadMessenge.monsterSkillInfo()`

---

### CMD -90 / CMD 90: Remove Actor
**Direction**: Server → Client

**Server Response**:
```
Command: -90 or 90
Data: [actor_id...]
```

**Handler**: `ReadMessenge.remove_Actor()`

---

### CMD -92: Use Item Arena
**Direction**: Server → Client

**Server Response**:
```
Command: -92
Data: [arena_item...]
```

**Handler**: `ReadMessenge.useItemArena()`

---

### CMD -94: Update Info Arena
**Direction**: Server → Client

**Server Response**:
```
Command: -94
Data: [arena_info...]
```

**Handler**: `ReadMessenge.UpdateInfoArena()`

---

### CMD -95: Update Mark Killer
**Direction**: Server → Client

**Server Response**:
```
Command: -95
Data: [killer_mark...]
```

**Handler**: `ReadMessenge.updateMarkKiller()`

---

### CMD -96: NPC Server
**Direction**: Server → Client

**Server Response**:
```
Command: -96
Data: [npc_server_data...]
```

**Handler**: `ReadMessenge.npcServer()`

---

## 📥 DATA LOADING

### CMD -51: Load Image
**Direction**: Bidirectional

**Client Request** (`GlobalService.load_image`):
```csharp
Command: -51
Data: [image_id: short]
```

**Server Response** (Special 4-byte length):
```
Command: -51
Data: [image_data...]
```

**Handler**: `ReadMessenge.loadImage()`

---

### CMD -52: Load Image Data Part Char
**Direction**: Bidirectional

**Client Request** (`GlobalService.load_image_data_part_char`):
```csharp
Command: -52
Data:
  - type: byte
  - id: short
```

**Server Response** (Special 4-byte length):
```
Command: -52
Data: [character_part_image...]
```

**Handler**: `ReadMessenge.loadImageDataCharPart()`

---

### CMD -49: Load Image Data Auto Effect
**Direction**: Bidirectional

**Client Request** (`GlobalService.load_image_data_auto_eff`):
```csharp
Command: -49
Data: [effect_id: short]
```

**Server Response**:
```
Command: -49
Data: [effect_image...]
```

**Handler**: `ReadMessenge.loadImageDataAutoEff()`

---

### CMD 26: Catalog Monster
**Direction**: Server → Client

**Server Response**:
```
Command: 26
Data: [monster_catalog...]  // ~26844 bytes in log
```

**Handler**: `ReadMessenge.catalogyMonster()`

---

### CMD -57: Update Data (Android)
**Direction**: Bidirectional

**Client Request** (`GlobalService.UpdateData`):
```csharp
Command: -57
Data: [] (empty)
```

**Server Response**:
```
Command: -57
Data: [update_info...]  // 4 bytes in log
```

**Handler**: `ReadMessenge.UpdateDataAndroid()`

---

### CMD -54: Compare Data (Android)
**Direction**: Server → Client

**Server Response**:
```
Command: -54
Data: [compare_data...]
```

**Handler**: `ReadMessenge.SoSanhDataAndroid()`

---

### CMD 7: Monster Info
**Direction**: Bidirectional

**Client Request** (`GlobalService.monster_info`):
```csharp
Command: 7
Data: [monster_id: short]
```

**Server Response**:
```
Command: 7
Data: [monster_details...]
```

**Handler**: `ReadMessenge.monsterInfo()`

---

### CMD 55: Save RMS Server
**Direction**: Bidirectional

**Client Request** (`GlobalService.Save_RMS_Server`):
```csharp
Command: 55
Data:
  - type: byte
  - id: byte
  - [data: byte[]]
```

**Server Response**:
```
Command: 55
Data: [save_result...]
```

**Handler**: `ReadMessenge.Save_RMS_Server()`

---

### CMD 56: List Servers
**Direction**: Bidirectional

**Client Request** (`GlobalService.set_Page`):
```csharp
Command: 56
Data: [page: byte]
```

**Server Response**:
```
Command: 56
Data: [server_list...]
```

**Handler**: `ReadMessenge.List_Serverz()`

---

## 💰 PAYMENT & TRANSACTIONS

### CMD -53: Nap Tien (Recharge/Payment)
**Direction**: Bidirectional

**Client Request** (`GlobalService.nap_tien`):
```csharp
Command: -53
Data:
  - type: short
  - numInfo: byte
  - [info: UTF[]]
```

**Server Response**:
```
Command: -53
Data: [payment_result...]
```

**Handler**: `ReadMessenge.nap_tien()`

---

### CMD -76: Payment Nokia
**Direction**: Client → Server

**Client Request** (`GlobalService.doPaymentNokia`):
```csharp
Command: -76
Data: [content: UTF]
```

**Handler**: `TemMidlet.handleMessage()`

---

### CMD -75: Payment Response
**Direction**: Server → Client

**Server Response**:
```
Command: -75
Data: [payment_response...]
```

**Handler**: `TemMidlet.handleMessage()`

---

### CMD -93: In-App Purchase
**Direction**: Bidirectional

**Client Request** (`GlobalService.requestInapPurchare`):
```csharp
Command: -93
Data:
  - type: byte
  - receipt: UTF
  - index: byte
```

**Server Response**:
```
Command: -93
Data: [] (empty - transaction cleared)
```

**Handler**: `Main.main.ClearTransaction()`

---

### CMD -91: Lottery
**Direction**: Bidirectional

**Client Request** (`GlobalService.request_LotteryItems` / `DoKhacItem`):
```csharp
Command: -91
Data: [lottery_data...]
```

**Server Response**:
```
Command: -91
Data: [lottery_result...]
```

**Handler**: `ReadMessenge.receiveLotteryReward()`

---

### CMD -28: Mon Capchar (Captcha)
**Direction**: Client → Server

**Client Request** (`GlobalService.Mon_Capchar`):
```csharp
Command: -28
Data: [captcha_num: byte]
```

---

## 📝 IMPLEMENTATION NOTES

### Priority Order for Rust Implementation

**Phase 1 - Core (Must Have)**:
1. CMD -40: Key Exchange ✅
2. CMD 1: Login
3. CMD 13: Character List/Select
4. CMD 3: Main Character Info
5. CMD 4: Movement
6. CMD 12: Change Map

**Phase 2 - Game Basics**:
7. CMD 9: Fire Monster
8. CMD 10: Monster Fire
9. CMD 17: Die Monster
10. CMD 19: Item Drop
11. CMD 20: Get Item
12. CMD 16: Inventory

**Phase 3 - Full Gameplay**:
- Skills (CMD 29, 22)
- Chat (CMD 27, 34)
- NPCs & Quests (CMD 23, 52)
- Equipment (CMD 15, 21)
- Party (CMD 48)

**Phase 4 - Advanced**:
- Clan (CMD 69)
- Trading (CMD 36, -102)
- Arena (CMD 68, -101)
- Effects & Buffs
- Special features

---

**Cập nhật lần cuối**: 02/01/2026  
**Tổng số commands**: ~100+  
**Đã implement**: CMD -40 (Key Exchange)  
**Tiếp theo**: CMD 1 (Login)
