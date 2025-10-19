# Solji Indexer API 对接文档

## 概述

Solji Indexer 是一个基于 Rust 和 Axum 构建的 Web API 服务，用于处理寺庙合约的事件数据，提供查询接口给前端应用。

**服务地址：** `http://localhost:3001` (默认端口)

## API 端点总览

### 基础接口
- `GET /health` - 健康检查
- `GET /api/stats` - 全局统计数据
- `GET /api/temple/level` - 寺庙等级信息
- `GET /api/temple/stats` - 寺庙统计信息

### 香火相关接口
- `GET /api/incense/types` - 获取香火类型
- `GET /api/incense/can-burn` - 检查是否可以燃烧香火
- `GET /api/incense/user/{user_pubkey}/burn-count` - 获取用户香火燃烧次数
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

### 管理接口
- `GET /api/admin/update-leaderboard` - 更新排行榜（管理员）

## 详细接口文档

### 基础接口

#### 健康检查
```http
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

#### 全局统计
```http
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

#### 寺庙等级
```http
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

### 香火相关接口

#### 获取香火类型
```http
GET /api/incense/types
```

**响应：**
```json
{
  "incense_types": [
    {
      "id": "basic",
      "name": "清香",
      "name_en": "Basic Incense",
      "price": 0.01,
      "merit_points": 1,
      "description": "Simple and pure, for daily devotion",
      "image": "/traditional-incense-stick-glowing.jpg",
      "daily_limit": 10
    },
    {
      "id": "sandalwood",
      "name": "檀香",
      "name_en": "Sandalwood",
      "price": 0.05,
      "merit_points": 5,
      "description": "Premium sandalwood for deeper meditation",
      "image": "/sandalwood-incense-with-golden-glow.jpg",
      "daily_limit": 10
    },
    {
      "id": "dragon",
      "name": "龙香",
      "name_en": "Dragon Incense",
      "price": 0.1,
      "merit_points": 10,
      "description": "Rare dragon incense for great fortune",
      "image": "/mystical-dragon-incense-with-purple-smoke.jpg",
      "daily_limit": 10
    },
    {
      "id": "supreme",
      "name": "至尊香",
      "name_en": "Supreme Incense",
      "price": 0.3,
      "merit_points": 30,
      "description": "The ultimate offering for enlightenment",
      "image": "/supreme-golden-incense-with-rainbow-aura.jpg",
      "daily_limit": 10
    }
  ]
}
```

#### 检查燃烧香火限制
```http
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

#### 用户香火燃烧次数
```http
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

### 许愿相关接口

#### 分页获取许愿
```http
GET /api/wishes?limit=20&offset=0
```

**查询参数：**
- `limit`: 每页数量 (默认20, 最大100)
- `offset`: 偏移量 (默认0)

**响应：**
```json
{
  "wishes": [
    {
      "id": 1,
      "wish_id": "wish_001",
      "user_pubkey": "UserPubkey...",
      "content": "祈求平安健康",
      "likes": 10,
      "created_at": "2025-01-01T10:00:00Z",
      "updated_at": "2025-01-01T10:00:00Z"
    }
  ],
  "pagination": {
    "limit": 20,
    "offset": 0,
    "count": 1
  }
}
```

#### 用户许愿塔信息
```http
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

### 捐赠相关接口

#### 获取捐赠等级
```http
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

#### 提交捐赠交易
```http
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

### 护身符相关接口

#### 获取待领取护身符
```http
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

#### 获取最近护身符掉落
```http
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

## 排行榜接口

### 香火排行榜
```http
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
```http
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

## 使用示例

### JavaScript/TypeScript 客户端示例

```typescript
class SoljiIndexerClient {
  private baseUrl: string;

  constructor(baseUrl = 'http://localhost:3001') {
    this.baseUrl = baseUrl;
  }

  async getTempleStats() {
    const response = await fetch(`${this.baseUrl}/api/temple/stats`);
    return response.json();
  }

  async checkCanBurnIncense(userPubkey: string, incenseType: number, amount: number) {
    const params = new URLSearchParams({
      user: userPubkey,
      incense_type: incenseType.toString(),
      amount: amount.toString()
    });
    const response = await fetch(`${this.baseUrl}/api/incense/can-burn?${params}`);
    return response.json();
  }

  async submitDonation(donation: {
    user_pubkey: string;
    amount_sol: number;
    tier: string;
    transaction_signature: string;
  }) {
    const response = await fetch(`${this.baseUrl}/api/donation/submit`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(donation)
    });
    return response.json();
  }
}

// 使用示例
const client = new SoljiIndexerClient();
const stats = await client.getTempleStats();
console.log('Temple level:', stats.level);
```

## 注意事项

1. **分页限制**：排行榜和列表接口都有最大限制，请注意分页处理
2. **数据一致性**：数据来源于区块链事件索引，确保数据的实时性
3. **错误处理**：前端需要处理各种错误情况，包括网络错误和服务器错误
4. **缓存策略**：建议对一些静态数据（如香火类型、捐赠等级）进行适当缓存
5. **实时更新**：排行榜数据会定期更新，不是实时数据

## 部署说明

- 默认运行端口：3001
- 支持 Docker 部署（见 docker-compose.yml）
- 数据库：PostgreSQL
- 区块链连接：通过 RPC 节点获取事件数据
