
**swagger : http://api-v1.solji.fun:8091/swagger-ui/**

**服务地址： https://api-v1.solji.fun**

## 

## API 端点总览

### 基础接口

- `GET /health` - 健康检查
- `GET /api/stats` - 全局统计数据
- `GET /api/temple/level` - 寺庙等级信息
- `GET /api/temple/stats` - 寺庙统计信息
- `GET /api/temple/activities/recent` - 最近操作列表

### 香火相关接口

- `GET /api/incense/types` - 获取香火类型
- `GET /api/incense/can-burn` - 检查是否可以燃烧香火
- `GET /api/incense/user/{user_pubkey}/burn-count` - 获取用户香火燃烧次数（所有香型）
- `GET /api/incense/user/{user_pubkey}/burn-count/{incense_type}` - 获取用户指定香型的燃烧次数
- `GET /api/incense/user/{user_pubkey}/nfts` - 获取用户香火 NFT
- `GET /api/incense/user/{user_pubkey}/history` - 获取用户香火燃烧历史
- `GET /api/incense/leaderboard` - 香火排行榜

### 许愿相关接口

- `GET /api/wishes` - 分页获取许愿列表
- `GET /api/wishes/public` - 获取公开许愿
- `GET /api/wishes/user/{user_pubkey}` - 获取用户许愿
- `GET /api/wishes/user/{user_pubkey}/count` - 获取用户每日许愿次数
- `GET /api/wish-tower/{user_pubkey}` - 获取用户许愿塔信息
- `POST /api/wishes/{wish_id}/like` - 点赞许愿

### 捐赠相关接口

- `GET /api/donation/leaderboard` - 捐赠排行榜
- `GET /api/donation/check-top-10000` - 检查是否为前10000捐赠者
- `GET /api/donation/tiers` - 获取捐赠等级信息
- `GET /api/donation/user/{user_pubkey}/history` - 获取用户捐赠历史
- `GET /api/donation/user/{user_pubkey}/badges` - 获取用户捐赠徽章
- `GET /api/donation/honor-wall` - 荣誉墙
- `POST /api/donation/submit` - 提交捐赠交易

### 护身符相关接口

- `GET /api/amulet/user/{user_pubkey}/pending` - 获取用户待领取护身符
- `GET /api/amulet/user/{user_pubkey}/recent-drop` - 获取用户最近护身符掉落
- `GET /api/amulet/user/{user_pubkey}/owned` - 获取用户拥有的护身符

### IPFS 相关接口

- `POST /api/ipfs/upload` - 上传内容到 IPFS
- `GET /api/ipfs/{hash}` - 通过哈希获取 IPFS 内容
- `POST /api/ipfs/batch` - 批量获取多个 IPFS 内容

### 个人信息相关接口

- `GET /api/profile/{user_pubkey}/basic` - 获取用户基本信息
- `GET /api/profile/{user_pubkey}/activities` - 获取用户活动历史
- `GET /api/profile/{user_pubkey}/achievements` - 获取用户成就
- `GET /api/profile/{user_pubkey}/nfts` - 获取用户NFT

## 详细接口文档

### 基础接口

### 健康检查

```
GET /health
```

**响应：**

```json
{
  "status": "ok",
  "service": "solji-indexer",
  "version": "0.1.0"
}
```

### 全局统计

```
GET /api/stats
```

**响应：**

```json
{
  "total_merit": 1000000,
  "total_incense_points": 500000,
  "total_donations_sol": 1000.5,
  "total_users": 10000,
  "total_wishes": 50000,
  "total_donations": 1500,
  "total_donation_amount": 1000.5,
  "total_merit_distributed": 800000,
  "total_incense_points_distributed": 400000,
  "total_draw_fortune": 25000,
  "updated_at": 1732000000,
  "created_at": "2025-01-01T00:00:00Z"
}
```

### 寺庙等级

```
GET /api/temple/level
```

**响应：**

```json
{
  "current_level": 2,
  "level_name": "赤庙",
  "level_name_en": "Vibrant Shrine",
  "stats": {
    "total_incense_points": 500000,
    "total_draw_fortune": 25000,
    "total_wishes": 50000,
    "total_donations_sol": 1000.5,
    "total_fortune_nfts": 1000
  },
  "next_level_requirements": {
    "level": 3,
    "level_name": "灵殿",
    "level_name_en": "Temple of Spirit",
    "requirements": {
      "incense_points": 500000,
      "draw_fortune": 30000,
      "wishes": 10000,
      "donations_sol": 1000.0,
      "fortune_nfts": 0
    }
  },
  "progress_percentage": 75.5,
  "updated_at": 1732000000
}
```

## 香火相关接口

### 获取香火类型

```
GET /api/incense/types
```

**响应：**

```json
{
  "incense_types": [
    {
      "id": "basic",
      "name": "清香",
      "name_en": "Clear Incense",
      "price": 0.01,
      "merit_points": 10,
      "description": "Simple and pure, for daily devotion. Drops amulets with chance.",
      "image": "/traditional-incense-stick-glowing.jpg",
      "daily_limit": 10
    },
    {
      "id": "sandalwood",
      "name": "檀香",
      "name_en": "Sandalwood",
      "price": 0.05,
      "merit_points": 65,
      "description": "Premium sandalwood for deeper meditation. Unlocks dynamic NFT incense effects.",
      "image": "/sandalwood-incense-with-golden-glow.jpg",
      "daily_limit": 10
    },
    {
      "id": "dragon",
      "name": "龙涎香",
      "name_en": "Ambergris Incense",
      "price": 0.1,
      "merit_points": 1200,
      "description": "Rare ambergris incense for great fortune. Can be used to upgrade temples.",
      "image": "/mystical-dragon-incense-with-purple-smoke.jpg",
      "daily_limit": 10
    },
    {
      "id": "supreme",
      "name": "太上灵香",
      "name_en": "Supreme Spirit Incense",
      "price": 0.3,
      "merit_points": 3400,
      "description": "The ultimate offering for enlightenment. Unlocks limited amulet NFTs + extra rewards.",
      "image": "/supreme-golden-incense-with-rainbow-aura.jpg",
      "daily_limit": 10
    },
    {
      "id": "secret",
      "name": "秘制香",
      "name_en": "Secret Brew Incense",
      "price": 5.0,
      "merit_points": 12000,
      "description": "Exclusive incense obtained through donations (5 SOL for 10 sticks). Unlocks Buddha NFTs.",
      "image": "https://solji.mypinata.cloud/ipfs/bafkreiesfvlpyunybdl22oogzj2kxaado3hpblwqmd3q45pcu4imyzj3ha",
      "daily_limit": 10
    },
    {
      "id": "celestial",
      "name": "天界香",
      "name_en": "Celestial Incense",
      "price": 50.0,
      "merit_points": 300000,
      "description": "Heavenly incense obtained through donations (50 SOL for 5 sticks). Unlocks Buddha NFTs.",
      "image": "https://solji.mypinata.cloud/ipfs/bafkreiesfvlpyunybdl22oogzj2kxaado3hpblwqmd3q45pcu4imyzj3ha",
      "daily_limit": 10
    }
  ]
}
```

### 获取最近活动

```
GET /api/temple/activities/recent
```

**响应：**

```json
{
  "wishes": [
    {
      "id": 1,
      "wish_id": 12345,
      "user_pubkey": "UserPubkey...",
      "content_hash": [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32],
      "likes": 10,
      "is_liked": false,
      "created_at": "2025-01-01T10:00:00Z"
    }
  ],
  "pagination": {
    "limit": 20,
    "offset": 0,
    "count": 1
  }
}

```
GET /api/incense/can-burn?user={user_pubkey}&incense_type={type}&amount={amount}
```

**查询参数：**

- `user`: 用户钱包地址
- `incense_type`: 香火类型 ID (数字)
- `amount`: 燃烧数量

**响应：**

```json
{
  "can_burn": true,
  "user": "UserPubkey...",
  "incense_type": 1,
  "requested_amount": 5,
  "max_daily_limit": 10
}
```

### 用户香火燃烧次数

```
GET /api/incense/user/{user_pubkey}/burn-count
```

**响应：**

```json
{
  "user_pubkey": "UserPubkey...",
  "burn_counts": [
    {
      "incense_id": 1,
      "count": 3
    }
  ],
  "max_daily_limit": 10
}
```

### 用户指定香型燃烧次数

```
GET /api/incense/user/{user_pubkey}/burn-count/{incense_type}
```

**路径参数：**

- `user_pubkey`: 用户公钥
- `incense_type`: 香火类型 ID (数字，0-5)

**响应：**

```json
{
  "user_pubkey": "UserPubkey...",
  "incense_type": 1,
  "burn_count": 5,
  "max_daily_limit": 10
}
```

**错误响应：**

- `400`: 无效的香火类型参数
- `500`: 服务器内部错误

### 获取用户香火燃烧历史

```
GET /api/incense/user/{user_pubkey}/history?limit=20
```

**路径参数：**

- `user_pubkey`: 用户公钥

**查询参数：**

- `limit`: 返回记录数量 (默认20, 最大100)

**响应：**

```json
{
  "user_pubkey": "5xot9PdcigoDgdXJYuSGKmHBhcQn3WHPh1EwLyBNxmNw",
  "history": [
    {
      "id": 1,
      "incense_type": 0,
      "incenseId": "basic",
      "name": "清香",
      "nameEn": "Clear Incense",
      "image": "https://solji.mypinata.cloud/ipfs/QmfE3pH44ef4iHHS7Vv81aDomY7yTzUtPnKxcBtZXyMkh4",
      "serial": 1,
      "incense_amount": 1,
      "meritPoints": 10,
      "incense_points_gained": 100,
      "transactionSignature": "签名",
      "mintedAt": "2025-01-01T10:00:00Z"
    },
    {
      "id": 2,
      "incense_type": 1,
      "incenseId": "sandalwood",
      "name": "檀香",
      "nameEn": "Sandalwood",
      "image": "https://solji.mypinata.cloud/ipfs/QmYBz666XhqdQtizZYgg4C6EH3cKKKDPRdNDZZ4SEcAxDD",
      "serial": 2,
      "incense_amount": 1,
      "meritPoints": 65,
      "incense_points_gained": 600,
      "transactionSignature": "签名2",
      "mintedAt": "2025-01-01T11:00:00Z"
    }
  ],
  "count": 2
}
```

**字段说明：**

- `id`: 记录ID
- `incense_type`: 香火类型数字ID (0-5)
- `incenseId`: 香火类型字符串ID
- `name`: 中文名称
- `nameEn`: 英文名称
- `image`: IPFS图片URL
- `serial`: 用户NFT连续编号
- `incense_amount`: 燃烧数量
- `meritPoints`: 获得功德点数
- `incense_points_gained`: 获得香火点数
- `transactionSignature`: 交易签名
- `mintedAt`: NFT铸造时间

## 许愿相关接口

### 分页获取许愿

```
GET /api/wishes?limit=20&offset=0
X-User-Pubkey: {user_pubkey}
```

**查询参数：**

- `limit`: 每页数量 (默认20, 最大100)
- `offset`: 偏移量 (默认0)

**请求头：**

- `X-User-Pubkey`: 用户公钥 (可选，用于检查点赞状态)

**响应：**

```json
{
  "wishes": [
    {
      "id": 1,
      "wish_id": 12345,
      "user_pubkey": "UserPubkey...",
      "content": "祈求平安健康",
      "likes": 10,
      "is_liked": false,
      "created_at": "2025-01-01T10:00:00Z"
    }
  ],
  "pagination": {
    "limit": 20,
    "offset": 0,
    "count": 1
  }
}
```

### 获取公开许愿

```
GET /api/wishes/public?limit=20&offset=0
X-User-Pubkey: {user_pubkey}
```

**查询参数：**

- `limit`: 每页数量 (默认20, 最大100)
- `offset`: 偏移量 (默认0)

**请求头：**

- `X-User-Pubkey`: 用户公钥 (可选，用于检查点赞状态)

**响应：**

```json
{
  "wishes": [
    {
      "id": 1,
      "wish_id": 12345,
      "user_pubkey": "UserPubkey...",
      "content": "公开许愿内容",
      "likes": 5,
      "is_liked": false,
      "created_at": "2025-01-01T10:00:00Z"
    }
  ],
  "pagination": {
    "limit": 20,
    "offset": 0,
    "count": 1
  }
}
```

### 获取用户许愿

```
GET /api/wishes/user/{user_pubkey}?limit=20&offset=0
X-User-Pubkey: {viewer_pubkey}
```

**路径参数：**

- `user_pubkey`: 用户公钥

**查询参数：**

- `limit`: 每页数量 (默认20, 最大100)
- `offset`: 偏移量 (默认0)

**请求头：**

- `X-User-Pubkey`: 查看者公钥 (可选，用于检查点赞状态)

**响应：**

```json
{
  "user_pubkey": "UserPubkey...",
  "wishes": [
    {
      "id": 1,
      "wish_id": 12345,
      "user_pubkey": "UserPubkey...",
      "content": "用户许愿内容",
      "likes": 3,
      "is_liked": true,
      "created_at": "2025-01-01T10:00:00Z"
    }
  ],
  "pagination": {
    "limit": 20,
    "offset": 0,
    "count": 1
  }
}
```

### 点赞许愿

```
POST /api/wishes/{wish_id}/like
X-User-Pubkey: {user_pubkey}
```

**路径参数：**

- `wish_id`: 许愿ID

**请求头：**

- `X-User-Pubkey`: 用户公钥 (必需)

**响应：**

```json
{
  "wish_id": 12345,
  "likes": 11,
  "success": true
}
```

**错误响应：**

- `400`: 缺少 X-User-Pubkey 请求头
- `409`: 用户已经点赞过此许愿
- `500`: 服务器内部错误

### 用户许愿塔信息

```
GET /api/wish-tower/{user_pubkey}
```

**响应：**

```json
{
  "user_pubkey": "UserPubkey...",
  "total_wishes": 15,
  "level": 2,
  "last_updated": "2025-01-01T10:00:00Z"
}
```

## 捐赠相关接口

### 获取捐赠等级

```
GET /api/donation/tiers
```

**响应：**

```json
{
  "tiers": [
    {
      "tier": "bronze",
      "name": "铜牌信士",
      "name_en": "Bronze Devotee",
      "min_amount": 0.05,
      "merit_points": 65,
      "badge": "入门功德铜章 NFT",
      "benefits": ["点亮香火墙名字"]
    },
    {
      "tier": "silver",
      "name": "银牌居士",
      "name_en": "Silver Layman",
      "min_amount": 0.2,
      "merit_points": 1300,
      "badge": "精进银章 NFT",
      "benefits": ["可为寺庙投票提案"]
    },
    {
      "tier": "gold",
      "name": "金牌护法",
      "name_en": "Gold Guardian",
      "min_amount": 1.0,
      "merit_points": 14000,
      "badge": "护法金章 NFT",
      "benefits": ["可参与寺庙 NFT 治理"]
    },
    {
      "tier": "supreme",
      "name": "至尊供奉",
      "name_en": "Supreme Patron",
      "min_amount": 5.0,
      "merit_points": 120000,
      "badge": "至尊龙章 NFT",
      "benefits": ["解锁彩蛋内容+寺庙共建者身份"]
    }
  ]
}
```

### 提交捐赠交易

```
POST /api/donation/submit
Content-Type: application/json

{
  "user_pubkey": "UserPubkey...",
  "amount_sol": 0.1,
  "tier": "bronze",
  "transaction_signature": "5xXxX..."
}
```

**响应：**

```json
{
  "success": true,
  "tier": "bronze",
  "merit_gained": 65,
  "badge_minted": false,
  "nft_mint": null,
  "transaction_signature": "5xXxX..."
}
```

## 护身符相关接口

### 获取待领取护身符

```
GET /api/amulet/user/{user_pubkey}/pending
```

**响应：**

```json
{
  "user_pubkey": "UserPubkey...",
  "pending_amulets": [
    {
      "id": 1,
      "user_pubkey": "UserPubkey...",
      "amulet_type": 0,
      "source": "burn_incense",
      "created_at": "2025-01-01T10:00:00Z"
    }
  ],
  "count": 1
}
```

### 获取最近护身符掉落

```
GET /api/amulet/user/{user_pubkey}/recent-drop
```

**响应：**

```json
{
  "user_pubkey": "UserPubkey...",
  "has_recent_drop": true,
  "recent_drop": {
    "id": 1,
    "amulet_type": 0,
    "source": "burn_incense",
    "created_at": "2025-01-01T10:00:00Z",
    "time_since_drop_seconds": 3600
  }
}
```

## IPFS 相关接口

### 上传内容到 IPFS

```
POST /api/ipfs/upload
Content-Type: application/json

{
  "content": "要上传到 IPFS 的文本内容"
}
```

**请求参数：**

- `content`: 要上传的文本内容 (必需，最大 1MB)

**响应：**

```json
{
  "hash": "QmXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX",
  "url": "<https://solji.mypinata.cloud/ipfs/QmXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX>",
  "size": 1024
}
```

### 通过哈希获取 IPFS 内容

```
GET /api/ipfs/{hash}
```

**路径参数：**

- `hash`: IPFS 内容哈希/CID

**响应：**

```json
{
  "hash": "QmXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX",
  "content": "存储在 IPFS 上的内容",
  "content_type": "text/plain",
  "size": 1024,
  "gateway_url": "<https://solji.mypinata.cloud/ipfs/QmXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX>"
}
```

### 批量获取多个 IPFS 内容

```
POST /api/ipfs/batch
Content-Type: application/json

{
  "hashes": [
    "QmXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX",
    "QmYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYY"
  ]
}
```

**请求参数：**

- `hashes`: IPFS 哈希数组 (必需，最多 50 个)

**响应：**

```json
{
  "contents": [
    {
      "hash": "QmXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX",
      "content": "第一个文件的内容",
      "content_type": "text/plain",
      "size": 1024,
      "gateway_url": "<https://solji.mypinata.cloud/ipfs/QmXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX>"
    },
    {
      "hash": "QmYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYY",
      "content": "第二个文件的内容",
      "content_type": "application/json",
      "size": 2048,
      "gateway_url": "<https://solji.mypinata.cloud/ipfs/QmYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYY>"
    }
  ],
  "errors": []
}
```

## 个人信息相关接口

### 获取用户基本信息

```
GET /api/profile/{user_pubkey}/basic
```

**响应：**

```json
{
  "user_pubkey": "UserPubkey...",
  "merit_points": 1520,
  "rank": "供奉",
  "joined_date": "2025-01-01T00:00:00Z",
  "stats": {
    "total_incense_burned": 45,
    "total_fortunes_drawn": 23,
    "total_wishes_made": 12,
    "total_donated_sol": 1.5
  }
}
```

### 获取用户活动历史

```
GET /api/profile/{user_pubkey}/activities
```

**查询参数：**

- `limit`: 每页数量 (默认20, 最大100)
- `offset`: 偏移量 (默认0)

**响应：**

```json
{
  "user_pubkey": "UserPubkey...",
  "activities": [
    {
      "activity_type": "incense_burn",
      "description": "Burned Supreme Incense",
      "merit_gained": 30,
      "created_at": "2025-01-01T12:00:00Z"
    },
    {
      "activity_type": "fortune_draw",
      "description": "Drew Great Fortune",
      "merit_gained": 2,
      "created_at": "2025-01-01T11:45:00Z"
    }
  ],
  "pagination": {
    "limit": 20,
    "offset": 0,
    "count": 2
  }
}
```

### 获取用户成就

```
GET /api/profile/{user_pubkey}/achievements
```

**响应：**

```json
{
  "user_pubkey": "UserPubkey...",
  "achievements": [
    {
      "title": "First Incense",
      "description": "Burned your first incense",
      "unlocked": true,
      "unlocked_at": "2025-01-01T10:00:00Z"
    },
    {
      "title": "Fortune Seeker",
      "description": "Drew 10 fortunes",
      "unlocked": true,
      "unlocked_at": "2025-01-01T11:00:00Z"
    },
    {
      "title": "Temple Master",
      "description": "Reach Temple Master rank",
      "unlocked": false,
      "unlocked_at": null
    }
  ]
}
```

### 获取用户NFT

```
GET /api/profile/{user_pubkey}/nfts
```

**响应：**

```json
{
  "user_pubkey": "UserPubkey...",
  "nfts": {
    "amulet_count": 3,
    "fortune_nft_count": 5,
    "buddha_nft_count": 1,
    "total_count": 9
  },
  "collections": [
    {
      "type": "amulet",
      "name": "护身符",
      "count": 3,
      "items": [
        {
          "id": 1,
          "name": "平安符",
          "rarity": "common",
          "minted_at": "2025-01-01T10:00:00Z"
        }
      ]
    }
  ]
}
```

## 排行榜接口

### 香火排行榜

```
GET /api/incense/leaderboard?period=all
```

**查询参数：**

- `period`: 时间周期 (all, daily, weekly, monthly)

**响应：**

```json
{
  "period": "all",
  "leaderboard": [
    {
      "rank": 1,
      "user_pubkey": "UserPubkey...",
      "total_incense_points": 10000,
      "burn_count": 500
    }
  ],
  "count": 100
}
```

### 捐赠排行榜

```
GET /api/donation/leaderboard?limit=100&offset=0
```

**响应：**

```json
{
  "leaderboard": [
    {
      "rank": 1,
      "user_pubkey": "UserPubkey...",
      "total_donated": 100.5
    }
  ],
  "pagination": {
    "limit": 100,
    "offset": 0,
    "count": 100
  }
}
```

## 错误响应

所有接口在出错时会返回相应的 HTTP 状态码和错误信息：

```json
{
  "error": "Error description"
}
```

常见状态码：

- `400` - 请求参数错误
- `500` - 服务器内部错误

## 

###
