use borsh::{BorshDeserialize, BorshSerialize};
use serde::Serialize;

#[derive(Debug, BorshDeserialize)]
#[borsh(use_discriminant = true)]
pub enum IncenseType {
    // 清香
    FaintScent = 0,
    // 橙香
    OrangeIncense = 1,
    // 龙涎香
    Ambergris = 2,
    // 灵香
    Lingxiang = 3,
    // 秘制香
    SecretIncense = 4,
    // 天界香
    CelestialIncense = 5,
}
#[derive(Debug, BorshDeserialize, BorshSerialize, Serialize)]
pub enum MedalLevel {
    None,
    Bronze,
    Silver,
    Gold,
    Supreme,
}

#[derive(Debug, BorshDeserialize, Serialize)]
pub enum ActivityEnum {
    Burn,
    Donate,
    Lottery,
    Wish,
    Like,
}

#[derive(Debug, BorshDeserialize, Serialize)]
#[borsh(use_discriminant = true)]
pub enum LotteryType {
    GreatFortune = 0,    // 大吉
    MiddleFortune = 1,   // 中吉
    SmallFortune = 2,    // 小吉
    Fortune = 3,         // 吉
    LateFortune = 4,     // 末吉
    Misfortune = 5,      // 凶
    GreatMisfortune = 6, // 大凶
}
